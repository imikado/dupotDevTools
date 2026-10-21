use adw::prelude::*;

use crate::domain::contract::FeatureContract;
use crate::domain::entity::FeatureEntity;

use super::app_window::title_case;

pub fn build(entity: &FeatureEntity, feature: &dyn FeatureContract) -> adw::NavigationPage {
    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&adw::HeaderBar::new());

    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_hexpand(true);

    let content = feature.widget();
    let outer: gtk::Widget = if feature.full_width() {
        content.set_hexpand(true);
        content.upcast()
    } else {
        let clamp = adw::Clamp::new();
        clamp.set_maximum_size(860);
        clamp.set_tightening_threshold(640);
        clamp.set_child(Some(&content));
        clamp.upcast()
    };
    outer.set_margin_top(24);
    outer.set_margin_bottom(24);
    outer.set_margin_start(12);
    outer.set_margin_end(12);
    scroll.set_child(Some(&outer));

    toolbar_view.set_content(Some(&scroll));
    adw::NavigationPage::new(&toolbar_view, &title_case(entity.name))
}
