//! The SQL viewer's part of the scripted demo (see `ui::demo`): creates a
//! connection to `DUPOT_DEMO_DB`, builds a query on the canvas, runs it, then
//! types SQL and lets the canvas redraw it.

use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use super::SqlViewer;
use crate::domain::entity::database::DbKind;
use crate::domain::feature::database::sql_viewer::SortDirection;
use crate::infrastructure::ui::demo::{
    self, find, pause, record, shot, type_into, type_into_buffer,
};

const TYPED_SQL: &str = "SELECT a.name, b.title, b.price\n\
                         FROM author a\n\
                         JOIN book b ON b.author_id = a.id\n\
                         WHERE b.price > 10\n\
                         ORDER BY b.price DESC";

impl SqlViewer {
    pub(super) fn start_demo(self: &Rc<Self>) {
        let this = Rc::clone(self);
        glib::spawn_future_local(async move {
            if let Some(db) = demo::database() {
                this.demo(&db).await;
            }
            demo::done();
        });
    }

    async fn demo(self: &Rc<Self>, db: &str) {
        pause(1000).await;
        record(true).await;

        // A new SQLite connection, typed into the dialog.
        let dialog = self.open_new_connection_dialog();
        pause(600).await;
        let entry_row = |title: &str| find::<adw::EntryRow>(&dialog, &|row| row.title() == title);
        if let Some(row) = find::<adw::ComboRow>(&dialog, &|row| row.title() == "Type") {
            let sqlite = DbKind::ALL.iter().position(|k| *k == DbKind::Sqlite);
            row.set_selected(sqlite.unwrap_or(0) as u32);
        }
        if let Some(row) = entry_row("Connection name") {
            type_into(&row, "Library (SQLite)").await;
        }
        if let Some(row) = entry_row("Database file") {
            row.set_text(db);
        }
        shot("sqlviewer_new_connection").await;
        match find::<gtk::Button>(&dialog, &|b| b.label().as_deref() == Some("Create")) {
            Some(create) => create.emit_clicked(),
            None => return,
        }
        pause(800).await;
        self.do_connect();
        if !self.db.borrow().is_connected() {
            return;
        }
        pause(1200).await;

        // Tables, joined by their foreign keys.
        self.add_table_card("author", 40.0, 30.0);
        pause(1000).await;
        self.add_table_card("book", 440.0, 20.0);
        pause(1000).await;
        let picker = self.show_table_picker(880.0, 30.0);
        pause(1200).await;
        shot("sqlviewer_add_table").await;
        picker.popdown();
        self.add_table_card("publisher", 880.0, 30.0);
        pause(800).await;
        self.sql_toggle.set_active(true);
        shot("sqlviewer_joins").await;

        // Filter and sort in the criteria, then run.
        let row = |table: &str, column: &str| {
            self.criteria_rows
                .borrow()
                .iter()
                .find(|r| r.card.table == table && r.column == column)
                .cloned()
        };
        if let Some(price) = row("book", "price") {
            *price.filter.borrow_mut() = "> 10".to_string();
        }
        if let Some(name) = row("author", "name") {
            name.sort.set(Some(SortDirection::Ascending));
            name.sort_order.set(1);
        }
        self.rebuild_criteria_grid();
        self.canvas_changed();
        pause(1500).await;
        self.do_run_query();
        shot("sqlviewer_request").await;
        pause(1500).await;

        // SQL typed by hand, drawn back on the canvas.
        self.sql_buffer.set_text("");
        type_into_buffer(&self.sql_buffer, TYPED_SQL).await;
        pause(1500).await;
        self.do_run_query();
        shot("sqlviewer_sql_to_canvas").await;
        pause(1500).await;
        record(false).await;
    }
}
