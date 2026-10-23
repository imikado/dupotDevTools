mod criteria;
pub mod geometry;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

use adw::prelude::*;
use gtk::{gdk, gio, glib, graphene};

use self::criteria::CriteriaRow;
use self::geometry::{clamp_to_span, facing_edge_x, hit_test, link_curve};
use crate::domain::contract::connection_repository::ConnectionRepository;
use crate::domain::contract::database_repository::DatabaseRepository;
use crate::domain::entity::database::{
    ConnectionConfig, ConnectionForm, DbKind, ForeignKey, QueryResult,
};
use crate::domain::error::DomainResult;
use crate::domain::feature::database::sql_viewer::{
    self as logic, CanvasJoin, CanvasTable, MAX_RESULT_ROWS, ParsedSelect,
};
use crate::domain::feature::formating::sql_format;
use crate::infrastructure::api::connection_store_api::ConnectionStoreApi;
use crate::infrastructure::api::database_api::DatabaseApi;
use crate::infrastructure::ui::feature_contract::{FeatureContract, FeatureView, LeaveGuard};
use crate::infrastructure::ui::shared::input_output_widget::timestamp;

const LINK_COLOR: (f64, f64, f64) = (0.22, 0.51, 0.89);
/// Pause after the last keystroke before the SQL is redrawn on the canvas.
const SQL_SYNC_DELAY: std::time::Duration = std::time::Duration::from_millis(700);
/// Table cards scroll their column list past this height.
const CARD_MAX_LIST_HEIGHT: i32 = 380;
/// Height of one column row in a table card (its link button sets it).
const CARD_ROW_HEIGHT: i32 = 34;
/// Canvas size at 100 % zoom.
const CANVAS_SIZE: (f64, f64) = (2000.0, 1200.0);
const ZOOM_MIN: f64 = 0.3;
const ZOOM_MAX: f64 = 2.0;
const ZOOM_STEP: f64 = 0.1;

pub struct Feature;

impl FeatureContract for Feature {
    fn full_width(&self) -> bool {
        true
    }

    fn scrollable(&self) -> bool {
        false
    }

    fn widget(&self) -> gtk::Widget {
        self.view().widget
    }

    fn view(&self) -> FeatureView {
        let (widget, viewer) = SqlViewer::build();
        let leave_guard: Weak<dyn LeaveGuard> = viewer;
        FeatureView {
            widget,
            leave_guard: Some(leave_guard),
        }
    }
}

struct ColumnWidgets {
    name: String,
    check: gtk::CheckButton,
    anchor: gtk::ToggleButton,
}

struct TableCard {
    table: String,
    frame: gtk::Frame,
    /// Scrolled list of the columns; rows scrolled out of it are hidden.
    fields_scroll: gtk::ScrolledWindow,
    /// Top-left corner on the canvas at 100 % zoom.
    pos: Cell<(f64, f64)>,
    columns: Vec<ColumnWidgets>,
    foreign_keys: Vec<ForeignKey>,
}

impl TableCard {
    fn column(&self, name: &str) -> Option<&ColumnWidgets> {
        self.columns.iter().find(|c| c.name == name).or_else(|| {
            self.columns
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case(name))
        })
    }
}

#[derive(Clone)]
struct Side {
    card: Rc<TableCard>,
    col: String,
    anchor: gtk::ToggleButton,
}

struct Join {
    left: Side,
    right: Side,
}

struct Hitbox {
    join: Rc<Join>,
    points: [(f64, f64); 4],
}

/// Widgets and state of one SQL viewer page. Signal handlers hold a `Weak`
/// to it; the root widget's destroy handler owns the only strong reference,
/// so leaving the page drops everything, including the DB connection.
struct SqlViewer {
    db: RefCell<Box<dyn DatabaseRepository>>,
    store: Box<dyn ConnectionRepository>,
    connections: RefCell<Vec<ConnectionConfig>>,
    /// File the connections were last opened from or saved to.
    connections_file: RefCell<Option<PathBuf>>,
    /// Connections changed since they were last opened or saved.
    unsaved: Cell<bool>,
    tables_cache: RefCell<Vec<String>>,
    table_cards: RefCell<Vec<Rc<TableCard>>>,
    joins: RefCell<Vec<Rc<Join>>>,
    pending_link: RefCell<Option<Side>>,
    link_hitboxes: RefCell<Vec<Hitbox>>,
    last_result: RefCell<QueryResult>,
    criteria_rows: RefCell<Vec<Rc<CriteriaRow>>>,
    /// Set while the code itself changes widgets, so their signal handlers
    /// don't treat it as a user edit.
    syncing: Cell<bool>,
    /// Last SQL written from the canvas; while the editor still holds it,
    /// canvas edits rewrite it.
    generated_sql: RefCell<String>,
    /// Row cap of the built query: the typed query's `LIMIT`, if any.
    limit: Cell<usize>,
    /// Canvas scale, 1.0 = 100 %.
    zoom: Cell<f64>,
    /// Bumped on each SQL edit; only the latest one's timer redraws the canvas.
    sql_edit_serial: Cell<u32>,

    conn_model: gtk::StringList,
    conn_dropdown: gtk::DropDown,
    connect_btn: gtk::Button,
    new_conn_btn: gtk::Button,
    conn_state_lbl: gtk::Label,
    /// Holds the connection bar, folded away once connected.
    conn_revealer: gtk::Revealer,
    conn_toggle: gtk::ToggleButton,
    /// Toolbar and panes, shown only while connected.
    workspace: gtk::Box,
    disconnected_page: adw::StatusPage,
    status_lbl: gtk::Label,
    zoom_btn: gtk::Button,
    fixed: gtk::Fixed,
    canvas_placeholder: gtk::Box,
    link_lines: gtk::DrawingArea,
    joins_box: gtk::Box,
    joins_btn: gtk::MenuButton,
    criteria_grid: gtk::Grid,
    add_criteria_btn: gtk::MenuButton,
    sql_buffer: gtk::TextBuffer,
    results_toggle: gtk::ToggleButton,
    results_lbl: gtk::Label,
    results_scroll: gtk::ScrolledWindow,
    export_csv_btn: gtk::Button,
}

fn heading(text: &str) -> gtk::Label {
    let lbl = gtk::Label::builder().label(text).xalign(0.0).build();
    lbl.add_css_class("heading");
    lbl
}

fn hint(text: &str) -> gtk::Label {
    let lbl = gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .wrap(true)
        .build();
    lbl.add_css_class("dim-label");
    lbl.add_css_class("caption");
    lbl
}

fn close_button() -> gtk::Button {
    let btn = gtk::Button::from_icon_name("window-close-symbolic");
    btn.add_css_class("flat");
    btn.add_css_class("circular");
    btn
}

fn results_placeholder(text: &str) -> gtk::Label {
    let lbl = gtk::Label::new(Some(text));
    lbl.add_css_class("dim-label");
    lbl.set_margin_top(24);
    lbl.set_margin_bottom(24);
    lbl
}

fn popover_at(parent: &impl IsA<gtk::Widget>, x: f64, y: f64) -> gtk::Popover {
    let popover = gtk::Popover::new();
    popover.set_parent(parent);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
    popover.connect_closed(|p| p.unparent());
    popover
}

/// Connects at the vertical height of the linked column, but always on the
/// frame edge that faces the other table — so the line hugs the outside of
/// both cards instead of cutting across whichever one sits in between. A
/// column scrolled out of its card's list attaches at the list's edge.
fn edge_point(
    target: &gtk::DrawingArea,
    anchor: &gtk::ToggleButton,
    card: &TableCard,
    facing: &gtk::Frame,
) -> Option<(f64, f64)> {
    let a = anchor.compute_point(
        target,
        &graphene::Point::new(anchor.width() as f32 / 2.0, anchor.height() as f32 / 2.0),
    )?;
    // `(x, width)` on the canvas, zoom included.
    let span = |frame: &gtk::Frame| {
        let left = frame.compute_point(target, &graphene::Point::zero())?;
        let right =
            frame.compute_point(target, &graphene::Point::new(frame.width() as f32, 0.0))?;
        Some((left.x() as f64, (right.x() - left.x()) as f64))
    };
    let list_top = card
        .fields_scroll
        .compute_point(target, &graphene::Point::zero())?;
    let list_bottom = card.fields_scroll.compute_point(
        target,
        &graphene::Point::new(0.0, card.fields_scroll.height() as f32),
    )?;
    let edge_x = facing_edge_x(span(&card.frame)?, span(facing)?);
    let y = clamp_to_span(
        a.y() as f64,
        list_top.y() as f64,
        (list_bottom.y() - list_top.y()) as f64,
    );
    Some((edge_x, y))
}

macro_rules! with_viewer {
    ($weak:expr, |$this:ident| $body:expr) => {{
        let weak = $weak.clone();
        move |_: &_| {
            if let Some($this) = weak.upgrade() {
                $body;
            }
        }
    }};
}

impl SqlViewer {
    /// The page's root widget and a weak handle to the viewer it owns.
    fn build() -> (gtk::Widget, Weak<Self>) {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 8);

        // --- Connection bar ---
        let conn_model = gtk::StringList::new(&[]);
        let conn_dropdown = gtk::DropDown::new(Some(conn_model.clone()), gtk::Expression::NONE);
        conn_dropdown.set_hexpand(true);
        let open_conns_btn = gtk::Button::builder()
            .icon_name("document-open-symbolic")
            .tooltip_text("Open a connections file…")
            .build();
        let save_conns_btn = gtk::Button::builder()
            .icon_name("document-save-symbolic")
            .tooltip_text("Save the connections (without passwords)")
            .build();
        let new_conn_btn = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("New connection…")
            .build();
        let conn_file_btns = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        conn_file_btns.add_css_class("linked");
        conn_file_btns.append(&open_conns_btn);
        conn_file_btns.append(&save_conns_btn);
        conn_file_btns.append(&new_conn_btn);
        let connect_btn = gtk::Button::with_label("Connect");
        connect_btn.add_css_class("suggested-action");
        connect_btn.set_sensitive(false);
        conn_dropdown.set_sensitive(false);

