pub trait FeatureContract {
    /// Opt out of the detail page's width clamp (e.g. for canvas/table-like
    /// UIs that need the full window width).
    fn full_width(&self) -> bool {
        false
    }

    fn widget(&self) -> gtk::Widget;
}
