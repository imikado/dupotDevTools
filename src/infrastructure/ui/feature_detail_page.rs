use std::rc::Weak;

use adw::prelude::*;

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::infrastructure::ui::feature_contract::{FeatureContract, FeatureView, LeaveGuard};

/// The page and its leave guard, if the feature has one.
pub fn build(
    entity: &FeatureEntity,
    feature: &dyn FeatureContract,
) -> (adw::NavigationPage, Option<Weak<dyn LeaveGuard>>) {
    let header_bar = adw::HeaderBar::new();
    header_bar.set_title_widget(Some(&adw::WindowTitle::new(
        entity.title,
        entity.description,
    )));

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);

    let FeatureView {
        widget: content,
        leave_guard,
    } = feature.view();
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
    let page = adw::NavigationPage::new(&toolbar_view, entity.title);

    if let Some(guard) = &leave_guard {
        // Popping only through this button lets the guard run first.
        page.set_can_pop(false);
        let back_btn = gtk::Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text("Back")
            .build();
        header_bar.pack_start(&back_btn);
        let guard = guard.clone();
        let page_weak = page.downgrade();
        back_btn.connect_clicked(move |_| {
            let Some(guard) = guard.upgrade() else { return };
            let page_weak = page_weak.clone();
            guard.confirm_leave(Box::new(move || {
                if let Some(nav) = page_weak
                    .upgrade()
                    .and_then(|p| p.ancestor(adw::NavigationView::static_type()))
                    .and_downcast::<adw::NavigationView>()
                {
                    nav.pop();
                }
            }));
        });
    }
    (page, leave_guard)
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