        let conn_state_lbl = gtk::Label::new(None);
        conn_state_lbl.add_css_class("caption");
        conn_state_lbl.set_valign(gtk::Align::Center);

        let conn_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        conn_bar.append(&conn_state_lbl);
        conn_bar.append(&conn_dropdown);
        conn_bar.append(&conn_file_btns);
        conn_bar.append(&connect_btn);
        let conn_revealer = gtk::Revealer::builder()
            .child(&conn_bar)
            .reveal_child(true)
            .build();

        let status_lbl = gtk::Label::new(None);
        status_lbl.add_css_class("dim-label");
        status_lbl.add_css_class("caption");
        status_lbl.set_halign(gtk::Align::Start);
        status_lbl.set_wrap(true);

        // --- Diagram pane ---
        let fixed = gtk::Fixed::new();
        fixed.set_size_request(CANVAS_SIZE.0 as i32, CANVAS_SIZE.1 as i32);
        fixed.add_css_class("view");

        let link_lines = gtk::DrawingArea::new();
        link_lines.set_can_target(false);

        let canvas_overlay = gtk::Overlay::new();
        canvas_overlay.set_child(Some(&fixed));
        canvas_overlay.add_overlay(&link_lines);

        let canvas_scroll = gtk::ScrolledWindow::new();
        canvas_scroll.add_css_class("card");
        canvas_scroll.set_vexpand(true);
        canvas_scroll.set_hexpand(true);
        canvas_scroll.set_child(Some(&canvas_overlay));

        let canvas_placeholder = gtk::Box::new(gtk::Orientation::Vertical, 6);
        canvas_placeholder.set_halign(gtk::Align::Center);
        canvas_placeholder.set_valign(gtk::Align::Center);
        canvas_placeholder.set_can_target(false);
        canvas_placeholder.add_css_class("dim-label");
        let placeholder_icon = gtk::Image::from_icon_name("network-server-symbolic");
        placeholder_icon.set_pixel_size(48);
        let placeholder_title = gtk::Label::new(Some("No tables on the canvas"));
        placeholder_title.add_css_class("title-3");
        canvas_placeholder.append(&placeholder_icon);
        canvas_placeholder.append(&placeholder_title);
        canvas_placeholder.append(&gtk::Label::new(Some(
            "Connect, then right-click here to add a table — foreign keys are linked automatically.\n\
             Link two columns with their link icons; click a join line to change it.",
        )));
        if let Some(lbl) = canvas_placeholder.last_child().and_downcast::<gtk::Label>() {
            lbl.set_justify(gtk::Justification::Center);
        }

        let diagram_pane = gtk::Overlay::new();
        diagram_pane.set_child(Some(&canvas_scroll));
        diagram_pane.add_overlay(&canvas_placeholder);
        diagram_pane.set_size_request(-1, 120);

        // --- Criteria pane ---
        let criteria_grid = gtk::Grid::builder()
            .row_spacing(2)
            .column_spacing(6)
            .margin_top(6)
            .margin_bottom(6)
            .margin_start(8)
            .margin_end(8)
            .build();
        let criteria_scroll = gtk::ScrolledWindow::new();
        criteria_scroll.add_css_class("card");
        criteria_scroll.set_vexpand(true);
        criteria_scroll.set_child(Some(&criteria_grid));
        let add_criteria_btn = gtk::MenuButton::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add a column to filter or sort on")
            .build();
        add_criteria_btn.add_css_class("flat");
        let criteria_pane = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let criteria_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let criteria_title = heading("Criteria");
        let criteria_hint = hint(
            "Filter: “> 8”, “LIKE 'A%'”, “IS NULL” or a value. Rows must match every Filter, or every Or….",
        );
        criteria_hint.set_hexpand(true);
        criteria_header.append(&criteria_title);
        criteria_header.append(&criteria_hint);
        criteria_header.append(&add_criteria_btn);
        criteria_pane.append(&criteria_header);
        criteria_pane.append(&criteria_scroll);
        criteria_pane.set_size_request(-1, 100);

        // --- SQL pane ---
        let sql_buffer = gtk::TextBuffer::new(None);
        let sql_view = gtk::TextView::with_buffer(&sql_buffer);
        sql_view.add_css_class("monospace");
        sql_view.set_wrap_mode(gtk::WrapMode::WordChar);
        sql_view.set_top_margin(8);
        sql_view.set_bottom_margin(8);
        sql_view.set_left_margin(8);
        sql_view.set_right_margin(8);

        let sql_scroll = gtk::ScrolledWindow::new();
        sql_scroll.add_css_class("card");
        sql_scroll.set_vexpand(true);
        sql_scroll.set_hexpand(true);
        sql_scroll.set_child(Some(&sql_view));
        let sql_pane = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let sql_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let sql_hint = hint(
            "Follows the canvas until you edit it. Type or paste a SELECT to draw it on the canvas.",
        );
        sql_hint.set_hexpand(true);
        sql_header.append(&heading("SQL"));
        sql_header.append(&sql_hint);
        sql_pane.append(&sql_header);
        sql_pane.append(&sql_scroll);
        sql_pane.set_size_request(-1, 90);

        // --- Results pane ---
        let results_lbl = heading("Results");
        results_lbl.set_hexpand(true);
        results_lbl.set_xalign(0.0);
        let export_csv_btn = gtk::Button::builder()
            .icon_name("document-save-as-symbolic")
            .tooltip_text("Export the results as CSV…")
            .build();
        export_csv_btn.add_css_class("flat");
        export_csv_btn.set_sensitive(false);
        let results_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        results_header.append(&results_lbl);
        results_header.append(&export_csv_btn);

        let results_scroll = gtk::ScrolledWindow::new();
        results_scroll.add_css_class("card");
        results_scroll.set_vexpand(true);
        results_scroll.set_hexpand(true);
        results_scroll.set_child(Some(&results_placeholder(
            "Run the query to see results here.",
        )));
        let results_pane = gtk::Box::new(gtk::Orientation::Vertical, 4);
        results_pane.append(&results_header);
        results_pane.append(&results_scroll);
        results_pane.set_size_request(-1, 110);

        // --- Panes: diagram / criteria / SQL / results, each resizable ---
        let paned = |start: &gtk::Widget, end: &gtk::Widget, position: i32| {
            gtk::Paned::builder()
                .orientation(gtk::Orientation::Vertical)
                .wide_handle(true)
                .shrink_start_child(false)
                .shrink_end_child(false)
                .start_child(start)
                .end_child(end)
                .position(position)
                .vexpand(true)
                .build()
        };
        let sql_results = paned(sql_pane.upcast_ref(), results_pane.upcast_ref(), 130);
        let lower = paned(criteria_pane.upcast_ref(), sql_results.upcast_ref(), 150);
        let panes = paned(diagram_pane.upcast_ref(), lower.upcast_ref(), 300);

        let pane_toggle = |icon: &str, label: &str, tooltip: &str| {
            gtk::ToggleButton::builder()
                .child(
                    &adw::ButtonContent::builder()
                        .icon_name(icon)
                        .label(label)
                        .build(),
                )
                .tooltip_text(tooltip)
                .active(true)
                .build()
        };
        let diagram_toggle = pane_toggle(
            "view-grid-symbolic",
            "Diagram",
            "Show or hide the tables canvas",
        );
        let criteria_toggle = pane_toggle(
            "view-list-symbolic",
            "Criteria",
            "Show or hide the criteria grid",
        );
        let sql_toggle = pane_toggle(
            "text-x-generic-symbolic",
            "SQL",
            "Show or hide the SQL query",
        );
        let results_toggle = pane_toggle(
            "x-office-spreadsheet-symbolic",
            "Results",
            "Show or hide the results",
        );
        let pane_toggles = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        pane_toggles.add_css_class("linked");
        for (toggle, pane) in [
            (&diagram_toggle, diagram_pane.upcast_ref::<gtk::Widget>()),
            (&criteria_toggle, criteria_pane.upcast_ref()),
            (&sql_toggle, sql_pane.upcast_ref()),
            (&results_toggle, results_pane.upcast_ref()),
        ] {
            toggle
                .bind_property("active", pane, "visible")
                .sync_create()
                .build();
            pane_toggles.append(toggle);
        }
        // A paned whose two panes are hidden must hide too, or it keeps its space.
        let sync_paneds = glib::clone!(
            #[weak]
            sql_pane,
            #[weak]
            results_pane,
            #[weak]
            criteria_pane,
            #[weak]
            sql_results,
            #[weak]
            lower,
            move || {
                sql_results.set_visible(sql_pane.is_visible() || results_pane.is_visible());
                lower.set_visible(criteria_pane.is_visible() || sql_results.is_visible());
            }
        );
        for pane in [&sql_pane, &results_pane, &criteria_pane] {
            let sync = sync_paneds.clone();
            pane.connect_visible_notify(move |_| sync());
        }
        // The designer writes the query: its text is shown on demand.
        sql_toggle.set_active(false);

