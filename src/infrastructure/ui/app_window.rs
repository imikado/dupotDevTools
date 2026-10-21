use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gdk_pixbuf, gio, glib};

use crate::infrastructure::service::feature_loader_service::FeatureLoaderService;
use crate::infrastructure::ui::features::RegisteredFeature;

use super::feature_detail_page;

const APP_ID: &str = "org.dupot.devtools";
const APP_NAME: &str = "Dupot Dev Tools";

const APP_CSS: &str = "
.dt-error-frame {
    box-shadow: inset 0 0 0 2px @error_color;
}

textview.dt-error-text text {
    color: @error_color;
}

.dt-tool-card {
    padding: 0;
}

.dt-tool-card .dt-description {
    opacity: 0.7;
}

.dt-icon-bubble {
    border-radius: 20px;
    min-width: 72px;
    min-height: 72px;
    background: linear-gradient(135deg, #9a9996, #5e5c64);
    box-shadow: 0 2px 6px alpha(black, 0.15);
}
.dt-section-convert  { background: linear-gradient(135deg, #c061cb, #813d9c); }
.dt-section-database { background: linear-gradient(135deg, #62a0ea, #1c71d8); }
.dt-section-encoding { background: linear-gradient(135deg, #57e389, #26a269); }
.dt-section-formating { background: linear-gradient(135deg, #ffa348, #e66100); }
.dt-section-string   { background: linear-gradient(135deg, #f66151, #c01c28); }

.dt-panel {
    border-radius: 12px;
}

.dt-panel-header {
    padding: 4px 6px 4px 12px;
    border-bottom: 1px solid alpha(currentColor, 0.1);
}

.dt-panel textview,
.dt-panel textview text {
    background: transparent;
}

.dt-grid-cell {
    padding: 5px 12px;
}

.dt-null {
    opacity: 0.5;
    font-style: italic;
}

.dt-grid-header {
    font-weight: bold;
    background: alpha(currentColor, 0.07);
}

.dt-grid-odd {
    background: alpha(currentColor, 0.03);
}
";

struct Section {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
}

const SECTIONS: &[Section] = &[
    Section {
        id: "convert",
        label: "Convert",
        icon: "media-playlist-shuffle-symbolic",
    },
    Section {
        id: "database",
        label: "Database",
        icon: "network-server-symbolic",
    },
    Section {
        id: "encoding",
        label: "Encoding",
        icon: "security-high-symbolic",
    },
    Section {
        id: "formating",
        label: "Formatting",
        icon: "format-justify-fill-symbolic",
    },
    Section {
        id: "graphics",
        label: "Graphics",
        icon: "image-x-generic-symbolic",
    },
    Section {
        id: "helpful",
        label: "Helpful",
        icon: "help-contents-symbolic",
    },
    Section {
        id: "parsing",
        label: "Parsing",
        icon: "text-x-generic-symbolic",
    },
    Section {
        id: "string",
        label: "String",
        icon: "insert-text-symbolic",
    },
    Section {
        id: "devices",
        label: "Devices",
        icon: "computer-symbolic",
    },
];

const CARD_ICON_SIZE: i32 = 40;

/// "html_to_text" -> "Html To Text"
pub fn title_case(name: &str) -> String {
    name.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars
                .next()
                .map(|c| {
                    c.to_uppercase()
                        .chain(chars.flat_map(char::to_lowercase))
                        .collect()
                })
                .unwrap_or_default()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn section_label(id: &str) -> String {
    SECTIONS
        .iter()
        .find(|s| s.id == id)
        .map_or_else(|| title_case(id), |s| s.label.to_string())
}

fn section_icon(id: &str) -> &'static str {
    SECTIONS
        .iter()
        .find(|s| s.id == id)
        .map_or("applications-utilities-symbolic", |s| s.icon)
}

fn install_app_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(APP_CSS);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Flattens an icon's colors to a single tint, keeping its alpha mask, so the
/// dark silhouettes shipped with each feature read as white glyphs on their
/// colored bubble, in both light and dark themes.
fn tinted_texture(bytes: &'static [u8], size: i32, rgb: [u8; 3]) -> Option<gdk::Texture> {
    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from_static(bytes));
    let pixbuf =
        gdk_pixbuf::Pixbuf::from_stream_at_scale(&stream, size, size, true, gio::Cancellable::NONE)
            .ok()?;
    let pixbuf = if pixbuf.has_alpha() {
        pixbuf
    } else {
        pixbuf.add_alpha(false, 0, 0, 0).ok()?
    };
    let (width, height) = (pixbuf.width() as usize, pixbuf.height() as usize);
    let stride = pixbuf.rowstride() as usize;
    let channels = pixbuf.n_channels() as usize;
    let mut data = pixbuf.read_pixel_bytes().to_vec();
    for y in 0..height {
        for x in 0..width {
            let i = y * stride + x * channels;
            data[i..i + 3].copy_from_slice(&rgb);
        }
    }
    Some(
        gdk::MemoryTexture::new(
            width as i32,
            height as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &glib::Bytes::from_owned(data),
            stride,
        )
        .upcast(),
    )
}

fn feature_icon_bubble(feature: &RegisteredFeature) -> gtk::Box {
    let image = match feature
        .icon
        .and_then(|b| tinted_texture(b, CARD_ICON_SIZE, [0xff, 0xff, 0xff]))
    {
        Some(texture) => gtk::Image::from_paintable(Some(&texture)),
        None => {
            let img = gtk::Image::from_icon_name(section_icon(feature.entity.section));
            img.add_css_class("lightbox");
            img
        }
    };
    image.set_pixel_size(CARD_ICON_SIZE);
    image.set_halign(gtk::Align::Center);
    image.set_valign(gtk::Align::Center);
    image.set_hexpand(true);

    let bubble = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    bubble.add_css_class("dt-icon-bubble");
    bubble.add_css_class(&format!("dt-section-{}", feature.entity.section));
    bubble.set_halign(gtk::Align::Center);
    bubble.append(&image);
    bubble
}

struct MainWindow {
    window: adw::ApplicationWindow,
    nav_view: adw::NavigationView,
    split_view: adw::OverlaySplitView,
    content_bin: adw::Bin,
    content_title: adw::WindowTitle,
    loader: FeatureLoaderService,
    /// `None` is the home page (every tool).
    current_section: Cell<Option<&'static str>>,
    query: RefCell<String>,
}

impl MainWindow {
    fn new(app: &adw::Application) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title(APP_NAME)
            .icon_name(APP_ID)
            .default_width(1100)
            .default_height(720)
            .width_request(360)
            .height_request(400)
            .build();
        let nav_view = adw::NavigationView::new();
        window.set_content(Some(&nav_view));

        let split_view = adw::OverlaySplitView::builder()
            .min_sidebar_width(200.0)
            .max_sidebar_width(260.0)
            .build();

        let breakpoint = adw::Breakpoint::new(
            adw::BreakpointCondition::parse("max-width: 700sp").expect("valid breakpoint"),
        );
        breakpoint.add_setter(&split_view, "collapsed", Some(&true.to_value()));
        window.add_breakpoint(breakpoint);

        let this = Rc::new(Self {
            window,
            nav_view,
            split_view,
            content_bin: adw::Bin::new(),
            content_title: adw::WindowTitle::new("", ""),
            loader: FeatureLoaderService::default(),
            current_section: Cell::new(None),
            query: RefCell::default(),
        });
        this.install_actions();
        this.split_view.set_sidebar(Some(&this.build_sidebar()));
        this.split_view.set_content(Some(&this.build_content()));
        this.nav_view
            .push(&adw::NavigationPage::new(&this.split_view, APP_NAME));
        this.refresh();
        this
    }

    fn install_actions(self: &Rc<Self>) {
        let about_action = gio::SimpleAction::new("about", None);
        let window = self.window.downgrade();
        about_action.connect_activate(move |_, _| {
            if let Some(w) = window.upgrade() {
                show_about(&w);
            }
        });
        self.window.add_action(&about_action);
    }

    fn build_sidebar(self: &Rc<Self>) -> adw::ToolbarView {
        let header_bar = adw::HeaderBar::new();
        header_bar.set_title_widget(Some(&adw::WindowTitle::new(APP_NAME, "")));

        let menu = gio::Menu::new();
        menu.append(Some("About Dupot Dev Tools"), Some("win.about"));
        let menu_btn = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .menu_model(&menu)
            .tooltip_text("Main menu")
            .primary(true)
            .build();
        header_bar.pack_end(&menu_btn);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.add_css_class("navigation-sidebar");

        let sections = self.loader.sections();
        list.append(&sidebar_row(
            "All tools",
            "view-grid-symbolic",
            self.loader.all_features().len(),
        ));
        for section in &sections {
            list.append(&sidebar_row(
                &section_label(section),
                section_icon(section),
                self.loader.features_in_section(section).len(),
            ));
        }
        if let Some(first) = list.row_at_index(0) {
            list.select_row(Some(&first));
        }

        let weak = Rc::downgrade(self);
        list.connect_row_selected(move |_, row| {
            let (Some(this), Some(row)) = (weak.upgrade(), row) else {
                return;
            };
            let section = usize::try_from(row.index() - 1)
                .ok()
                .and_then(|i| sections.get(i));
            this.current_section.set(section.copied());
            this.refresh();
            if this.split_view.is_collapsed() {
                this.split_view.set_show_sidebar(false);
            }
        });

        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .child(&list)
            .build();

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.set_content(Some(&scroll));
        toolbar_view
    }

    fn build_content(self: &Rc<Self>) -> adw::ToolbarView {
        let header_bar = adw::HeaderBar::new();
        header_bar.set_title_widget(Some(&self.content_title));

        let sidebar_btn = gtk::ToggleButton::builder()
            .icon_name("sidebar-show-symbolic")
            .tooltip_text("Show sections")
            .build();
        self.split_view
            .bind_property("show-sidebar", &sidebar_btn, "active")
            .bidirectional()
            .sync_create()
            .build();
        self.split_view
            .bind_property("collapsed", &sidebar_btn, "visible")
            .sync_create()
            .build();
        header_bar.pack_start(&sidebar_btn);

        let search_btn = gtk::ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search tools")
            .build();
        header_bar.pack_end(&search_btn);

        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text("Search tools…")
            .hexpand(true)
            .build();
        let search_clamp = adw::Clamp::builder()
            .maximum_size(420)
            .child(&search_entry)
            .build();
        let search_bar = gtk::SearchBar::builder().child(&search_clamp).build();
        search_bar.connect_entry(&search_entry);
        search_bar.set_key_capture_widget(Some(&self.window));
        search_bar
            .bind_property("search-mode-enabled", &search_btn, "active")
            .bidirectional()
            .sync_create()
            .build();

        let weak = Rc::downgrade(self);
        search_entry.connect_search_changed(move |entry| {
            if let Some(this) = weak.upgrade() {
                *this.query.borrow_mut() = entry.text().trim().to_lowercase();
                this.refresh();
            }
        });

        self.content_bin.set_hexpand(true);
        self.content_bin.set_vexpand(true);

        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header_bar);
        toolbar_view.add_top_bar(&search_bar);
        toolbar_view.set_content(Some(&self.content_bin));
        toolbar_view
    }

    fn visible_features(&self) -> Vec<RegisteredFeature> {
        let query = self.query.borrow();
        if !query.is_empty() {
            return self
                .loader
                .all_features()
                .into_iter()
                .filter(|f| {
                    f.entity.matches(&query)
                        || section_label(f.entity.section)
                            .to_lowercase()
                            .contains(query.as_str())
                })
                .collect();
        }
        match self.current_section.get() {
            None => self.loader.all_features(),
            Some(s) => self.loader.features_in_section(s),
        }
    }

    fn refresh(self: &Rc<Self>) {
        let features = self.visible_features();
        let searching = !self.query.borrow().is_empty();
        let title = if searching {
            "Search results".to_string()
        } else {
            self.current_section
                .get()
                .map_or_else(|| "All tools".to_string(), section_label)
        };
        self.content_title.set_title(&title);
        self.content_title.set_subtitle(&match features.len() {
            1 => "1 tool".to_string(),
            n => format!("{n} tools"),
        });

        if features.is_empty() {
            let status = adw::StatusPage::builder()
                .icon_name("system-search-symbolic")
                .title("No tools found")
                .description("Try another search term")
                .build();
            self.content_bin.set_child(Some(&status));
            return;
        }

        let flow = gtk::FlowBox::builder()
            .valign(gtk::Align::Start)
            .max_children_per_line(5)
            .min_children_per_line(1)
            .homogeneous(true)
            .selection_mode(gtk::SelectionMode::None)
            .row_spacing(16)
            .column_spacing(16)
            .build();

        for feature in features {
            flow.append(&self.build_feature_card(feature));
        }

        let clamp = adw::Clamp::builder()
            .maximum_size(1200)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .child(&flow)
            .build();
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&clamp)
            .build();
        self.content_bin.set_child(Some(&scroll));
    }

    fn build_feature_card(self: &Rc<Self>, feature: RegisteredFeature) -> gtk::Button {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
        content.set_margin_top(20);
        content.set_margin_bottom(20);
        content.set_margin_start(14);
        content.set_margin_end(14);
        content.set_valign(gtk::Align::Start);

        let bubble = feature_icon_bubble(&feature);
        bubble.set_margin_bottom(8);

        let name_lbl = gtk::Label::builder()
            .label(feature.entity.title)
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build();
        name_lbl.add_css_class("heading");

        let desc_lbl = gtk::Label::builder()
            .label(feature.entity.description)
            .wrap(true)
            .justify(gtk::Justification::Center)
            .max_width_chars(24)
            .build();
        desc_lbl.add_css_class("caption");
        desc_lbl.add_css_class("dt-description");

        content.append(&bubble);
        content.append(&name_lbl);
        content.append(&desc_lbl);

        let button = gtk::Button::builder()
            .child(&content)
            .tooltip_text(feature.entity.description)
            .width_request(180)
            .height_request(180)
            .build();
        button.add_css_class("card");
        button.add_css_class("dt-tool-card");

        let weak = Rc::downgrade(self);
        button.connect_clicked(move |_| {
            if let Some(this) = weak.upgrade() {
                let instance = (feature.create)();
                this.nav_view.push(&feature_detail_page::build(
                    &feature.entity,
                    instance.as_ref(),
                ));
            }
        });
        button
    }
}

fn sidebar_row(label_text: &str, icon_name: &str, count: usize) -> gtk::ListBoxRow {
    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row_box.set_margin_start(6);
    row_box.set_margin_end(6);
    row_box.set_margin_top(6);
    row_box.set_margin_bottom(6);

    let label = gtk::Label::builder()
        .label(label_text)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();
    let count_lbl = gtk::Label::new(Some(&count.to_string()));
    count_lbl.add_css_class("dim-label");
    count_lbl.add_css_class("caption");
    count_lbl.add_css_class("numeric");

    row_box.append(&gtk::Image::from_icon_name(icon_name));
    row_box.append(&label);
    row_box.append(&count_lbl);
    gtk::ListBoxRow::builder().child(&row_box).build()
}

fn show_about(parent: &adw::ApplicationWindow) {
    let about = adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .developer_name("Michael Bertocchi")
        .developers(["Michael Bertocchi"])
        .copyright("© Michael Bertocchi")
        .comments("The developer's Swiss Army knife: format, convert, encode, decode, hash… and explore your SQL databases visually. Everything stays on your machine.")
        .website("https://www.dupot.org")
        .issue_url("https://github.com/imikado/dupotDevTools/issues")
        .license_type(gtk::License::Lgpl21)
        .build();
    let has_icon =
        gdk::Display::default().is_some_and(|d| gtk::IconTheme::for_display(&d).has_icon(APP_ID));
    if has_icon {
        about.set_application_icon(APP_ID);
    }
    about.present(Some(parent));
}

pub fn build_application() -> adw::Application {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        install_app_css();
        let main = MainWindow::new(app);
        main.window.present();
        // The window's lifetime anchors the Rc that its callbacks reach weakly.
        let keep = Rc::clone(&main);
        main.window.connect_destroy(move |_| {
            let _ = &keep;
        });
    });
    app
}
