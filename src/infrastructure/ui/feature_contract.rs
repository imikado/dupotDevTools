use std::rc::{Rc, Weak};

/// A tool's GTK view. Its logic lives in `domain::feature`; the view only
/// builds widgets and wires them to that logic.
pub trait FeatureContract {
    /// Opt out of the detail page's width clamp (e.g. for canvas/table-like
    /// UIs that need the full window width).
    fn full_width(&self) -> bool {
        false
    }

    /// Opt out of the detail page's vertical scrolling, for views that fill
    /// the window and scroll their own panes.
    fn scrollable(&self) -> bool {
        true
    }

    fn widget(&self) -> gtk::Widget;

    /// The widget and, for tools holding unsaved work, a guard consulted
    /// before leaving the page or closing the window.
    fn view(&self) -> FeatureView {
        FeatureView {
            widget: self.widget(),
            leave_guard: None,
        }
    }
}

pub struct FeatureView {
    pub widget: gtk::Widget,
    /// Weak: the view's widgets own the guard, not the page or the window.
    pub leave_guard: Option<Weak<dyn LeaveGuard>>,
}

pub trait LeaveGuard {
    /// Calls `proceed` once leaving is fine, right away or after asking the
    /// user; drops it if they cancel.
    fn confirm_leave(self: Rc<Self>, proceed: Box<dyn FnOnce()>);
}