        let joins_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        joins_box.set_margin_top(6);
        joins_box.set_margin_bottom(6);
        joins_box.set_margin_start(6);
        joins_box.set_margin_end(6);
        let joins_btn = gtk::MenuButton::builder()
            .label("Joins")
            .tooltip_text("Joins between the canvas tables")
            .popover(&gtk::Popover::builder().child(&joins_box).build())
            .build();

        let generate_sql_btn = gtk::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Rebuild the query from the canvas and criteria")
            .build();
        let run_btn = gtk::Button::builder()
            .child(
                &adw::ButtonContent::builder()
                    .icon_name("media-playback-start-symbolic")
                    .label("Run")
                    .build(),
            )
            .tooltip_text("Run the query (Ctrl+Enter)")
            .build();
        run_btn.add_css_class("suggested-action");

        let conn_toggle = gtk::ToggleButton::builder()
            .icon_name("network-server-symbolic")
            .tooltip_text("Show or hide the connection")
            .build();
        conn_revealer
            .bind_property("reveal-child", &conn_toggle, "active")
            .bidirectional()
            .sync_create()
            .build();

        let zoom_out_btn = gtk::Button::builder()
            .icon_name("zoom-out-symbolic")
            .tooltip_text("Zoom out the tables (Ctrl+scroll)")
            .build();
        let zoom_btn = gtk::Button::builder()
            .label("100%")
            .tooltip_text("Reset the zoom")
            .build();
        let zoom_in_btn = gtk::Button::builder()
            .icon_name("zoom-in-symbolic")
            .tooltip_text("Zoom in the tables (Ctrl+scroll)")
            .build();
        let zoom_btns = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        zoom_btns.add_css_class("linked");
        zoom_btns.append(&zoom_out_btn);
        zoom_btns.append(&zoom_btn);
        zoom_btns.append(&zoom_in_btn);

        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        toolbar.append(&conn_toggle);
        toolbar.append(&pane_toggles);
        toolbar.append(&zoom_btns);
        toolbar.append(&spacer);
        toolbar.append(&joins_btn);
        toolbar.append(&generate_sql_btn);
        toolbar.append(&run_btn);

        let workspace = gtk::Box::new(gtk::Orientation::Vertical, 8);
        workspace.set_vexpand(true);
        workspace.append(&toolbar);
        workspace.append(&panes);

        let disconnected_page = adw::StatusPage::builder()
            .icon_name("network-server-symbolic")
            .title("Not connected")
            .description("Choose a connection above, or create one with +, then Connect.")
            .vexpand(true)
            .build();

        root.append(&conn_revealer);
        root.append(&status_lbl);
        root.append(&disconnected_page);
        root.append(&workspace);

        let this = Rc::new(Self {
            db: RefCell::new(Box::new(DatabaseApi::default())),
            store: Box::new(ConnectionStoreApi::default()),
            connections: RefCell::default(),
            connections_file: RefCell::default(),
            unsaved: Cell::new(false),
            tables_cache: RefCell::default(),
            table_cards: RefCell::default(),
            joins: RefCell::default(),
            pending_link: RefCell::default(),
            link_hitboxes: RefCell::default(),
            last_result: RefCell::default(),
            criteria_rows: RefCell::default(),
            syncing: Cell::new(false),
            generated_sql: RefCell::default(),
            limit: Cell::new(MAX_RESULT_ROWS),
            zoom: Cell::new(1.0),
            sql_edit_serial: Cell::new(0),
            conn_model,
            conn_dropdown,
            connect_btn: connect_btn.clone(),
            new_conn_btn,
            conn_state_lbl,
            conn_revealer,
            conn_toggle,
            workspace,
            disconnected_page,
            status_lbl,
            zoom_btn: zoom_btn.clone(),
            fixed,
            canvas_placeholder,
            link_lines,
            joins_box,
            joins_btn,
            criteria_grid,
            add_criteria_btn,
            sql_buffer,
            results_toggle,
            results_lbl,
            results_scroll,
            export_csv_btn,
        });
        let weak = Rc::downgrade(&this);

        let draw_weak = weak.clone();
        this.link_lines.set_draw_func(move |_, cr, _, _| {
            if let Some(this) = draw_weak.upgrade() {
                this.draw_links(cr);
            }
        });

