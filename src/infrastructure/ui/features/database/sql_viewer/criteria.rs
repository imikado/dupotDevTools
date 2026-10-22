//! The criteria grid: one row per canvas column that is output, sorted or
//! filtered, as in SQL Server Management Studio's query designer.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use super::{SqlViewer, TableCard, close_button, hint};
use crate::domain::feature::database::sql_viewer::{ColumnCriteria, SortDirection};

const SORT_LABELS: [&str; 3] = ["Unsorted", "Ascending", "Descending"];
const HEADERS: [&str; 8] = [
    "Column", "Table", "Alias", "Output", "Sort", "Order", "Filter", "Or…",
];

pub(super) struct CriteriaRow {
    pub card: Rc<TableCard>,
    pub column: String,
    pub alias: RefCell<String>,
    pub sort: Cell<Option<SortDirection>>,
    pub sort_order: Cell<u32>,
    pub filter: RefCell<String>,
    pub or_filter: RefCell<String>,
    /// Added from the grid: kept even when not output and without criteria.
    pub pinned: Cell<bool>,
    order_lbl: RefCell<Option<gtk::Label>>,
}

impl CriteriaRow {
    fn new(card: &Rc<TableCard>, column: &str) -> Rc<Self> {
        Rc::new(Self {
            card: Rc::clone(card),
            column: column.to_string(),
            alias: RefCell::default(),
            sort: Cell::new(None),
            sort_order: Cell::new(0),
            filter: RefCell::default(),
            or_filter: RefCell::default(),
            pinned: Cell::new(false),
            order_lbl: RefCell::default(),
        })
    }

    fn is_output(&self) -> bool {
        self.card
            .column(&self.column)
            .is_some_and(|c| c.check.is_active())
    }

    fn has_criteria(&self) -> bool {
        !self.alias.borrow().trim().is_empty()
            || self.sort.get().is_some()
            || !self.filter.borrow().trim().is_empty()
            || !self.or_filter.borrow().trim().is_empty()
    }

    fn is_for(&self, card: &Rc<TableCard>, column: &str) -> bool {
        Rc::ptr_eq(&self.card, card) && self.column == column
    }
}

impl SqlViewer {
    /// Matches the rows to the canvas: drops rows of removed tables and of
    /// unchecked columns without criteria, adds checked columns. Rebuilds
    /// the grid only when the rows changed, so typing keeps its focus.
    pub(super) fn sync_criteria_rows(self: &Rc<Self>) {
        let cards = self.table_cards.borrow().clone();
        let changed = {
            let mut rows = self.criteria_rows.borrow_mut();
            let before = rows.len();
            rows.retain(|r| {
                cards.iter().any(|c| Rc::ptr_eq(c, &r.card))
                    && (r.is_output() || r.has_criteria() || r.pinned.get())
            });
            let mut changed = rows.len() != before;
            for card in &cards {
                for col in card.columns.iter().filter(|c| c.check.is_active()) {
                    if !rows.iter().any(|r| r.is_for(card, &col.name)) {
                        rows.push(CriteriaRow::new(card, &col.name));
                        changed = true;
                    }
                }
            }
            changed
        };
        if changed {
            self.renumber_sorts();
            self.rebuild_criteria_grid();
        }
    }

    /// The rows as the domain's criteria, `table` indexing `cards`.
    pub(super) fn criteria_snapshot(&self, cards: &[Rc<TableCard>]) -> Vec<ColumnCriteria> {
        self.criteria_rows
            .borrow()
            .iter()
            .filter_map(|r| {
                Some(ColumnCriteria {
                    table: cards.iter().position(|c| Rc::ptr_eq(c, &r.card))?,
                    column: r.column.clone(),
                    alias: r.alias.borrow().clone(),
                    sort: r.sort.get(),
                    sort_order: r.sort_order.get(),
                    filter: r.filter.borrow().clone(),
                    or_filter: r.or_filter.borrow().clone(),
                })
            })
            .collect()
    }

