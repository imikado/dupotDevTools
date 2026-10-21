use std::rc::Rc;

use adw::prelude::*;
use gtk::{gdk, gdk_pixbuf, gio, glib};

use crate::domain::entity::FeatureEntity;
use crate::infrastructure::service::feature_loader_service::FeatureLoaderService;

use super::feature_detail_page;

const APP_ID: &str = "org.dupot.devtools";

const APP_CSS: &str = "
.dt-error-frame {
    border: 2px solid @error_color;
    border-radius: 6px;
}

textview.dt-error-text text {
    color: @error_color;
}
";

const SECTION_ICONS: &[(&str, &str)] = &[
    ("convert", "media-playlist-shuffle-symbolic"),
    ("database", "network-server-symbolic"),
    ("encoding", "security-high-symbolic"),
    ("formating", "format-justify-fill-symbolic"),
    ("graphics", "image-x-generic-symbolic"),
    ("helpful", "help-contents-symbolic"),
    ("parsing", "text-x-generic-symbolic"),
    ("string", "insert-text-symbolic"),
    ("devices", "computer-symbolic"),
];

const ICON_SIZE: i32 = 88;

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

/// Flattens an icon's colors to a single tint, keeping its alpha mask.
///
/// Feature icons ship as dark silhouettes on transparent backgrounds, so they
/// vanish against a dark theme. Recoloring them like symbolic icons keeps them
/// legible without needing a background chip behind each one.
fn recolored_texture(bytes: &'static [u8], size: i32, rgb: [u8; 3]) -> Option<gdk::Texture> {
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

fn feature_icon(icon: Option<&'static [u8]>) -> gtk::Image {
    let dark = adw::StyleManager::default().is_dark();
    let tint = if dark {
        [0xff, 0xff, 0xff]
    } else {
        [0x2e, 0x34, 0x36]
    };
    let image = match icon.and_then(|b| recolored_texture(b, ICON_SIZE, tint)) {
        Some(texture) => gtk::Image::from_paintable(Some(&texture)),
        None => gtk::Image::from_icon_name("applications-utilities-symbolic"),
    };
    image.set_pixel_size(ICON_SIZE);
    image.set_size_request(ICON_SIZE, ICON_SIZE);
    image
}

struct MainWindow {
    window: adw::ApplicationWindow,
    nav_view: adw::NavigationView,
    content_bin: adw::Bin,
    loader: FeatureLoaderService,
}

impl MainWindow {
    fn new(app: &adw::Application) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("DevTools")
            .default_width(1100)
            .default_height(700)
            .build();
        let nav_view = adw::NavigationView::new();
        window.set_content(Some(&nav_view));

        let content_bin = adw::Bin::new();
        content_bin.set_hexpand(true);
        content_bin.set_vexpand(true);

        let this = Rc::new(Self {
            window,
            nav_view,
            content_bin,
            loader: FeatureLoaderService::new(),
        });
        this.nav_view.push(&this.build_home_page());
        this.show_section(None);
        this
    }

    fn build_home_page(self: &Rc<Self>) -> adw::NavigationPage {
        let toolbar_view = adw::ToolbarView::new();

        let header_bar = adw::HeaderBar::new();
        header_bar.set_title_widget(Some(&gtk::Label::new(Some("DevTools"))));

        let menu = gio::Menu::new();
        menu.append(Some("About"), Some("win.about"));
        let menu_btn = gtk::MenuButton::new();
        menu_btn.set_icon_name("open-menu-symbolic");
        menu_btn.set_menu_model(Some(&menu));
        header_bar.pack_end(&menu_btn);

        let about_action = gio::SimpleAction::new("about", None);
        let window = self.window.downgrade();
        about_action.connect_activate(move |_, _| {
            if let Some(w) = window.upgrade() {
                show_about(&w);
            }
        });
        self.window.add_action(&about_action);

        toolbar_view.add_top_bar(&header_bar);

        let main_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        main_box.append(&self.build_sidebar());
        main_box.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        main_box.append(&self.content_bin);

        toolbar_view.set_content(Some(&main_box));
        adw::NavigationPage::new(&toolbar_view, "DevTools")
    }

    fn build_sidebar(self: &Rc<Self>) -> gtk::Box {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_size_request(220, -1);

        let lbl = gtk::Label::builder().label("Sections").xalign(0.0).build();
        lbl.add_css_class("heading");
        lbl.set_margin_top(16);
        lbl.set_margin_bottom(8);
        lbl.set_margin_start(16);
        container.append(&lbl);

        let list = gtk::ListBox::new();
        list.set_selection_mode(gtk::SelectionMode::Single);
        list.add_css_class("navigation-sidebar");

        let sections = self.loader.sections();
        list.append(&sidebar_row("Home", "go-home-symbolic"));
        for section in &sections {
            let icon = SECTION_ICONS
                .iter()
                .find(|(s, _)| s == section)
                .map_or("applications-utilities-symbolic", |(_, i)| i);
            list.append(&sidebar_row(&title_case(section), icon));
        }

        let weak = Rc::downgrade(self);
        list.connect_row_activated(move |_, row| {
            if let Some(this) = weak.upgrade() {
                let section = usize::try_from(row.index() - 1)
                    .ok()
                    .and_then(|i| sections.get(i));
                this.show_section(section.copied());
            }
        });

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_child(Some(&list));
        container.append(&scroll);

        if let Some(first) = list.row_at_index(0) {
            list.select_row(Some(&first));
        }
        container
    }

    /// `None` shows the home page: every feature of every section.
    fn show_section(self: &Rc<Self>, section: Option<&str>) {
        let features = match section {
            None => self.loader.all_features(),
            Some(s) => self.loader.features_in_section(s),
        };

        let flow = gtk::FlowBox::new();
        flow.set_valign(gtk::Align::Start);
        flow.set_max_children_per_line(4);
        flow.set_min_children_per_line(2);
        flow.set_selection_mode(gtk::SelectionMode::None);
        flow.set_row_spacing(12);
        flow.set_column_spacing(12);
        flow.set_margin_start(24);
        flow.set_margin_end(24);
        flow.set_margin_top(24);
        flow.set_margin_bottom(24);

        for feature in features {
            flow.append(&self.build_feature_card(feature));
        }

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);
        scroll.set_child(Some(&flow));
        self.content_bin.set_child(Some(&scroll));
    }

    fn build_feature_card(self: &Rc<Self>, feature: FeatureEntity) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("card");
        button.set_size_request(210, 210);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content.set_halign(gtk::Align::Center);
        content.set_valign(gtk::Align::Center);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let icon_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        icon_box.set_size_request(ICON_SIZE, ICON_SIZE);
        icon_box.set_halign(gtk::Align::Center);
        icon_box.set_valign(gtk::Align::Center);
        icon_box.append(&feature_icon(feature.icon));

        let name_lbl = gtk::Label::new(Some(&title_case(feature.name)));
        name_lbl.add_css_class("title-4");
        name_lbl.set_wrap(true);
        name_lbl.set_justify(gtk::Justification::Center);

        let desc_lbl = gtk::Label::new(Some(feature.description));
        desc_lbl.add_css_class("dim-label");
        desc_lbl.add_css_class("caption");
        desc_lbl.set_wrap(true);
        desc_lbl.set_justify(gtk::Justification::Center);
        desc_lbl.set_max_width_chars(22);

        content.append(&icon_box);
        content.append(&name_lbl);
        content.append(&desc_lbl);
        button.set_child(Some(&content));

        let weak = Rc::downgrade(self);
        button.connect_clicked(move |_| {
            if let Some(this) = weak.upgrade() {
                let instance = (feature.create)();
                this.nav_view
                    .push(&feature_detail_page::build(&feature, instance.as_ref()));
            }
        });
        button
    }
}

fn sidebar_row(label_text: &str, icon_name: &str) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row_box.set_margin_start(12);
    row_box.set_margin_end(12);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);

    let label = gtk::Label::new(Some(label_text));
    label.set_halign(gtk::Align::Start);
    label.set_hexpand(true);

    row_box.append(&gtk::Image::from_icon_name(icon_name));
    row_box.append(&label);
    row.set_child(Some(&row_box));
    row
}

fn show_about(parent: &adw::ApplicationWindow) {
    let about = adw::AboutDialog::new();
    about.set_application_name("DevTools");
    about.set_application_icon(APP_ID);
    about.set_version(env!("CARGO_PKG_VERSION"));
    about.set_developer_name("dupot");
    about.set_license_type(gtk::License::Lgpl21);
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