        this.new_conn_btn
            .connect_clicked(with_viewer!(weak, |this| this.open_new_connection_dialog()));
        connect_btn.connect_clicked(with_viewer!(weak, |this| this.do_connect()));
        open_conns_btn.connect_clicked(with_viewer!(weak, |this| this.do_open_connections()));
        save_conns_btn.connect_clicked(with_viewer!(weak, |this| this.do_save_connections()));
        generate_sql_btn.connect_clicked(with_viewer!(weak, |this| this.do_generate_sql()));
        run_btn.connect_clicked(with_viewer!(weak, |this| this.do_run_query()));
        zoom_out_btn
            .connect_clicked(with_viewer!(weak, |this| this.set_zoom(this.zoom.get() - ZOOM_STEP)));
        zoom_in_btn
            .connect_clicked(with_viewer!(weak, |this| this.set_zoom(this.zoom.get() + ZOOM_STEP)));
        zoom_btn.connect_clicked(with_viewer!(weak, |this| this.set_zoom(1.0)));
        let zoom_scroll =
            gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let w = weak.clone();
        zoom_scroll.connect_scroll(move |ctrl, _, dy| {
            let ctrl_held = ctrl
                .current_event_state()
                .contains(gdk::ModifierType::CONTROL_MASK);
            match w.upgrade() {
                Some(this) if ctrl_held => {
                    this.set_zoom(this.zoom.get() - dy.signum() * ZOOM_STEP);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        zoom_scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
        canvas_scroll.add_controller(zoom_scroll);
        this.export_csv_btn
            .connect_clicked(with_viewer!(weak, |this| this.do_export_csv()));

        let w = weak.clone();
        let run_shortcut = gtk::Shortcut::new(
            gtk::ShortcutTrigger::parse_string("<Control>Return"),
            Some(gtk::CallbackAction::new(move |_, _| {
                if let Some(this) = w.upgrade() {
                    this.do_run_query();
                }
                glib::Propagation::Stop
            })),
        );
        let shortcuts = gtk::ShortcutController::new();
        shortcuts.add_shortcut(run_shortcut);
        sql_view.add_controller(shortcuts);

        let w = weak.clone();
        this.sql_buffer.connect_changed(move |_| {
            if let Some(this) = w.upgrade() {
                this.on_sql_edited();
            }
        });

        let right_click = gtk::GestureClick::builder()
            .button(gdk::BUTTON_SECONDARY)
            .build();
        let w = weak.clone();
        right_click.connect_pressed(move |gesture, _, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            if let Some(this) = w.upgrade() {
                this.on_canvas_right_click(x, y);
            }
        });
        this.fixed.add_controller(right_click);

        let left_click = gtk::GestureClick::builder()
            .button(gdk::BUTTON_PRIMARY)
            .build();
        let w = weak.clone();
        left_click.connect_pressed(move |gesture, _, x, y| {
            let Some(this) = w.upgrade() else { return };
            if let Some(join) = this.find_join_at(x, y, 7.0) {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                this.show_join_editor(join, x, y);
            }
        });
        this.fixed.add_controller(left_click);

        this.refresh_joins_panel();
        this.setup_add_criteria_menu();
        this.rebuild_criteria_grid();
        this.update_connection_state();
        this.load_connections();

        root.connect_destroy(move |_| {
            let _ = &this;
        });
        (root.upcast(), weak)
    }

    fn set_error(&self, message: &str) {
        self.status_lbl.remove_css_class("dim-label");
        self.status_lbl.add_css_class("error");
        self.status_lbl.set_text(&format!("⚠ Error: {message}"));
    }

    fn set_status(&self, message: &str) {
        self.status_lbl.remove_css_class("error");
        self.status_lbl.add_css_class("dim-label");
        self.status_lbl.set_text(message);
    }

    // --- Link lines ---

    fn draw_links(&self, cr: &gtk::cairo::Context) {
        let (r, g, b) = LINK_COLOR;
        let mut hitboxes = self.link_hitboxes.borrow_mut();
        hitboxes.clear();
        for join in self.joins.borrow().iter() {
            let (l, rt) = (&join.left, &join.right);
            let p1 = edge_point(&self.link_lines, &l.anchor, &l.card, &rt.card.frame);
            let p2 = edge_point(&self.link_lines, &rt.anchor, &rt.card, &l.card.frame);
            let (Some(from), Some(to)) = (p1, p2) else {
                continue;
            };
            let [p0, c1, c2, p3] = link_curve(from, to);
            cr.set_source_rgba(r, g, b, 0.9);
            cr.set_line_width(2.0);
            cr.move_to(p0.0, p0.1);
            cr.curve_to(c1.0, c1.1, c2.0, c2.1, p3.0, p3.1);
            let _ = cr.stroke();
            for (px, py) in [p0, p3] {
                cr.arc(px, py, 3.5, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            }
            hitboxes.push(Hitbox {
                join: Rc::clone(join),
                points: [p0, c1, c2, p3],
            });
        }
    }

    fn find_join_at(&self, x: f64, y: f64, tolerance: f64) -> Option<Rc<Join>> {
        let hitboxes = self.link_hitboxes.borrow();
        let curves: Vec<_> = hitboxes.iter().map(|hb| hb.points).collect();
        hit_test(&curves, x, y, tolerance).map(|i| Rc::clone(&hitboxes[i].join))
    }

    // --- Joins panel ---

    fn refresh_joins_panel(self: &Rc<Self>) {
        while let Some(child) = self.joins_box.first_child() {
            self.joins_box.remove(&child);
        }
        let joins = self.joins.borrow();
        self.joins_btn
            .set_label(&format!("Joins ({})", joins.len()));
        if joins.is_empty() {
            self.joins_box.append(&hint(
                "No joins yet. Link a column from each table to combine them.",
            ));
            return;
        }
        for join in joins.iter() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let text = format!(
                "{}.{}  =  {}.{}",
                join.left.card.table, join.left.col, join.right.card.table, join.right.col
            );
            let lbl = gtk::Label::builder()
                .label(text)
                .xalign(0.0)
                .hexpand(true)
                .build();
            let remove_btn = close_button();
            let weak = Rc::downgrade(self);
            let join = Rc::downgrade(join);
            remove_btn.connect_clicked(move |_| {
                if let (Some(this), Some(join)) = (weak.upgrade(), join.upgrade()) {
                    this.remove_join(&join);
                }
            });
            row.append(&lbl);
            row.append(&remove_btn);
            self.joins_box.append(&row);
        }
    }

    fn joins_changed(self: &Rc<Self>) {
        self.refresh_joins_panel();
        self.link_lines.queue_draw();
        self.canvas_changed();
    }

    /// Runs `f` with widget signal handlers muted (see `syncing`).
    fn quietly<T>(&self, f: impl FnOnce() -> T) -> T {
        let was = self.syncing.replace(true);
        let result = f();
        self.syncing.set(was);
        result
    }

    fn remove_join(self: &Rc<Self>, join: &Rc<Join>) {
        self.joins.borrow_mut().retain(|j| !Rc::ptr_eq(j, join));
        let joins = self.joins.borrow();
        self.quietly(|| {
            for anchor in [&join.left.anchor, &join.right.anchor] {
                // An anchor stays lit while another join still uses it.
                if !joins
                    .iter()
                    .any(|j| j.left.anchor == *anchor || j.right.anchor == *anchor)
                {
                    anchor.set_active(false);
                }
            }
        });
        drop(joins);
        self.joins_changed();
    }

    /// Joins two columns without going through the anchors' click flow.
    /// Returns false when a column is missing or the join already exists.
    fn link_columns(
        &self,
        left: &Rc<TableCard>,
        left_col: &str,
        right: &Rc<TableCard>,
        right_col: &str,
    ) -> bool {
        let (Some(l), Some(r)) = (left.column(left_col), right.column(right_col)) else {
            return false;
        };
        if Rc::ptr_eq(left, right) {
            return false;
        }
        let exists = self.joins.borrow().iter().any(|j| {
            (j.left.anchor == l.anchor && j.right.anchor == r.anchor)
                || (j.left.anchor == r.anchor && j.right.anchor == l.anchor)
        });
        if exists {
            return false;
        }
        self.quietly(|| {
            l.anchor.set_active(true);
            r.anchor.set_active(true);
        });
        let side = |card: &Rc<TableCard>, col: &ColumnWidgets| Side {
            card: Rc::clone(card),
            col: col.name.clone(),
            anchor: col.anchor.clone(),
        };
        self.joins.borrow_mut().push(Rc::new(Join {
            left: side(left, l),
            right: side(right, r),
        }));
        true
    }

    // --- Join anchors ---

    fn on_anchor_toggled(self: &Rc<Self>, side: Side) {
        if self.syncing.get() {
            return;
        }
        let btn = side.anchor.clone();
        if !btn.is_active() {
            let removed: Vec<Rc<Join>> = {
                let mut joins = self.joins.borrow_mut();
                let (removed, kept) = joins
                    .drain(..)
                    .partition(|j| j.left.anchor == btn || j.right.anchor == btn);
                *joins = kept;
                removed
            };
            {
                let mut pending = self.pending_link.borrow_mut();
                if pending.as_ref().is_some_and(|p| p.anchor == btn) {
                    *pending = None;
                }
            }
            for j in &removed {
                let other = if j.left.anchor == btn {
                    &j.right
                } else {
                    &j.left
                };
                other.anchor.set_active(false);
            }
            if !removed.is_empty() {
                self.joins_changed();
            }
            return;
        }

        let pending = self.pending_link.borrow_mut().take();
        let Some(other) = pending else {
            *self.pending_link.borrow_mut() = Some(side);
            return;
        };

        if Rc::ptr_eq(&other.card, &side.card) {
            // Linking a table to itself doesn't make sense here — reset.
            other.anchor.set_active(false);
            btn.set_active(false);
            return;
        }

        self.joins.borrow_mut().push(Rc::new(Join {
            left: other,
            right: side,
        }));
        self.joins_changed();
    }

    // --- Table cards ---

    fn build_table_card(
        self: &Rc<Self>,
        table: &str,
        columns: &[(String, String)],
        foreign_keys: Vec<ForeignKey>,
    ) -> Rc<TableCard> {
        let frame = gtk::Frame::new(None);
        frame.add_css_class("card");
        frame.set_size_request(220, -1);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(6);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let table_icon = gtk::Image::from_icon_name("view-grid-symbolic");
        table_icon.add_css_class("dim-label");
        let name_lbl = heading(table);
        name_lbl.set_hexpand(true);
        name_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
        let count_lbl = gtk::Label::new(Some(&columns.len().to_string()));
        count_lbl.add_css_class("dim-label");
        count_lbl.add_css_class("caption");
        count_lbl.set_tooltip_text(Some("Columns"));
        let close_btn = close_button();
        close_btn.set_tooltip_text(Some("Remove from the canvas"));
        header.append(&table_icon);
        header.append(&name_lbl);
        header.append(&count_lbl);
        header.append(&close_btn);
        header.set_cursor_from_name(Some("move"));

        let fields_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        fields_box.set_margin_end(4);
        let mut column_widgets = Vec::new();
        for (col_name, col_type) in columns {
            let references = foreign_keys.iter().find(|fk| fk.column == *col_name);
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let check = gtk::CheckButton::new();
            check.set_active(true);
            check.set_tooltip_text(Some("Select this column"));
            let name_lbl = gtk::Label::builder()
                .label(col_name)
                .xalign(0.0)
                .hexpand(true)
                .build();
            // Clicking the name toggles the check box, like a check button label.
            check.set_child(Some(&name_lbl));
            check.set_hexpand(true);
            let type_text = match references {
                Some(fk) => format!("{col_type} → {}", fk.ref_table),
                None => col_type.clone(),
            };
            let type_lbl = gtk::Label::new(Some(&type_text));
            type_lbl.add_css_class("dim-label");
            type_lbl.add_css_class("caption");
            if let Some(fk) = references {
                row_box.set_tooltip_text(Some(&format!(
                    "References {}.{}",
                    fk.ref_table, fk.ref_column
                )));
            }
            let anchor = gtk::ToggleButton::new();
            anchor.set_icon_name("insert-link-symbolic");
            anchor.add_css_class("flat");
            anchor.add_css_class("circular");
            anchor.set_valign(gtk::Align::Center);
            anchor.set_tooltip_text(Some(
                "Link this column to another table's column to join them",
            ));
            row_box.append(&check);
            row_box.append(&type_lbl);
            row_box.append(&anchor);
            fields_box.append(&row_box);
            column_widgets.push(ColumnWidgets {
                name: col_name.clone(),
                check,
                anchor,
            });
        }
        if columns.is_empty() {
            let empty_lbl = gtk::Label::new(Some("No columns found"));
            empty_lbl.add_css_class("dim-label");
            fields_box.append(&empty_lbl);
        }

        let fields_scroll = gtk::ScrolledWindow::new();
        fields_scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
        // `gtk::Fixed` sizes cards to their minimum, so that is what must
        // fit every row (up to the cap).
        let rows_height = i32::try_from(columns.len())
            .unwrap_or(i32::MAX)
            .saturating_mul(CARD_ROW_HEIGHT);
        fields_scroll.set_min_content_height(rows_height.min(CARD_MAX_LIST_HEIGHT));
        fields_scroll.set_max_content_height(CARD_MAX_LIST_HEIGHT);
        fields_scroll.set_propagate_natural_height(true);
        fields_scroll.set_propagate_natural_width(true);
        fields_scroll.set_vexpand(true);
        fields_scroll.set_child(Some(&fields_box));
        // Join lines follow their columns as the list scrolls.
        fields_scroll
            .vadjustment()
            .connect_value_changed(glib::clone!(
                #[weak(rename_to = link_lines)]
                self.link_lines,
                move |_| link_lines.queue_draw()
            ));

        vbox.append(&header);
        vbox.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        vbox.append(&fields_scroll);

        let card_overlay = gtk::Overlay::new();
        card_overlay.set_child(Some(&vbox));

        let resize_handle = gtk::Label::new(Some("⤡"));
        resize_handle.add_css_class("dim-label");
        resize_handle.set_halign(gtk::Align::End);
        resize_handle.set_valign(gtk::Align::End);
        resize_handle.set_margin_end(3);
        resize_handle.set_margin_bottom(2);
        resize_handle.set_cursor_from_name(Some("se-resize"));
        resize_handle.set_tooltip_text(Some("Drag to resize this table"));
        card_overlay.add_overlay(&resize_handle);

        frame.set_child(Some(&card_overlay));

        let card = Rc::new(TableCard {
            table: table.to_string(),
            frame: frame.clone(),
            fields_scroll: fields_scroll.clone(),
            pos: Cell::new((0.0, 0.0)),
            columns: column_widgets,
            foreign_keys,
        });
        let weak = Rc::downgrade(self);
        let weak_card = Rc::downgrade(&card);

        for col in &card.columns {
            let w = weak.clone();
            col.check.connect_toggled(move |_| {
                if let Some(this) = w.upgrade()
                    && !this.syncing.get()
                {
                    this.sync_criteria_rows();
                    this.canvas_changed();
                }
            });
            let (w, wc, col_name) = (weak.clone(), weak_card.clone(), col.name.clone());
            col.anchor.connect_toggled(move |btn| {
                if let (Some(this), Some(card)) = (w.upgrade(), wc.upgrade()) {
                    this.on_anchor_toggled(Side {
                        card,
                        col: col_name.clone(),
                        anchor: btn.clone(),
                    });
                }
            });
        }

        let (w, wc) = (weak.clone(), weak_card.clone());
        close_btn.connect_clicked(move |_| {
            if let (Some(this), Some(card)) = (w.upgrade(), wc.upgrade()) {
                this.remove_table_card(&card);
            }
        });

        // Drag the header to move the card. The pointer is followed on the
        // canvas itself, so the card stays under it at any zoom.
        let drag_start = Rc::new(Cell::new(((0.0, 0.0), (0.0, 0.0))));
        let pointer_on_canvas = |gesture: &gtk::GestureDrag, this: &SqlViewer| {
            let (x, y) = gesture.point(None)?;
            let widget = gesture.widget()?;
            let p = widget.compute_point(&this.fixed, &graphene::Point::new(x as f32, y as f32))?;
            Some((p.x() as f64, p.y() as f64))
        };
        let drag = gtk::GestureDrag::new();
        let (w, wc, ds) = (weak.clone(), weak_card.clone(), Rc::clone(&drag_start));
        drag.connect_drag_begin(move |gesture, _, _| {
            if let (Some(this), Some(card)) = (w.upgrade(), wc.upgrade())
                && let Some(pointer) = pointer_on_canvas(gesture, &this)
            {
                ds.set((card.pos.get(), pointer));
            }
        });
        let (w, wc) = (weak.clone(), weak_card.clone());
        drag.connect_drag_update(move |gesture, _, _| {
            let (Some(this), Some(card)) = (w.upgrade(), wc.upgrade()) else {
                return;
            };
            let Some((px, py)) = pointer_on_canvas(gesture, &this) else {
                return;
            };
            let ((x, y), (sx, sy)) = drag_start.get();
            let z = this.zoom.get();
            this.move_card(
                &card,
                (x + (px - sx) / z).max(0.0),
                (y + (py - sy) / z).max(0.0),
            );
            this.link_lines.queue_draw();
        });
        header.add_controller(drag);

        // Drag the corner grip to resize the card.
        let resize_start = Rc::new(Cell::new((0, 0)));
        let resize_drag = gtk::GestureDrag::new();
        resize_drag.connect_drag_begin(glib::clone!(
            #[strong]
            resize_start,
            #[weak]
            frame,
            move |_, _, _| resize_start.set((frame.width(), frame.height()))
        ));
        resize_drag.connect_drag_update(glib::clone!(
            #[weak]
            frame,
            #[weak(rename_to = link_lines)]
            self.link_lines,
            move |_, dx, dy| {
                let (w, h) = resize_start.get();
                frame.set_size_request(
                    (w as f64 + dx).max(200.0) as i32,
                    (h as f64 + dy).max(120.0) as i32,
                );
                link_lines.queue_draw();
            }
        ));
        resize_handle.add_controller(resize_drag);

        card
    }

    fn remove_table_card(self: &Rc<Self>, card: &Rc<TableCard>) {
        self.fixed.remove(&card.frame);
        self.table_cards
            .borrow_mut()
            .retain(|c| !Rc::ptr_eq(c, card));
        {
            let mut pending = self.pending_link.borrow_mut();
            if pending.as_ref().is_some_and(|p| Rc::ptr_eq(&p.card, card)) {
                *pending = None;
            }
        }
        let stale: Vec<Rc<Join>> = self
            .joins
            .borrow()
            .iter()
            .filter(|j| Rc::ptr_eq(&j.left.card, card) || Rc::ptr_eq(&j.right.card, card))
            .cloned()
            .collect();
        for join in &stale {
            self.remove_join(join);
        }
        self.link_lines.queue_draw();
        self.update_canvas_placeholder();
        self.sync_criteria_rows();
        self.canvas_changed();
    }

    fn update_canvas_placeholder(&self) {
        self.canvas_placeholder
            .set_visible(self.table_cards.borrow().is_empty());
    }

    /// Puts a card on the canvas, without joins.
    fn place_table_card(self: &Rc<Self>, table: &str, x: f64, y: f64) -> Option<Rc<TableCard>> {
        let (columns, foreign_keys) = {
            let mut db = self.db.borrow_mut();
            let columns = db.list_columns(table);
            // Joins can still be drawn by hand if the keys can't be read.
            (columns, db.list_foreign_keys(table).unwrap_or_default())
        };
        let columns = match columns {
            Ok(c) => c,
            Err(e) => {
                self.set_error(e.message());
                return None;
            }
        };
        let card = self.build_table_card(table, &columns, foreign_keys);
        self.fixed.put(&card.frame, 0.0, 0.0);
        self.move_card(&card, x, y);
        self.table_cards.borrow_mut().push(Rc::clone(&card));
        self.update_canvas_placeholder();
        self.sync_criteria_rows();
        Some(card)
    }

    /// Places a card at `(x, y)` in unzoomed canvas coordinates.
    fn move_card(&self, card: &TableCard, x: f64, y: f64) {
        card.pos.set((x, y));
        let z = self.zoom.get();
        let transform = gtk::gsk::Transform::new()
            .translate(&graphene::Point::new((x * z) as f32, (y * z) as f32))
            .scale(z as f32, z as f32);
        self.fixed
            .set_child_transform(&card.frame, Some(&transform));
    }

    fn set_zoom(&self, zoom: f64) {
        // Rounded so repeated steps land on whole percents.
        let zoom = (zoom.clamp(ZOOM_MIN, ZOOM_MAX) * 100.0).round() / 100.0;
        self.zoom.set(zoom);
        for card in self.table_cards.borrow().iter() {
            let (x, y) = card.pos.get();
            self.move_card(card, x, y);
        }
        self.fixed
            .set_size_request((CANVAS_SIZE.0 * zoom) as i32, (CANVAS_SIZE.1 * zoom) as i32);
        self.zoom_btn.set_label(&format!("{:.0}%", zoom * 100.0));
        self.link_lines.queue_draw();
    }

    /// Free spot for a card placed by the code: a grid, row after row.
    fn next_card_position(&self) -> (f64, f64) {
        let n = self.table_cards.borrow().len();
        (30.0 + (n % 3) as f64 * 340.0, 30.0 + (n / 3) as f64 * 300.0)
    }

    fn add_table_card(self: &Rc<Self>, table: &str, x: f64, y: f64) {
        let Some(card) = self.place_table_card(table, x, y) else {
            return;
        };
        let cards = self.table_cards.borrow().clone();
        let fks: Vec<(&str, &[ForeignKey])> = cards
            .iter()
            .map(|c| (c.table.as_str(), c.foreign_keys.as_slice()))
            .collect();
        let new = cards.len() - 1;
        let mut linked = Vec::new();
        for j in logic::foreign_key_joins(&fks, new) {
            let (l, r) = (&cards[j.left_table], &cards[j.right_table]);
            if self.link_columns(l, &j.left_column, r, &j.right_column) {
                let other = if Rc::ptr_eq(l, &card) { r } else { l };
                linked.push(other.table.clone());
            }
        }
        self.joins_changed();
        if linked.is_empty() {
            self.set_status(&format!("Added “{table}” to the canvas."));
        } else {
            self.set_status(&format!(
                "Added “{table}”, joined to {} by foreign key.",
                linked.join(", ")
            ));
        }
    }

    fn on_canvas_right_click(self: &Rc<Self>, x: f64, y: f64) {
        if !self.db.borrow().is_connected() {
            return self.set_error("Connect to a database first.");
        }
        if self.tables_cache.borrow().is_empty() {
            return self.set_error("No tables found on this connection.");
        }
        self.show_table_picker(x, y);
    }

    fn show_table_picker(self: &Rc<Self>, x: f64, y: f64) {
        let popover = popover_at(&self.fixed, x, y);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);

        let on_canvas: Vec<String> = self
            .table_cards
            .borrow()
            .iter()
            .map(|c| c.table.clone())
            .collect();
        let tables: Vec<String> = self
            .tables_cache
            .borrow()
            .iter()
            .filter(|t| !on_canvas.contains(t))
            .cloned()
            .collect();
        if tables.is_empty() {
            vbox.append(&hint("Every table is already on the canvas."));
            popover.set_child(Some(&vbox));
            popover.popup();
            return;
        }

        let search = gtk::SearchEntry::builder()
            .placeholder_text("Search tables…")
            .build();
        let listbox = gtk::ListBox::new();
        listbox.add_css_class("navigation-sidebar");
        for table in &tables {
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row_box.append(&gtk::Image::from_icon_name("view-grid-symbolic"));
            row_box.append(&gtk::Label::builder().label(table).xalign(0.0).build());
            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&row_box));
            listbox.append(&row);
        }