    /// Replaces every row's criteria with `criteria`, whose `table` indexes
    /// `cards` (`None` for a table that isn't on the canvas).
    pub(super) fn apply_criteria(
        self: &Rc<Self>,
        cards: &[Option<Rc<TableCard>>],
        criteria: &[ColumnCriteria],
    ) {
        {
            let mut rows = self.criteria_rows.borrow_mut();
            for row in rows.iter() {
                row.alias.borrow_mut().clear();
                row.sort.set(None);
                row.sort_order.set(0);
                row.filter.borrow_mut().clear();
                row.or_filter.borrow_mut().clear();
                row.pinned.set(false);
            }
            for c in criteria {
                let Some(Some(card)) = cards.get(c.table) else {
                    continue;
                };
                let Some(column) = card.column(&c.column).map(|w| w.name.clone()) else {
                    continue;
                };
                let row = match rows.iter().find(|r| r.is_for(card, &column)) {
                    Some(row) => Rc::clone(row),
                    None => {
                        let row = CriteriaRow::new(card, &column);
                        rows.push(Rc::clone(&row));
                        row
                    }
                };
                *row.alias.borrow_mut() = c.alias.clone();
                row.sort.set(c.sort);
                row.sort_order.set(c.sort_order);
                *row.filter.borrow_mut() = c.filter.clone();
                *row.or_filter.borrow_mut() = c.or_filter.clone();
            }
        }
        self.sync_criteria_rows();
        self.renumber_sorts();
        self.rebuild_criteria_grid();
    }

    /// Sort orders become 1, 2, 3… keeping their relative order.
    fn renumber_sorts(&self) {
        let rows = self.criteria_rows.borrow();
        let mut sorted: Vec<&Rc<CriteriaRow>> =
            rows.iter().filter(|r| r.sort.get().is_some()).collect();
        sorted.sort_by_key(|r| r.sort_order.get());
        for (i, row) in sorted.iter().enumerate() {
            row.sort_order.set(i as u32 + 1);
        }
        for row in rows.iter() {
            if row.sort.get().is_none() {
                row.sort_order.set(0);
            }
            if let Some(lbl) = &*row.order_lbl.borrow() {
                lbl.set_text(&match row.sort_order.get() {
                    0 => String::new(),
                    n => n.to_string(),
                });
            }
        }
    }

    fn criteria_edited(self: &Rc<Self>) {
        if !self.syncing.get() {
            self.canvas_changed();
        }
    }

    pub(super) fn rebuild_criteria_grid(self: &Rc<Self>) {
        let grid = &self.criteria_grid;
        while let Some(child) = grid.first_child() {
            grid.remove(&child);
        }
        let rows = self.criteria_rows.borrow().clone();
        if rows.is_empty() {
            grid.attach(
                &hint("Columns checked on the canvas appear here, to rename, sort or filter them."),
                0,
                0,
                1,
                1,
            );
            return;
        }
        for (c, title) in HEADERS.iter().enumerate() {
            let lbl = gtk::Label::builder().label(*title).xalign(0.0).build();
            lbl.add_css_class("caption-heading");
            lbl.add_css_class("dim-label");
            grid.attach(&lbl, c as i32, 0, 1, 1);
        }
        self.quietly(|| {
            for (r, row) in rows.iter().enumerate() {
                self.attach_criteria_row(row, r as i32 + 1);
            }
        });
    }

