use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gio, glib, graphene};

use crate::domain::contract::FeatureContract;
use crate::infrastructure::api::database_api::{self, ConnectionParams, DatabaseApi, DbKind};
use crate::infrastructure::ui::shared::input_output_widget::timestamp;

pub const DESCRIPTION: &str = "Browse database tables on an interactive canvas";

const MAX_RESULT_ROWS: usize = 200;
const LINK_COLOR: (f64, f64, f64) = (0.22, 0.51, 0.89);

pub struct Feature;

impl FeatureContract for Feature {
    fn full_width(&self) -> bool {
        true
    }

    fn widget(&self) -> gtk::Widget {
        SqlViewer::build()
    }
}

struct ConnectionConfig {
    name: String,
    kind: DbKind,
    params: ConnectionParams,
}

struct ColumnWidgets {
    name: String,
    check: gtk::CheckButton,
    anchor: gtk::ToggleButton,
}

struct TableCard {
    table: String,
    frame: gtk::Frame,
    columns: Vec<ColumnWidgets>,
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
    db: RefCell<DatabaseApi>,
    connections: RefCell<Vec<ConnectionConfig>>,
    tables_cache: RefCell<Vec<String>>,
    table_cards: RefCell<Vec<Rc<TableCard>>>,
    joins: RefCell<Vec<Rc<Join>>>,
    pending_link: RefCell<Option<Side>>,
    link_hitboxes: RefCell<Vec<Hitbox>>,
    last_result: RefCell<database_api::QueryResult>,

