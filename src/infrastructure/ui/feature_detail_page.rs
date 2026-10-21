use adw::prelude::*;

use crate::domain::contract::FeatureContract;
use crate::domain::entity::FeatureEntity;

pub fn build(entity: &FeatureEntity, feature: &dyn FeatureContract) -> adw::NavigationPage {
    let header_bar = adw::HeaderBar::new();
    header_bar.set_title_widget(Some(&adw::WindowTitle::new(
        entity.title,
        entity.description,
    )));

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);

    let content = feature.widget();
    let outer: gtk::Widget = if feature.full_width() {
        content.set_hexpand(true);
        content.upcast()
    } else {
        adw::Clamp::builder()
            .maximum_size(1200)
            .tightening_threshold(900)
            .child(&content)
            .build()
            .upcast()
    };
    outer.set_margin_top(18);
    outer.set_margin_bottom(18);
    outer.set_margin_start(18);
    outer.set_margin_end(18);

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .child(&outer)
        .build();

    // Features reach this through `widget.ancestor(ToastOverlay)`.
    let toasts = adw::ToastOverlay::new();
    toasts.set_child(Some(&scroll));
    toolbar_view.set_content(Some(&toasts));
    adw::NavigationPage::new(&toolbar_view, entity.title)
}

/// Shows a short notification on the page hosting `widget`.
pub fn toast(widget: &impl IsA<gtk::Widget>, message: &str) {
    if let Some(overlay) = widget.ancestor(adw::ToastOverlay::static_type()) {
        let toast = adw::Toast::new(message);
        toast.set_timeout(2);
        overlay
            .downcast_ref::<adw::ToastOverlay>()
            .unwrap()
            .add_toast(toast);
    }
}