        let filter_tables = tables.clone();
        listbox.set_filter_func(glib::clone!(
            #[weak]
            search,
            #[upgrade_or]
            true,
            move |row| {
                let query = search.text().to_lowercase();
                usize::try_from(row.index())
                    .ok()
                    .and_then(|i| filter_tables.get(i))
                    .is_some_and(|t| t.to_lowercase().contains(&query))
            }
        ));
        search.connect_search_changed(glib::clone!(
            #[weak]
            listbox,
            move |_| listbox.invalidate_filter()
        ));
        // Enter adds the first table still shown.
        search.connect_activate(glib::clone!(
            #[weak]
            listbox,
            move |_| {
                let mut child = listbox.first_child();
                while let Some(widget) = child {
                    if let Some(row) = widget.downcast_ref::<gtk::ListBoxRow>()
                        && row.is_child_visible()
                    {
                        row.activate();
                        return;
                    }
                    child = widget.next_sibling();
                }
            }
        ));

        let weak = Rc::downgrade(self);
        listbox.connect_row_activated(glib::clone!(
            #[weak]
            popover,
            move |_, row| {
                popover.popdown();
                let table = usize::try_from(row.index())
                    .ok()
                    .and_then(|i| tables.get(i));
                if let (Some(this), Some(table)) = (weak.upgrade(), table) {
                    // The click is in zoomed canvas coordinates.
                    let z = this.zoom.get();
                    this.add_table_card(table, x / z, y / z);
                }
            }
        ));