    conn_model: gtk::StringList,
    conn_dropdown: gtk::DropDown,
    new_conn_btn: gtk::Button,
    status_lbl: gtk::Label,
    fixed: gtk::Fixed,
    link_lines: gtk::DrawingArea,
    joins_box: gtk::Box,
    sql_buffer: gtk::TextBuffer,
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

fn bezier_point(p: &[(f64, f64); 4], t: f64) -> (f64, f64) {
    let mt = 1.0 - t;
    let (a, b, c, d) = (
        mt.powi(3),
        3.0 * mt.powi(2) * t,
        3.0 * mt * t.powi(2),
        t.powi(3),
    );
    (
        a * p[0].0 + b * p[1].0 + c * p[2].0 + d * p[3].0,
        a * p[0].1 + b * p[1].1 + c * p[2].1 + d * p[3].1,
    )
}

/// Connects at the vertical height of the linked column, but always on the
/// frame edge that faces the other table — so the line hugs the outside of
/// both cards instead of cutting across whichever one sits in between.
fn edge_point(
    target: &gtk::DrawingArea,
    anchor: &gtk::ToggleButton,
    frame: &gtk::Frame,
    facing: &gtk::Frame,
) -> Option<(f64, f64)> {
    let a = anchor.compute_point(
        target,
        &graphene::Point::new(anchor.width() as f32 / 2.0, anchor.height() as f32 / 2.0),
    )?;
    let f = frame.compute_point(target, &graphene::Point::zero())?;
    let ff = facing.compute_point(target, &graphene::Point::zero())?;
    let own_center = f.x() + frame.width() as f32 / 2.0;
    let facing_center = ff.x() + facing.width() as f32 / 2.0;
    let edge_x = if own_center <= facing_center {
        f.x() + frame.width() as f32
    } else {
        f.x()
    };
    Some((edge_x as f64, a.y() as f64))
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
    fn build() -> gtk::Widget {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);

        // --- Connection bar ---
        let conn_model = gtk::StringList::new(&[]);
        let conn_dropdown = gtk::DropDown::new(Some(conn_model.clone()), gtk::Expression::NONE);
        conn_dropdown.set_hexpand(true);
        let new_conn_btn = gtk::Button::with_label("New Connection…");
        let connect_btn = gtk::Button::with_label("Connect");
        connect_btn.add_css_class("suggested-action");

        let conn_bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        conn_bar.append(&conn_dropdown);
        conn_bar.append(&new_conn_btn);
        conn_bar.append(&connect_btn);

        let status_lbl = gtk::Label::new(None);
        status_lbl.add_css_class("dim-label");
        status_lbl.set_halign(gtk::Align::Start);
        status_lbl.set_wrap(true);

        // --- Canvas ---
        let fixed = gtk::Fixed::new();
        fixed.set_size_request(2000, 1200);
        fixed.add_css_class("view");

        let link_lines = gtk::DrawingArea::new();
        link_lines.set_can_target(false);

        let canvas_overlay = gtk::Overlay::new();
        canvas_overlay.set_child(Some(&fixed));
        canvas_overlay.add_overlay(&link_lines);

        let canvas_scroll = gtk::ScrolledWindow::new();
        canvas_scroll.add_css_class("card");
        canvas_scroll.set_size_request(-1, 420);
        canvas_scroll.set_vexpand(true);
        canvas_scroll.set_hexpand(true);
        canvas_scroll.set_child(Some(&canvas_overlay));

        let canvas_panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
        canvas_panel.append(&canvas_scroll);
        canvas_panel.append(&canvas_resize_handle(&canvas_scroll));

        // --- Joins panel ---
        let joins_box = gtk::Box::new(gtk::Orientation::Vertical, 4);

        // --- SQL editor ---
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
        sql_scroll.set_min_content_height(110);
        sql_scroll.set_max_content_height(260);
        sql_scroll.set_propagate_natural_height(true);
        sql_scroll.set_hexpand(true);
        sql_scroll.set_child(Some(&sql_view));

        let generate_sql_btn = gtk::Button::with_label("Generate from canvas");
        let run_btn = gtk::Button::with_label("Run Query");
        run_btn.add_css_class("suggested-action");
        let sql_actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        sql_actions.append(&generate_sql_btn);
        sql_actions.append(&run_btn);

        // --- Results ---
        let results_lbl = heading("Results");
        results_lbl.set_hexpand(true);
        let export_csv_btn = gtk::Button::with_label("Export CSV…");
        export_csv_btn.set_sensitive(false);
        let results_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        results_header.append(&results_lbl);
        results_header.append(&export_csv_btn);

        let results_scroll = gtk::ScrolledWindow::new();
        results_scroll.add_css_class("card");
        results_scroll.set_min_content_height(160);
        results_scroll.set_vexpand(true);
        results_scroll.set_hexpand(true);
        results_scroll.set_child(Some(&results_placeholder(
            "Run the query to see results here.",
        )));

        root.append(&heading("Connection"));
        root.append(&conn_bar);
        root.append(&status_lbl);
        root.append(&heading("Schema canvas"));
        root.append(&hint(
            "Right-click the canvas to add a connected table. \
             Click the link icon on two columns to join them. \
             Click a join line to change its columns.",
        ));
        root.append(&canvas_panel);
        root.append(&heading("Joins"));
        root.append(&joins_box);
        root.append(&heading("SQL query"));
        root.append(&hint(
            "Built from the canvas — edit freely to add a WHERE, GROUP BY, ORDER BY…, then Run Query.",
        ));
        root.append(&sql_scroll);
        root.append(&sql_actions);
        root.append(&results_header);
        root.append(&results_scroll);

        let this = Rc::new(Self {
            db: RefCell::default(),
            connections: RefCell::default(),
            tables_cache: RefCell::default(),
            table_cards: RefCell::default(),
            joins: RefCell::default(),
            pending_link: RefCell::default(),
            link_hitboxes: RefCell::default(),
            last_result: RefCell::default(),
            conn_model,
            conn_dropdown,
            new_conn_btn,
            status_lbl,
            fixed,
            link_lines,
            joins_box,
            sql_buffer,
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
        generate_sql_btn.connect_clicked(with_viewer!(weak, |this| this.do_generate_sql()));
        run_btn.connect_clicked(with_viewer!(weak, |this| this.do_run_query()));
        this.export_csv_btn
            .connect_clicked(with_viewer!(weak, |this| this.do_export_csv()));

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

        root.connect_destroy(move |_| {
            let _ = &this;
        });
        root.upcast()
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
            let p1 = edge_point(&self.link_lines, &l.anchor, &l.card.frame, &rt.card.frame);
            let p2 = edge_point(&self.link_lines, &rt.anchor, &rt.card.frame, &l.card.frame);
            let (Some((x1, y1)), Some((x2, y2))) = (p1, p2) else {
                continue;
            };
            let mid = (x1 + x2) / 2.0;
            cr.set_source_rgba(r, g, b, 0.9);
            cr.set_line_width(2.0);
            cr.move_to(x1, y1);
            cr.curve_to(mid, y1, mid, y2, x2, y2);
            let _ = cr.stroke();
            for (px, py) in [(x1, y1), (x2, y2)] {
                cr.arc(px, py, 3.5, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            }
            hitboxes.push(Hitbox {
                join: Rc::clone(join),
                points: [(x1, y1), (mid, y1), (mid, y2), (x2, y2)],
            });
        }
    }

    fn find_join_at(&self, x: f64, y: f64, tolerance: f64) -> Option<Rc<Join>> {
        let mut best = None;
        let mut best_dist = tolerance;
        for hb in self.link_hitboxes.borrow().iter() {
            for i in 0..=20 {
                let (px, py) = bezier_point(&hb.points, i as f64 / 20.0);
                let dist = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
                if dist < best_dist {
                    best_dist = dist;
                    best = Some(Rc::clone(&hb.join));
                }
            }
        }
        best
    }

    // --- Joins panel ---

    fn refresh_joins_panel(self: &Rc<Self>) {
        while let Some(child) = self.joins_box.first_child() {
            self.joins_box.remove(&child);
        }
        let joins = self.joins.borrow();
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
    }

    fn remove_join(self: &Rc<Self>, join: &Rc<Join>) {
        self.joins.borrow_mut().retain(|j| !Rc::ptr_eq(j, join));
        join.left.anchor.set_active(false);
        join.right.anchor.set_active(false);
        self.joins_changed();
    }

    // --- Join anchors ---

    fn on_anchor_toggled(self: &Rc<Self>, side: Side) {
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
    ) -> Rc<TableCard> {
        let frame = gtk::Frame::new(None);
        frame.add_css_class("card");
        frame.set_size_request(240, 160);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);

        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let name_lbl = heading(table);
        name_lbl.set_hexpand(true);
        let close_btn = close_button();
        header.append(&name_lbl);
        header.append(&close_btn);
        header.set_cursor_from_name(Some("move"));

        let fields_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        fields_box.set_margin_end(10);
        let mut column_widgets = Vec::new();
        for (col_name, col_type) in columns {
            let label = if col_type.is_empty() {
                col_name.clone()
            } else {
                format!("{col_name}  ·  {col_type}")
            };
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            let check = gtk::CheckButton::with_label(&label);
            check.set_active(true);
            check.set_hexpand(true);
            let anchor = gtk::ToggleButton::new();
            anchor.set_icon_name("insert-link-symbolic");
            anchor.add_css_class("flat");
            anchor.add_css_class("circular");
            anchor.set_tooltip_text(Some(
                "Link this column to another table's column to join them",
            ));
            row_box.append(&check);
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
        fields_scroll.set_max_content_height(220);
        fields_scroll.set_propagate_natural_height(columns.len() <= 8);
        fields_scroll.set_vexpand(true);
        fields_scroll.set_overlay_scrolling(false);
        fields_scroll.set_child(Some(&fields_box));

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
            columns: column_widgets,
        });
        let weak = Rc::downgrade(self);
        let weak_card = Rc::downgrade(&card);

        for col in &card.columns {
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

        // Drag the header to move the card.
        let start_pos = Rc::new(Cell::new((0.0, 0.0)));
        let drag = gtk::GestureDrag::new();
        drag.connect_drag_begin(glib::clone!(
            #[strong]
            start_pos,
            #[weak]
            frame,
            #[weak(rename_to = fixed)]
            self.fixed,
            move |_, _, _| start_pos.set(fixed.child_position(&frame))
        ));
        drag.connect_drag_update(glib::clone!(
            #[weak]
            frame,
            #[weak(rename_to = fixed)]
            self.fixed,
            #[weak(rename_to = link_lines)]
            self.link_lines,
            move |_, dx, dy| {
                let (x, y) = start_pos.get();
                fixed.move_(&frame, (x + dx).max(0.0), (y + dy).max(0.0));
                link_lines.queue_draw();
            }
        ));
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
    }

    fn add_table_card(self: &Rc<Self>, table: &str, x: f64, y: f64) {
        let columns = match self.db.borrow_mut().list_columns(table) {
            Ok(c) => c,
            Err(e) => return self.set_error(&e),
        };
        let card = self.build_table_card(table, &columns);
        self.fixed.put(&card.frame, x, y);
        self.table_cards.borrow_mut().push(card);
        self.set_status(&format!("Added “{table}” to the canvas."));
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

        let listbox = gtk::ListBox::new();
        listbox.add_css_class("boxed-list");
        let tables = self.tables_cache.borrow().clone();
        for table in &tables {
            let row_lbl = gtk::Label::builder().label(table).xalign(0.0).build();
            row_lbl.set_margin_top(6);
            row_lbl.set_margin_bottom(6);
            row_lbl.set_margin_start(10);
            row_lbl.set_margin_end(10);
            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&row_lbl));
            listbox.append(&row);
        }

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
                    this.add_table_card(table, x, y);
                }
            }
        ));

        let list_scroll = gtk::ScrolledWindow::new();
        list_scroll.set_max_content_height(240);
        list_scroll.set_propagate_natural_height(true);
        list_scroll.set_min_content_width(220);
        list_scroll.set_max_content_width(480);
        list_scroll.set_propagate_natural_width(true);
        list_scroll.set_child(Some(&listbox));
        popover.set_child(Some(&list_scroll));
        popover.popup();
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

    fn quote_ident(&self, name: &str) -> String {
        if self.db.borrow().kind() == Some(DbKind::Mysql) {
            format!("`{}`", name.replace('`', "``"))
        } else {
            format!("\"{}\"", name.replace('"', "\"\""))
        }
    }

    fn build_query(&self) -> Result<String, String> {
        let cards = self.table_cards.borrow();
        if cards.is_empty() {
            return Err("Add at least one table to the canvas first.".into());
        }
        let alias = |card: &Rc<TableCard>| {
            let i = cards.iter().position(|c| Rc::ptr_eq(c, card)).unwrap_or(0);
            format!("t{}", i + 1)
        };

        let mut select_parts = Vec::new();
        for card in cards.iter() {
            let a = alias(card);
            for col in card.columns.iter().filter(|c| c.check.is_active()) {
                select_parts.push(format!(
                    "{a}.{} AS \"{a}_{}\"",
                    self.quote_ident(&col.name),
                    col.name.replace('"', "\"\"")
                ));
            }
        }
        if select_parts.is_empty() {
            return Err("Check at least one column to select.".into());
        }

        let base = &cards[0];
        let from_clause = format!("{} AS {}", self.quote_ident(&base.table), alias(base));

        let mut joined: HashSet<*const TableCard> = HashSet::from([Rc::as_ptr(base)]);
        let mut remaining: Vec<Rc<Join>> = self.joins.borrow().clone();
        let mut join_clauses = Vec::new();
        let mut progress = true;
        while !remaining.is_empty() && progress {
            progress = false;
            remaining.retain(|join| {
                let has_left = joined.contains(&Rc::as_ptr(&join.left.card));
                let has_right = joined.contains(&Rc::as_ptr(&join.right.card));
                let (old, new) = match (has_left, has_right) {
                    (true, false) => (&join.left, &join.right),
                    (false, true) => (&join.right, &join.left),
                    _ => return true,
                };
                join_clauses.push(format!(
                    "INNER JOIN {} AS {} ON {}.{} = {}.{}",
                    self.quote_ident(&new.card.table),
                    alias(&new.card),
                    alias(&old.card),
                    self.quote_ident(&old.col),
                    alias(&new.card),
                    self.quote_ident(&new.col),
                ));
                joined.insert(Rc::as_ptr(&new.card));
                progress = true;
                false
            });
        }

        let unreached: Vec<&str> = cards
            .iter()
            .filter(|c| !joined.contains(&Rc::as_ptr(c)))
            .map(|c| c.table.as_str())
            .collect();
        if !unreached.is_empty() {
            return Err(format!(
                "Link a column to connect: {}",
                unreached.join(", ")
            ));
        }

        let mut sql = format!("SELECT {} FROM {from_clause}", select_parts.join(", "));
        if !join_clauses.is_empty() {
            sql.push(' ');
            sql.push_str(&join_clauses.join(" "));
        }
        sql.push_str(&format!(" LIMIT {MAX_RESULT_ROWS}"));
        Ok(sql)
    }

    fn do_generate_sql(&self) {
        match self.build_query() {
            Ok(sql) => {
                self.sql_buffer.set_text(&sql);
                self.set_status("SQL generated from the canvas — edit it, then Run Query.");
            }
            Err(e) => self.set_error(&e),
        }
    }

    fn do_run_query(&self) {
        let (start, end) = self.sql_buffer.bounds();
        let mut sql = self.sql_buffer.text(&start, &end, true).trim().to_string();
        if sql.is_empty() {
            // Nothing typed yet — build it from the canvas as a convenience.
            match self.build_query() {
                Ok(built) => {
                    self.sql_buffer.set_text(&built);
                    sql = built;
                }
                Err(e) => return self.set_error(&e),
            }
        }
        let result = self.db.borrow_mut().execute(&sql);
        let (columns, rows) = match result {
            Ok(r) => r,
            Err(e) => return self.set_error(&e),
        };
        let truncated = if rows.len() == MAX_RESULT_ROWS {
            " (truncated)"
        } else {
            ""
        };
        let status = format!("{} row(s){truncated} — {}", rows.len(), timestamp());
        self.render_results(columns, rows);
        self.set_status(&status);
    }

    fn render_results(&self, columns: Vec<String>, rows: Vec<Vec<database_api::Cell>>) {
        self.export_csv_btn.set_sensitive(!columns.is_empty());
        if columns.is_empty() {
            self.results_scroll
                .set_child(Some(&results_placeholder("Query executed — no rows.")));
        } else {
            let grid = gtk::Grid::new();
            grid.set_row_spacing(1);
            grid.set_column_spacing(1);
            for (c, name) in columns.iter().enumerate() {
                let lbl = heading(name);
                lbl.set_margin_top(6);
                lbl.set_margin_bottom(6);
                lbl.set_margin_start(10);
                lbl.set_margin_end(10);
                grid.attach(&lbl, c as i32, 0, 1, 1);
            }
            for (r, row) in rows.iter().enumerate() {
                for (c, value) in row.iter().enumerate() {
                    let lbl = gtk::Label::builder()
                        .label(value.as_deref().unwrap_or(""))
                        .xalign(0.0)
                        .selectable(true)
                        .max_width_chars(30)
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .build();
                    lbl.set_margin_top(4);
                    lbl.set_margin_bottom(4);
                    lbl.set_margin_start(10);
                    lbl.set_margin_end(10);
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
                Err(e) => this.set_error(&e),
            }
        });
    }

    fn write_csv(&self, path: &std::path::Path) -> Result<usize, String> {
        let (columns, rows) = &*self.last_result.borrow();
        let mut writer = csv::Writer::from_path(path).map_err(|e| e.to_string())?;
        writer.write_record(columns).map_err(|e| e.to_string())?;
        for row in rows {
            writer
                .write_record(row.iter().map(|v| v.as_deref().unwrap_or("")))
                .map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())?;
        Ok(rows.len())
    }

    // --- Connections ---

    fn do_connect(&self) {
        let idx = self.conn_dropdown.selected();
        let connections = self.connections.borrow();
        let Some(cfg) = connections.get(idx as usize) else {
            return self.set_error("Create a connection first.");
        };
        let mut db = self.db.borrow_mut();
        let tables = db
            .connect(cfg.kind, &cfg.params)
            .and_then(|_| db.list_tables());
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
            Err(e) => self.set_error(&e),
        }
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
                let mut name = name_row.text().trim().to_string();
                if name.is_empty() {
                    name = format!("connection-{}", this.connections.borrow().len() + 1);
                }

                let mut params = ConnectionParams::new();
                if kind == DbKind::Sqlite {
                    let path = path_row.text().trim().to_string();
                    if path.is_empty() {
                        path_row.add_css_class("error");
                        return;
                    }
                    params.insert("path", path);
                } else {
                    let or = |v: String, d: &str| if v.is_empty() { d.to_string() } else { v };
                    params.insert("host", or(host_row.text().trim().into(), "localhost"));
                    params.insert(
                        "port",
                        or(port_row.text().trim().into(), kind.default_port()),
                    );
                    params.insert("database", database_row.text().trim().into());
                    params.insert("user", user_row.text().trim().into());
                    params.insert("password", password_row.text().into());
                }

                this.conn_model.append(&name);
                this.set_status(&format!(
                    "Connection “{name}” created. Click Connect to use it."
                ));
                this.connections
                    .borrow_mut()
                    .push(ConnectionConfig { name, kind, params });
                this.conn_dropdown
                    .set_selected(this.conn_model.n_items() - 1);
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

fn canvas_resize_handle(canvas_scroll: &gtk::ScrolledWindow) -> gtk::Box {
    let handle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    handle.set_size_request(-1, 10);
    handle.set_halign(gtk::Align::Fill);
    handle.add_css_class("dim-label");
    handle.set_cursor_from_name(Some("ns-resize"));
    handle.set_tooltip_text(Some("Drag to resize the canvas"));
    let grip = gtk::Label::new(Some("⋯"));
    grip.set_hexpand(true);
    grip.add_css_class("dim-label");
    handle.append(&grip);

    let start = Rc::new(Cell::new(0));
    let drag = gtk::GestureDrag::new();
    drag.connect_drag_begin(glib::clone!(
        #[strong]
        start,
        #[weak]
        canvas_scroll,
        move |_, _, _| start.set(canvas_scroll.height())
    ));
    drag.connect_drag_update(glib::clone!(
        #[weak]
        canvas_scroll,
        move |_, _, dy| {
            let new_h = (start.get() as f64 + dy).max(240.0);
            canvas_scroll.set_size_request(-1, new_h as i32);
        }
    ));
    handle.add_controller(drag);
    handle
}