    fn attach_criteria_row(self: &Rc<Self>, row: &Rc<CriteriaRow>, y: i32) {
        let grid = &self.criteria_grid;
        let weak = Rc::downgrade(self);

        let column_lbl = gtk::Label::builder().label(&row.column).xalign(0.0).build();
        let table_lbl = gtk::Label::builder()
            .label(&row.card.table)
            .xalign(0.0)
            .build();
        table_lbl.add_css_class("dim-label");

        let text_entry = |value: &str, placeholder: &str, width: i32| {
            gtk::Entry::builder()
                .text(value)
                .placeholder_text(placeholder)
                .width_chars(width)
                .hexpand(width > 10)
                .build()
        };
        let alias = text_entry(&row.alias.borrow(), "", 8);
        let filter = text_entry(&row.filter.borrow(), "e.g. > 8", 14);
        let or_filter = text_entry(&row.or_filter.borrow(), "", 14);
        for (entry, cell) in [(&alias, 0usize), (&filter, 1), (&or_filter, 2)] {
            let (w, row) = (weak.clone(), Rc::clone(row));
            entry.connect_changed(move |e| {
                let target = match cell {
                    0 => &row.alias,
                    1 => &row.filter,
                    _ => &row.or_filter,
                };
                *target.borrow_mut() = e.text().into();
                if let Some(this) = w.upgrade() {
                    this.criteria_edited();
                }
            });
        }

        let output = gtk::CheckButton::new();
        output.set_halign(gtk::Align::Center);
        if let Some(col) = row.card.column(&row.column) {
            col.check
                .bind_property("active", &output, "active")
                .bidirectional()
                .sync_create()
                .build();
        }
        // Unchecking here keeps the row, to go on filtering on the column.
        // The binding also toggles it when the canvas check changes; only a
        // click here (which focuses it) pins the row.
        let r = Rc::clone(row);
        output.connect_toggled(move |btn| {
            if btn.has_focus() {
                r.pinned.set(true);
            }
        });

        let sort = gtk::DropDown::from_strings(&SORT_LABELS);
        sort.set_selected(match row.sort.get() {
            None => 0,
            Some(SortDirection::Ascending) => 1,
            Some(SortDirection::Descending) => 2,
        });
        let order_lbl = gtk::Label::new(None);
        order_lbl.add_css_class("dim-label");
        *row.order_lbl.borrow_mut() = Some(order_lbl.clone());
        let (w, r) = (weak.clone(), Rc::clone(row));
        sort.connect_selected_notify(move |dd| {
            let Some(this) = w.upgrade() else { return };
            let direction = match dd.selected() {
                1 => Some(SortDirection::Ascending),
                2 => Some(SortDirection::Descending),
                _ => None,
            };
            if r.sort.get().is_none() && direction.is_some() {
                // A newly sorted column sorts last.
                r.sort_order.set(u32::MAX);
            }
            r.sort.set(direction);
            this.renumber_sorts();
            this.criteria_edited();
        });

        let remove_btn = close_button();
        remove_btn.set_tooltip_text(Some("Remove this row and uncheck the column"));
        let (w, r) = (weak.clone(), Rc::clone(row));
        remove_btn.connect_clicked(move |_| {
            let Some(this) = w.upgrade() else { return };
            this.criteria_rows
                .borrow_mut()
                .retain(|x| !Rc::ptr_eq(x, &r));
            this.renumber_sorts();
            this.rebuild_criteria_grid();
            // Unchecking calls `canvas_changed`; do it once the row is gone.
            match r.card.column(&r.column) {
                Some(col) if col.check.is_active() => col.check.set_active(false),
                _ => this.criteria_edited(),
            }
        });

        let cells: [&gtk::Widget; 9] = [
            column_lbl.upcast_ref(),
            table_lbl.upcast_ref(),
            alias.upcast_ref(),
            output.upcast_ref(),
            sort.upcast_ref(),
            order_lbl.upcast_ref(),
            filter.upcast_ref(),
            or_filter.upcast_ref(),
            remove_btn.upcast_ref(),
        ];
        for (x, cell) in cells.into_iter().enumerate() {
            grid.attach(cell, x as i32, y, 1, 1);
        }
        self.renumber_sorts();
    }

    /// Fills the "add a column" menu with the canvas columns not in the grid.
    pub(super) fn setup_add_criteria_menu(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.add_criteria_btn.set_create_popup_func(move |btn| {
            let Some(this) = weak.upgrade() else { return };
            let listbox = gtk::ListBox::new();
            listbox.add_css_class("navigation-sidebar");
            let mut choices: Vec<(Rc<TableCard>, String)> = Vec::new();
            {
                let rows = this.criteria_rows.borrow();
                for card in this.table_cards.borrow().iter() {
                    for col in &card.columns {
                        if !rows.iter().any(|r| r.is_for(card, &col.name)) {
                            choices.push((Rc::clone(card), col.name.clone()));
                        }
                    }
                }
            }
            let popover = gtk::Popover::new();
            if choices.is_empty() {
                popover.set_child(Some(&hint("Every canvas column is already in the grid.")));
                btn.set_popover(Some(&popover));
                return;
            }
            for (card, column) in &choices {
                let lbl = gtk::Label::builder()
                    .label(format!("{}.{column}", card.table))
                    .xalign(0.0)
                    .build();
                listbox.append(&lbl);
            }
            let w = Rc::downgrade(&this);
            listbox.connect_row_activated(glib::clone!(
                #[weak]
                popover,
                move |_, row| {
                    popover.popdown();
                    let (Some(this), Some((card, column))) = (
                        w.upgrade(),
                        usize::try_from(row.index())
                            .ok()
                            .and_then(|i| choices.get(i)),
                    ) else {
                        return;
                    };
                    let new_row = CriteriaRow::new(card, column);
                    new_row.pinned.set(true);
                    this.criteria_rows.borrow_mut().push(new_row);
                    this.rebuild_criteria_grid();
                }
            ));
            let scroll = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .max_content_height(300)
                .propagate_natural_height(true)
                .propagate_natural_width(true)
                .child(&listbox)
                .build();
            popover.set_child(Some(&scroll));
            btn.set_popover(Some(&popover));
        });
    }
}