        let list_scroll = gtk::ScrolledWindow::new();
        list_scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
        list_scroll.set_max_content_height(280);
        list_scroll.set_propagate_natural_height(true);
        list_scroll.set_min_content_width(240);
        list_scroll.set_propagate_natural_width(true);
        list_scroll.set_child(Some(&listbox));
        vbox.append(&search);
        vbox.append(&list_scroll);
        popover.set_child(Some(&vbox));
        popover.popup();
        search.grab_focus();
    }

    // --- Join editor (click a join line to change its columns) ---

    fn show_join_editor(self: &Rc<Self>, join: Rc<Join>, x: f64, y: f64) {
        let popover = popover_at(&self.fixed, x, y);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);
        vbox.append(&heading("Edit join"));

        let column_picker = |side: &Side| {
            let names: Vec<&str> = side.card.columns.iter().map(|c| c.name.as_str()).collect();
            let lbl = gtk::Label::builder()
                .label(&side.card.table)
                .xalign(0.0)
                .build();
            lbl.add_css_class("dim-label");
            lbl.add_css_class("caption");
            let dropdown = gtk::DropDown::from_strings(&names);
            if let Some(i) = names.iter().position(|n| *n == side.col) {
                dropdown.set_selected(i as u32);
            }
            vbox.append(&lbl);
            vbox.append(&dropdown);
            dropdown
        };
        let left_dropdown = column_picker(&join.left);
        let right_dropdown = column_picker(&join.right);

        let btn_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        btn_row.set_margin_top(4);
        let remove_btn = gtk::Button::with_label("Remove join");
        remove_btn.add_css_class("destructive-action");
        remove_btn.set_hexpand(true);
        let apply_btn = gtk::Button::with_label("Apply");
        apply_btn.add_css_class("suggested-action");
        apply_btn.set_hexpand(true);
        btn_row.append(&remove_btn);
        btn_row.append(&apply_btn);
        vbox.append(&btn_row);
        popover.set_child(Some(&vbox));

        let weak = Rc::downgrade(self);
        let weak_join = Rc::downgrade(&join);
        remove_btn.connect_clicked(glib::clone!(
            #[weak]
            popover,
            #[strong]
            weak,
            #[strong]
            weak_join,
            move |_| {
                popover.popdown();
                if let (Some(this), Some(join)) = (weak.upgrade(), weak_join.upgrade()) {
                    this.remove_join(&join);
                }
            }
        ));

        apply_btn.connect_clicked(glib::clone!(
            #[weak]
            popover,
            move |_| {
                popover.popdown();
                let (Some(this), Some(join)) = (weak.upgrade(), weak_join.upgrade()) else {
                    return;
                };
                let pick = |side: &Side, dropdown: &gtk::DropDown| {
                    side.card
                        .columns
                        .get(dropdown.selected() as usize)
                        .map(|c| (c.name.clone(), c.anchor.clone()))
                };
                let (Some((new_left, left_anchor)), Some((new_right, right_anchor))) = (
                    pick(&join.left, &left_dropdown),
                    pick(&join.right, &right_dropdown),
                ) else {
                    return;
                };
                if new_left == join.left.col && new_right == join.right.col {
                    return;
                }
                this.remove_join(&join);
                left_anchor.set_active(true);
                right_anchor.set_active(true);
            }
        ));

        popover.popup();
    }

    // --- Query building / execution ---

    /// Snapshot of the canvas as the domain's query model.
    fn build_query(&self) -> DomainResult<String> {
        let cards = self.table_cards.borrow();
        let index_of = |card: &Rc<TableCard>| cards.iter().position(|c| Rc::ptr_eq(c, card));
        let tables: Vec<CanvasTable> = cards
            .iter()
            .map(|card| CanvasTable {
                table: card.table.clone(),
                selected_columns: card
                    .columns
                    .iter()
                    .filter(|c| c.check.is_active())
                    .map(|c| c.name.clone())
                    .collect(),
            })
            .collect();
        let joins: Vec<CanvasJoin> = self
            .joins
            .borrow()
            .iter()
            .filter_map(|j| {
                Some(CanvasJoin {
                    left_table: index_of(&j.left.card)?,
                    left_column: j.left.col.clone(),
                    right_table: index_of(&j.right.card)?,
                    right_column: j.right.col.clone(),
                })
            })
            .collect();
        let criteria = self.criteria_snapshot(&cards);
        let kind = self.db.borrow().kind().unwrap_or(DbKind::Sqlite);
        logic::build_select(kind, &tables, &joins, &criteria, self.limit.get())
            .map(|sql| sql_format::format(&sql))
    }

    fn sql_text(&self) -> String {
        let (start, end) = self.sql_buffer.bounds();
        self.sql_buffer.text(&start, &end, true).to_string()
    }

    /// Writes generated SQL to the editor; later canvas edits keep it in sync.
    fn set_generated_sql(&self, sql: &str) {
        self.quietly(|| self.sql_buffer.set_text(sql));
        *self.generated_sql.borrow_mut() = sql.to_string();
    }

    /// Rewrites the query from the canvas, unless the user has edited it.
    fn canvas_changed(&self) {
        if self.syncing.get() {
            return;
        }
        let current = self.sql_text();
        if !current.trim().is_empty() && current != *self.generated_sql.borrow() {
            return self.set_status(
                "The SQL query was edited by hand, so the designer doesn't rewrite it — \
                 use Rebuild to apply its changes.",
            );
        }
        match self.build_query() {
            Ok(sql) => self.set_generated_sql(&sql),
            Err(_) if self.table_cards.borrow().is_empty() => self.set_generated_sql(""),
            // Incomplete canvas (e.g. a table not linked yet): keep the last query.
            Err(_) => {}
        }
    }

    fn do_generate_sql(&self) {
        match self.build_query() {
            Ok(sql) => {
                self.set_generated_sql(&sql);
                self.set_status("SQL generated from the canvas — edit it, then Run Query.");
            }
            Err(e) => self.set_error(e.message()),
        }
    }

    // --- Query → canvas ---

    fn on_sql_edited(self: &Rc<Self>) {
        if self.syncing.get() {
            return;
        }
        let serial = self.sql_edit_serial.get().wrapping_add(1);
        self.sql_edit_serial.set(serial);
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(SQL_SYNC_DELAY, move || {
            if let Some(this) = weak.upgrade()
                && this.sql_edit_serial.get() == serial
            {
                this.draw_sql_on_canvas();
            }
        });
    }

    /// Redraws the canvas from the typed query when it is a plain SELECT on
    /// known tables; anything else leaves the canvas alone.
    fn draw_sql_on_canvas(self: &Rc<Self>) {
        let Some(kind) = self.db.borrow().kind() else {
            return;
        };
        let sql = self.sql_text();
        if sql.trim().is_empty() || sql == *self.generated_sql.borrow() {
            return;
        }
        let Ok(parsed) = logic::parse_select(kind, &sql) else {
            return;
        };
        let known: Vec<Option<String>> = {
            let cache = self.tables_cache.borrow();
            parsed
                .tables
                .iter()
                .map(|t| {
                    cache
                        .iter()
                        .find(|c| **c == t.table)
                        .or_else(|| cache.iter().find(|c| c.eq_ignore_ascii_case(&t.table)))
                        .cloned()
                })
                .collect()
        };
        if known.iter().all(Option::is_none) {
            return;
        }
        self.quietly(|| self.apply_parsed_select(&parsed, &known));
        self.refresh_joins_panel();
        self.link_lines.queue_draw();
        self.update_canvas_placeholder();

        let unknown: Vec<&str> = parsed
            .tables
            .iter()
            .zip(&known)
            .filter(|(_, k)| k.is_none())
            .map(|(t, _)| t.table.as_str())
            .collect();
        if unknown.is_empty() && parsed.complete {
            // The designer holds the whole query: its edits may rewrite it.
            *self.generated_sql.borrow_mut() = sql;
            self.limit.set(parsed.limit.unwrap_or(MAX_RESULT_ROWS));
            self.set_status("Canvas and criteria redrawn from the SQL query.");
        } else if unknown.is_empty() {
            self.set_status(
                "Canvas redrawn from the SQL query. It has clauses the designer can't show, \
                 so canvas edits won't rewrite it — use Rebuild to replace it.",
            );
        } else {
            self.set_error(&format!(
                "Unknown table(s) in the query: {}",
                unknown.join(", ")
            ));
        }
    }

    /// `known[i]` is the database's name for `parsed.tables[i]`, if it exists.
    fn apply_parsed_select(self: &Rc<Self>, parsed: &ParsedSelect, known: &[Option<String>]) {
        // The n-th occurrence of a table in the query is its n-th card, so a
        // self-join keeps one card per side; cards beyond the query's count go.
        let occurrences = |name: &str| known.iter().flatten().filter(|k| *k == name).count();
        let stale: Vec<Rc<TableCard>> = {
            let mut seen: HashMap<String, usize> = HashMap::new();
            self.table_cards
                .borrow()
                .iter()
                .filter(|c| {
                    let n = seen.entry(c.table.clone()).or_default();
                    *n += 1;
                    *n > occurrences(&c.table)
                })
                .cloned()
                .collect()
        };
        for card in &stale {
            self.remove_table_card(card);
        }
        for join in self.joins.borrow_mut().drain(..) {
            join.left.anchor.set_active(false);
            join.right.anchor.set_active(false);
        }
        self.pending_link.borrow_mut().take();

        let mut used: HashMap<String, usize> = HashMap::new();
        let cards: Vec<Option<Rc<TableCard>>> = known
            .iter()
            .map(|name| {
                let name = name.as_ref()?;
                let nth = used.entry(name.clone()).or_default();
                let existing = self
                    .table_cards
                    .borrow()
                    .iter()
                    .filter(|c| c.table == *name)
                    .nth(*nth)
                    .cloned();
                *nth += 1;
                existing.or_else(|| {
                    let (x, y) = self.next_card_position();
                    self.place_table_card(name, x, y)
                })
            })
            .collect();

        for (table, card) in parsed.tables.iter().zip(&cards) {
            let Some(card) = card else { continue };
            for col in &card.columns {
                let selected = match &table.columns {
                    None => true,
                    Some(names) => names
                        .iter()
                        .chain(&parsed.unqualified_columns)
                        .any(|n| n.eq_ignore_ascii_case(&col.name)),
                };
                col.check.set_active(selected);
            }
        }
        for j in &parsed.joins {
            if let (Some(Some(l)), Some(Some(r))) =
                (cards.get(j.left_table), cards.get(j.right_table))
            {
                self.link_columns(l, &j.left_column, r, &j.right_column);
            }
        }
        self.apply_criteria(&cards, &parsed.criteria);
    }

    fn do_run_query(&self) {
        let mut sql = self.sql_text().trim().to_string();
        if sql.is_empty() {
            // Nothing typed yet — build it from the canvas as a convenience.
            match self.build_query() {
                Ok(built) => {
                    self.set_generated_sql(&built);
                    sql = built;
                }
                Err(e) => return self.set_error(e.message()),
            }
        }
        if let Some(kind) = self.db.borrow().kind()
            && let Err(e) = logic::check_syntax(kind, &sql)
        {
            return self.set_error(e.message());
        }
        let result = self.db.borrow_mut().execute(&sql);
        let (columns, rows) = match result {
            Ok(r) => r,
            Err(e) => return self.set_error(e.message()),
        };
        let truncated = if logic::is_truncated(rows.len()) {
            " (truncated)"
        } else {
            ""
        };
        let status = format!("{} row(s){truncated} — {}", rows.len(), timestamp());
        self.render_results((columns, rows));
        self.set_status(&status);
        self.show_results();
    }

    fn show_results(&self) {
        self.results_toggle.set_active(true);
    }

    fn render_results(&self, (columns, rows): QueryResult) {
        self.export_csv_btn.set_sensitive(!columns.is_empty());
        self.results_lbl.set_text(&match rows.len() {
            _ if columns.is_empty() => "Results".to_string(),
            1 => "Results · 1 row".to_string(),
            n => format!("Results · {n} rows"),
        });
        if columns.is_empty() {
            self.results_scroll
                .set_child(Some(&results_placeholder("Query executed — no rows.")));
        } else {
            let grid = gtk::Grid::new();
            for (c, name) in columns.iter().enumerate() {
                let lbl = gtk::Label::builder().label(name).xalign(0.0).build();
                lbl.add_css_class("dt-grid-cell");
                lbl.add_css_class("dt-grid-header");
                grid.attach(&lbl, c as i32, 0, 1, 1);
            }
            for (r, row) in rows.iter().enumerate() {
                for (c, value) in row.iter().enumerate() {
                    let lbl = gtk::Label::builder()
                        .label(value.as_deref().unwrap_or("NULL"))
                        .xalign(0.0)
                        .selectable(true)
                        .max_width_chars(30)
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();
                    lbl.add_css_class("dt-grid-cell");
                    if value.is_none() {
                        lbl.add_css_class("dt-null");
                    }
                    if r % 2 == 1 {
                        lbl.add_css_class("dt-grid-odd");
                    }
                    grid.attach(&lbl, c as i32, r as i32 + 1, 1, 1);
                }
            }
            self.results_scroll.set_child(Some(&grid));
        }
        *self.last_result.borrow_mut() = (columns, rows);
    }

    fn do_export_csv(self: &Rc<Self>) {
        if self.last_result.borrow().0.is_empty() {
            return self.set_error("Run a query first.");
        }
        let dialog = gtk::FileDialog::builder()
            .initial_name("query_results.csv")
            .build();
        let window = self.export_csv_btn.root().and_downcast::<gtk::Window>();
        let weak = Rc::downgrade(self);
        dialog.save(window.as_ref(), gio::Cancellable::NONE, move |res| {
            let (Ok(file), Some(this)) = (res, weak.upgrade()) else {
                return;
            };
            let Some(path) = file.path() else { return };
            match this.write_csv(&path) {
                Ok(n) => this.set_status(&format!("Exported {n} row(s) to {}", path.display())),
                Err(e) => this.set_error(e.message()),
            }
        });
    }

    fn write_csv(&self, path: &std::path::Path) -> DomainResult<usize> {
        let (columns, rows) = &*self.last_result.borrow();
        std::fs::write(path, logic::to_csv(columns, rows)?)?;
        Ok(rows.len())
    }

    // --- Connections ---

    fn load_connections(&self) {
        if let Some(path) = self.store.last_path() {
            self.open_connections_file(&path);
        }
    }

    fn file_name(path: &Path) -> String {
        path.file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .into_owned()
    }

    fn open_connections_file(&self, path: &Path) {
        let connections = match self.store.load(path) {
            Ok(connections) => connections,
            Err(e) => {
                return self.set_error(&format!(
                    "Cannot open “{}”: {}",
                    path.display(),
                    e.message()
                ));
            }
        };
        let names: Vec<&str> = connections.iter().map(|c| c.name.as_str()).collect();
        self.conn_model.splice(0, self.conn_model.n_items(), &names);
        self.connect_btn.set_sensitive(!connections.is_empty());
        self.set_status(&format!(
            "Opened “{}” — {} connection(s).",
            Self::file_name(path),
            connections.len()
        ));
        *self.connections.borrow_mut() = connections;
        self.update_connection_state();
        *self.connections_file.borrow_mut() = Some(path.to_path_buf());
        self.unsaved.set(false);
        let _ = self.store.set_last_path(path);
    }

    /// True when the connections were written.
    fn save_connections_file(&self, path: &Path) -> bool {
        if let Err(e) = self.store.save(path, &self.connections.borrow()) {
            self.set_error(&format!(
                "Cannot save “{}”: {}",
                path.display(),
                e.message()
            ));
            return false;
        }
        self.set_status(&format!(
            "Connections saved to “{}”.",
            Self::file_name(path)
        ));
        *self.connections_file.borrow_mut() = Some(path.to_path_buf());
        self.unsaved.set(false);
        let _ = self.store.set_last_path(path);
        true
    }

    fn json_file_dialog(&self, title: &str) -> gtk::FileDialog {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("JSON files"));
        filter.add_pattern("*.json");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder()
            .title(title)
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();
        match self.connections_file.borrow().as_deref() {
            Some(path) => dialog.set_initial_file(Some(&gio::File::for_path(path))),
            None => dialog.set_initial_name(Some("connections.json")),
        }
        dialog
    }

    fn window(&self) -> Option<gtk::Window> {
        self.connect_btn.root().and_downcast::<gtk::Window>()
    }

    fn do_open_connections(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        Rc::clone(self).confirm_leave(Box::new(move || {
            let Some(this) = weak.upgrade() else { return };
            let weak = weak.clone();
            this.json_file_dialog("Open Connections").open(
                this.window().as_ref(),
                gio::Cancellable::NONE,
                move |res| {
                    if let (Some(this), Some(path)) =
                        (weak.upgrade(), res.ok().and_then(|f| f.path()))
                    {
                        this.open_connections_file(&path);
                    }
                },
            );
        }));
    }

    fn do_save_connections(self: &Rc<Self>) {
        let current = self.connections_file.borrow().clone();
        match current {
            Some(path) => {
                self.save_connections_file(&path);
            }
            None => self.ask_where_to_save(Box::new(|| {})),
        }
    }

    /// Asks for a file, saves to it, then calls `then`. Cancelling skips both.
    fn ask_where_to_save(self: &Rc<Self>, then: Box<dyn FnOnce()>) {
        let weak = Rc::downgrade(self);
        self.json_file_dialog("Save Connections").save(
            self.window().as_ref(),
            gio::Cancellable::NONE,
            move |res| {
                if let (Some(this), Some(path)) = (weak.upgrade(), res.ok().and_then(|f| f.path()))
                    && this.save_connections_file(&path)
                {
                    then();
                }
            },
        );
    }

    fn update_connection_state(&self) {
        let connected = self.db.borrow().is_connected();
        let lbl = &self.conn_state_lbl;
        if connected {
            lbl.set_text("● Connected");
            lbl.remove_css_class("dim-label");
            lbl.add_css_class("success");
            self.connect_btn.set_label("Disconnect");
            self.connect_btn.remove_css_class("suggested-action");
        } else {
            lbl.set_text("○ Not connected");
            lbl.remove_css_class("success");
            lbl.add_css_class("dim-label");
            self.connect_btn.set_label("Connect");
            self.connect_btn.add_css_class("suggested-action");
        }
        // Switching connection while connected would leave the canvas stale.
        self.conn_dropdown
            .set_sensitive(!connected && self.conn_model.n_items() > 0);
        // On (dis)connecting only: connected, the bar folds away behind
        // `conn_toggle`; disconnected, it is all there is to use.
        if self.workspace.is_visible() != connected {
            self.conn_revealer.set_reveal_child(!connected);
        }
        self.workspace.set_visible(connected);
        self.disconnected_page.set_visible(!connected);
        let name = self
            .conn_model
            .string(self.conn_dropdown.selected())
            .filter(|_| connected);
        self.conn_toggle.set_tooltip_text(Some(&match name {
            Some(name) => format!("Connection “{name}” — show or hide"),
            None => "Show or hide the connection".to_string(),
        }));
    }

    fn do_connect(self: &Rc<Self>) {
        if self.db.borrow().is_connected() {
            self.db.borrow_mut().close();
            self.tables_cache.borrow_mut().clear();
            self.update_connection_state();
            return self.set_status("Disconnected.");
        }
        let idx = self.conn_dropdown.selected() as usize;
        let Some(needs_password) = self
            .connections
            .borrow()
            .get(idx)
            .map(ConnectionConfig::needs_password)
        else {
            return self.set_error("Create a connection first.");
        };
        if needs_password {
            self.ask_password(idx);
        } else {
            self.connect_to(idx);
        }
    }

    fn connect_to(&self, idx: usize) {
        let mut connections = self.connections.borrow_mut();
        let cfg = &mut connections[idx];
        let mut db = self.db.borrow_mut();
        let tables = db.connect(cfg).and_then(|_| db.list_tables());
        drop(db);
        match tables {
            Ok(tables) => {
                let n = tables.len();
                *self.tables_cache.borrow_mut() = tables;
                self.set_status(&format!(
                    "Connected to “{}” — {n} table(s) found. Right-click the canvas to add one.",
                    cfg.name
                ));
            }
            Err(e) => {
                // Ask again next time, in case the password was wrong.
                cfg.forget_password();
                self.db.borrow_mut().close();
                self.set_error(e.message());
            }
        }
        drop(connections);
        self.update_connection_state();
    }

    fn ask_password(self: &Rc<Self>, idx: usize) {
        let name = self.connections.borrow()[idx].name.clone();
        let dialog = adw::AlertDialog::new(
            Some("Password required"),
            Some(&format!("Enter the password for “{name}”.")),
        );
        dialog.add_responses(&[("cancel", "Cancel"), ("connect", "Connect")]);
        dialog.set_response_appearance("connect", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("connect"));
        dialog.set_close_response("cancel");

        let entry = gtk::PasswordEntry::builder()
            .show_peek_icon(true)
            .activates_default(true)
            .build();
        dialog.set_extra_child(Some(&entry));

        let weak = Rc::downgrade(self);
        dialog.connect_response(
            Some("connect"),
            glib::clone!(
                #[weak]
                entry,
                move |_, _| {
                    let Some(this) = weak.upgrade() else { return };
                    this.connections.borrow_mut()[idx].set_password(entry.text().into());
                    this.connect_to(idx);
                }
            ),
        );
        dialog.present(Some(&self.connect_btn));
        dialog.set_focus(Some(&entry));
    }

    fn open_new_connection_dialog(self: &Rc<Self>) {
        let dialog = adw::Dialog::new();
        dialog.set_title("New Database Connection");
        dialog.set_content_width(420);

        let toolbar_view = adw::ToolbarView::new();
        let header_bar = adw::HeaderBar::new();
        header_bar.set_show_end_title_buttons(false);
        let cancel_btn = gtk::Button::with_label("Cancel");
        let create_btn = gtk::Button::with_label("Create");
        create_btn.add_css_class("suggested-action");
        header_bar.pack_start(&cancel_btn);
        header_bar.pack_end(&create_btn);
        toolbar_view.add_top_bar(&header_bar);

        let page = adw::PreferencesPage::new();

        let general_group = adw::PreferencesGroup::new();
        let name_row = adw::EntryRow::builder().title("Connection name").build();
        let labels: Vec<&str> = DbKind::ALL.iter().map(|k| k.label()).collect();
        let type_row = adw::ComboRow::builder()
            .title("Type")
            .model(&gtk::StringList::new(&labels))
            .build();
        general_group.add(&name_row);
        general_group.add(&type_row);
        page.add(&general_group);

        let sqlite_group = adw::PreferencesGroup::builder().title("SQLite").build();
        let path_row = adw::EntryRow::builder().title("Database file").build();
        let browse_btn = gtk::Button::from_icon_name("document-open-symbolic");
        browse_btn.add_css_class("flat");
        browse_btn.set_valign(gtk::Align::Center);
        path_row.add_suffix(&browse_btn);
        sqlite_group.add(&path_row);
        page.add(&sqlite_group);

        let server_group = adw::PreferencesGroup::builder().title("Server").build();
        let host_row = adw::EntryRow::builder()
            .title("Host")
            .text("localhost")
            .build();
        let port_row = adw::EntryRow::builder().title("Port").build();
        let database_row = adw::EntryRow::builder().title("Database").build();
        let user_row = adw::EntryRow::builder().title("User").build();
        let password_row = adw::PasswordEntryRow::builder().title("Password").build();
        server_group.add(&host_row);
        server_group.add(&port_row);
        server_group.add(&database_row);
        server_group.add(&user_row);
        server_group.add(&password_row);
        page.add(&server_group);

        let selected_kind = |row: &adw::ComboRow| DbKind::ALL[row.selected() as usize];

        let update_visibility = glib::clone!(
            #[weak]
            sqlite_group,
            #[weak]
            server_group,
            #[weak]
            port_row,
            move |row: &adw::ComboRow| {
                let kind = selected_kind(row);
                let is_sqlite = kind == DbKind::Sqlite;
                sqlite_group.set_visible(is_sqlite);
                server_group.set_visible(!is_sqlite);
                if !is_sqlite && port_row.text().is_empty() {
                    port_row.set_text(kind.default_port());
                }
            }
        );
        update_visibility(&type_row);
        type_row.connect_selected_notify(update_visibility);

        toolbar_view.set_content(Some(&page));
        dialog.set_child(Some(&toolbar_view));

        browse_btn.connect_clicked(glib::clone!(
            #[weak]
            path_row,
            move |btn| {
                let window = btn.root().and_downcast::<gtk::Window>();
                gtk::FileDialog::new().open(window.as_ref(), gio::Cancellable::NONE, move |res| {
                    if let Some(path) = res.ok().and_then(|f| f.path()) {
                        path_row.set_text(&path.to_string_lossy());
                    }
                });
            }
        ));

        let weak = Rc::downgrade(self);
        create_btn.connect_clicked(glib::clone!(
            #[weak]
            dialog,
            #[weak]
            type_row,
            #[weak]
            name_row,
            #[weak]
            path_row,
            #[weak]
            host_row,
            #[weak]
            port_row,
            #[weak]
            database_row,
            #[weak]
            user_row,
            #[weak]
            password_row,
            move |_| {
                let Some(this) = weak.upgrade() else { return };
                let kind = selected_kind(&type_row);
                let form = ConnectionForm {
                    name: name_row.text().into(),
                    path: path_row.text().into(),
                    host: host_row.text().into(),
                    port: port_row.text().into(),
                    database: database_row.text().into(),
                    user: user_row.text().into(),
                    password: password_row.text().into(),
                };
                let existing = this.connections.borrow().len();
                let config = match form.into_config(kind, existing) {
                    Ok(config) => config,
                    Err(_) => {
                        // The only validation rule: SQLite needs a file path.
                        path_row.add_css_class("error");
                        return;
                    }
                };
                let name = config.name.clone();

                this.conn_model.append(&name);
                this.set_status(&format!(
                    "Connection “{name}” created. Click Connect to use it."
                ));
                this.connections.borrow_mut().push(config);
                this.unsaved.set(true);
                this.conn_dropdown
                    .set_selected(this.conn_model.n_items() - 1);
                this.connect_btn.set_sensitive(true);
                this.update_connection_state();
                dialog.close();
            }
        ));
        cancel_btn.connect_clicked(glib::clone!(
            #[weak]
            dialog,
            move |_| {
                dialog.close();
            }
        ));

        dialog.present(Some(&self.new_conn_btn));
    }
}

impl LeaveGuard for SqlViewer {
    fn confirm_leave(self: Rc<Self>, proceed: Box<dyn FnOnce()>) {
        if !self.unsaved.get() {
            return proceed();
        }
        let dialog = adw::AlertDialog::new(
            Some("Save connections?"),
            Some("The connection list has unsaved changes. Passwords are never saved."),
        );
        dialog.add_responses(&[
            ("cancel", "Cancel"),
            ("discard", "Discard"),
            ("save", "Save…"),
        ]);
        dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
        dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some("save"));
        dialog.set_close_response("cancel");

        let weak = Rc::downgrade(&self);
        let proceed = RefCell::new(Some(proceed));
        dialog.connect_response(None, move |_, response| {
            let (Some(this), Some(proceed)) = (weak.upgrade(), proceed.take()) else {
                return;
            };
            match response {
                "save" => this.ask_where_to_save(proceed),
                "discard" => proceed(),
                _ => {}
            }
        });
        dialog.present(Some(&self.connect_btn));
    }
}
