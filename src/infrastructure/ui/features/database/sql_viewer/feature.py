import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, GLib, Gtk

from domain.contract.feature_contract import FeatureContract
from infrastructure.api.database_api import DatabaseApi, DatabaseConnectionError

_DB_KINDS = ["sqlite", "postgresql", "mysql"]
_DB_LABELS = ["SQLite", "PostgreSQL", "MySQL"]
_DEFAULT_PORTS = {"postgresql": "5432", "mysql": "3306"}


class Feature(FeatureContract):
    def get_widget(self):
        db = DatabaseApi()
        connections = []  # [{"name", "kind", "params"}, ...], mirrors conn_model
        tables_cache = []  # table names on the active connection

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)

        # --- Connection bar ---
        conn_lbl = Gtk.Label(label="Connection", xalign=0)
        conn_lbl.add_css_class("heading")

        conn_model = Gtk.StringList.new([])
        conn_dropdown = Gtk.DropDown(model=conn_model)
        conn_dropdown.set_hexpand(True)

        new_conn_btn = Gtk.Button(label="New Connection…")
        connect_btn = Gtk.Button(label="Connect")
        connect_btn.add_css_class("suggested-action")

        conn_bar = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        conn_bar.append(conn_dropdown)
        conn_bar.append(new_conn_btn)
        conn_bar.append(connect_btn)

        status_lbl = Gtk.Label(label="")
        status_lbl.add_css_class("dim-label")
        status_lbl.set_halign(Gtk.Align.START)
        status_lbl.set_wrap(True)

        def set_error(message: str):
            status_lbl.remove_css_class("dim-label")
            status_lbl.add_css_class("error")
            status_lbl.set_text(f"⚠ Error: {message}")

        def set_status(message: str):
            status_lbl.remove_css_class("error")
            status_lbl.add_css_class("dim-label")
            status_lbl.set_text(message)

        # --- Canvas ---
        canvas_lbl = Gtk.Label(label="Schema canvas", xalign=0)
        canvas_lbl.add_css_class("heading")

        canvas_hint = Gtk.Label(
            label="Right-click the canvas to add a connected table.", xalign=0
        )
        canvas_hint.add_css_class("dim-label")
        canvas_hint.add_css_class("caption")

        fixed = Gtk.Fixed()
        fixed.set_size_request(2000, 1200)
        fixed.add_css_class("view")

        canvas_scroll = Gtk.ScrolledWindow()
        canvas_scroll.add_css_class("card")
        canvas_scroll.set_min_content_height(420)
        canvas_scroll.set_vexpand(True)
        canvas_scroll.set_hexpand(True)
        canvas_scroll.set_child(fixed)

        # --- Table card on the canvas ---
        def build_table_card(table_name, columns):
            frame = Gtk.Frame()
            frame.add_css_class("card")
            frame.set_size_request(220, -1)

            vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
            vbox.set_margin_top(10)
            vbox.set_margin_bottom(10)
            vbox.set_margin_start(10)
            vbox.set_margin_end(10)

            header = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            name_lbl = Gtk.Label(label=table_name, xalign=0)
            name_lbl.add_css_class("heading")
            name_lbl.set_hexpand(True)
            close_btn = Gtk.Button()
            close_btn.set_icon_name("window-close-symbolic")
            close_btn.add_css_class("flat")
            close_btn.add_css_class("circular")
            header.append(name_lbl)
            header.append(close_btn)
            header.set_cursor(Gdk.Cursor.new_from_name("move"))

            fields_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
            if columns:
                for col_name, col_type in columns:
                    label = f"{col_name}  ·  {col_type}" if col_type else col_name
                    check = Gtk.CheckButton(label=label)
                    check.set_active(True)
                    fields_box.append(check)
            else:
                empty_lbl = Gtk.Label(label="No columns found")
                empty_lbl.add_css_class("dim-label")
                fields_box.append(empty_lbl)

            vbox.append(header)
            vbox.append(Gtk.Separator())
            if len(columns) > 8:
                fields_scroll = Gtk.ScrolledWindow()
                fields_scroll.set_max_content_height(220)
                fields_scroll.set_propagate_natural_height(True)
                fields_scroll.set_child(fields_box)
                vbox.append(fields_scroll)
            else:
                vbox.append(fields_box)

            frame.set_child(vbox)
            close_btn.connect("clicked", lambda _b: fixed.remove(frame))

            start_pos = [0.0, 0.0]
            drag = Gtk.GestureDrag()

            def on_drag_begin(_g, _x, _y):
                start_pos[0], start_pos[1] = fixed.get_child_position(frame)

            def on_drag_update(_g, dx, dy):
                fixed.move(frame, start_pos[0] + dx, start_pos[1] + dy)

            drag.connect("drag-begin", on_drag_begin)
            drag.connect("drag-update", on_drag_update)
            header.add_controller(drag)

            return frame

        def add_table_card(table_name, x, y):
            try:
                columns = db.list_columns(table_name)
            except Exception as e:
                set_error(str(e))
                return
            fixed.put(build_table_card(table_name, columns), x, y)
            set_status(f"Added “{table_name}” to the canvas.")

        def show_table_picker(x, y):
            popover = Gtk.Popover()
            popover.set_parent(fixed)
            rect = Gdk.Rectangle()
            rect.x, rect.y, rect.width, rect.height = int(x), int(y), 1, 1
            popover.set_pointing_to(rect)

            listbox = Gtk.ListBox()
            listbox.add_css_class("boxed-list")
            for table_name in tables_cache:
                row = Gtk.ListBoxRow()
                row_lbl = Gtk.Label(label=table_name, xalign=0)
                row_lbl.set_margin_top(6)
                row_lbl.set_margin_bottom(6)
                row_lbl.set_margin_start(10)
                row_lbl.set_margin_end(10)
                row.set_child(row_lbl)
                row._table_name = table_name
                listbox.append(row)

            def on_row_activated(_lb, row):
                popover.popdown()
                add_table_card(row._table_name, x, y)

            listbox.connect("row-activated", on_row_activated)

            list_scroll = Gtk.ScrolledWindow()
            list_scroll.set_max_content_height(240)
            list_scroll.set_propagate_natural_height(True)
            list_scroll.set_child(listbox)
            popover.set_child(list_scroll)
            popover.connect("closed", lambda p: p.unparent())
            popover.popup()

        def on_canvas_right_click(gesture, _n_press, x, y):
            gesture.set_state(Gtk.EventSequenceState.CLAIMED)
            if not db.is_connected():
                set_error("Connect to a database first.")
                return
            if not tables_cache:
                set_error("No tables found on this connection.")
                return
            show_table_picker(x, y)

        right_click = Gtk.GestureClick(button=Gdk.BUTTON_SECONDARY)
        right_click.connect("pressed", on_canvas_right_click)
        fixed.add_controller(right_click)

        # --- New connection dialog ---
        def open_new_connection_dialog(_btn):
            dialog = Adw.Dialog()
            dialog.set_title("New Database Connection")
            dialog.set_content_width(420)

            toolbar_view = Adw.ToolbarView()
            header_bar = Adw.HeaderBar()
            header_bar.set_show_end_title_buttons(False)
            cancel_btn = Gtk.Button(label="Cancel")
            create_btn = Gtk.Button(label="Create")
            create_btn.add_css_class("suggested-action")
            header_bar.pack_start(cancel_btn)
            header_bar.pack_end(create_btn)
            toolbar_view.add_top_bar(header_bar)

            page = Adw.PreferencesPage()

            general_group = Adw.PreferencesGroup()
            name_row = Adw.EntryRow()
            name_row.set_title("Connection name")
            type_row = Adw.ComboRow()
            type_row.set_title("Type")
            type_row.set_model(Gtk.StringList.new(_DB_LABELS))
            general_group.add(name_row)
            general_group.add(type_row)
            page.add(general_group)

            sqlite_group = Adw.PreferencesGroup()
            sqlite_group.set_title("SQLite")
            path_row = Adw.EntryRow()
            path_row.set_title("Database file")
            browse_btn = Gtk.Button()
            browse_btn.set_icon_name("document-open-symbolic")
            browse_btn.add_css_class("flat")
            path_row.add_suffix(browse_btn)
            sqlite_group.add(path_row)
            page.add(sqlite_group)

            server_group = Adw.PreferencesGroup()
            server_group.set_title("Server")
            host_row = Adw.EntryRow()
            host_row.set_title("Host")
            host_row.set_text("localhost")
            port_row = Adw.EntryRow()
            port_row.set_title("Port")
            database_row = Adw.EntryRow()
            database_row.set_title("Database")
            user_row = Adw.EntryRow()
            user_row.set_title("User")
            password_row = Adw.PasswordEntryRow()
            password_row.set_title("Password")
            for row in (host_row, port_row, database_row, user_row, password_row):
                server_group.add(row)
            page.add(server_group)

            def update_visibility(*_a):
                kind = _DB_KINDS[type_row.get_selected()]
                is_sqlite = kind == "sqlite"
                sqlite_group.set_visible(is_sqlite)
                server_group.set_visible(not is_sqlite)
                if not is_sqlite and not port_row.get_text():
                    port_row.set_text(_DEFAULT_PORTS[kind])

            type_row.connect("notify::selected", update_visibility)
            update_visibility()

            toolbar_view.set_content(page)
            dialog.set_child(toolbar_view)

            def do_browse(_b):
                file_dialog = Gtk.FileDialog()

                def on_open(d, result):
                    try:
                        f = d.open_finish(result)
                        path_row.set_text(f.get_path())
                    except GLib.Error:
                        pass

                file_dialog.open(new_conn_btn.get_root(), None, on_open)

            browse_btn.connect("clicked", do_browse)

            def do_create(_b):
                kind = _DB_KINDS[type_row.get_selected()]
                name = name_row.get_text().strip() or f"connection-{len(connections) + 1}"

                if kind == "sqlite":
                    path = path_row.get_text().strip()
                    if not path:
                        path_row.add_css_class("error")
                        return
                    params = {"path": path}
                else:
                    params = {
                        "host": host_row.get_text().strip() or "localhost",
                        "port": port_row.get_text().strip() or _DEFAULT_PORTS[kind],
                        "database": database_row.get_text().strip(),
                        "user": user_row.get_text().strip(),
                        "password": password_row.get_text(),
                    }

                connections.append({"name": name, "kind": kind, "params": params})
                conn_model.append(name)
                conn_dropdown.set_selected(conn_model.get_n_items() - 1)
                set_status(f"Connection “{name}” created. Click Connect to use it.")
                dialog.close()

            cancel_btn.connect("clicked", lambda _b: dialog.close())
            create_btn.connect("clicked", do_create)

            dialog.present(new_conn_btn)

        def do_connect(_btn):
            idx = conn_dropdown.get_selected()
            if idx == Gtk.INVALID_LIST_POSITION or idx >= len(connections):
                set_error("Create a connection first.")
                return
            cfg = connections[idx]
            try:
                db.connect(cfg["kind"], **cfg["params"])
                tables_cache[:] = db.list_tables()
            except DatabaseConnectionError as e:
                set_error(str(e))
                return
            except Exception as e:
                set_error(str(e))
                return
            set_status(
                f"Connected to “{cfg['name']}” — {len(tables_cache)} table(s) found. "
                "Right-click the canvas to add one."
            )

        new_conn_btn.connect("clicked", open_new_connection_dialog)
        connect_btn.connect("clicked", do_connect)

        root.append(conn_lbl)
        root.append(conn_bar)
        root.append(status_lbl)
        root.append(canvas_lbl)
        root.append(canvas_hint)
        root.append(canvas_scroll)
        return root
