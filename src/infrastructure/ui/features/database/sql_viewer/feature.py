import datetime

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, GLib, Gtk, Pango

from domain.contract.feature_contract import FeatureContract
from infrastructure.api.database_api import DatabaseApi, DatabaseConnectionError

_DB_KINDS = ["sqlite", "postgresql", "mysql"]
_DB_LABELS = ["SQLite", "PostgreSQL", "MySQL"]
_DEFAULT_PORTS = {"postgresql": "5432", "mysql": "3306"}
_MAX_RESULT_ROWS = 200
_LINK_COLOR = (0.22, 0.51, 0.89)


class Feature(FeatureContract):
    def get_widget(self):
        db = DatabaseApi()
        connections = []  # [{"name", "kind", "params"}, ...], mirrors conn_model
        tables_cache = []  # table names on the active connection

        table_cards = []  # [{"table", "frame", "columns": {name: {"check","anchor"}}}]
        joins = []  # [{"left": {...}, "right": {...}}]
        pending_link = [None]  # anchor waiting for its counterpart, or None

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
            label="Right-click the canvas to add a connected table. "
            "Click the link icon on two columns to join them.",
            xalign=0,
        )
        canvas_hint.add_css_class("dim-label")
        canvas_hint.add_css_class("caption")
        canvas_hint.set_wrap(True)

        fixed = Gtk.Fixed()
        fixed.set_size_request(2000, 1200)
        fixed.add_css_class("view")

        link_lines = Gtk.DrawingArea()
        link_lines.set_can_target(False)

        def draw_links(_area, cr, _width, _height, _data=None):
            r, g, b = _LINK_COLOR
            for join in joins:
                left_anchor = join["left"]["anchor"]
                right_anchor = join["right"]["anchor"]
                p1 = left_anchor.translate_coordinates(
                    link_lines, left_anchor.get_width() / 2, left_anchor.get_height() / 2
                )
                p2 = right_anchor.translate_coordinates(
                    link_lines, right_anchor.get_width() / 2, right_anchor.get_height() / 2
                )
                if p1 is None or p2 is None:
                    continue
                x1, y1 = p1
                x2, y2 = p2
                cr.set_source_rgba(r, g, b, 0.9)
                cr.set_line_width(2)
                cr.move_to(x1, y1)
                cr.curve_to((x1 + x2) / 2, y1, (x1 + x2) / 2, y2, x2, y2)
                cr.stroke()
                for px, py in ((x1, y1), (x2, y2)):
                    cr.arc(px, py, 3.5, 0, 2 * 3.14159)
                    cr.fill()

        link_lines.set_draw_func(draw_links)

        canvas_overlay = Gtk.Overlay()
        canvas_overlay.set_child(fixed)
        canvas_overlay.add_overlay(link_lines)

        canvas_scroll = Gtk.ScrolledWindow()
        canvas_scroll.add_css_class("card")
        canvas_scroll.set_min_content_height(420)
        canvas_scroll.set_vexpand(True)
        canvas_scroll.set_hexpand(True)
        canvas_scroll.set_child(canvas_overlay)

        # --- Joins panel ---
        joins_lbl = Gtk.Label(label="Joins", xalign=0)
        joins_lbl.add_css_class("heading")

        joins_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)

        joins_empty_lbl = Gtk.Label(
            label="No joins yet. Link a column from each table to combine them.",
            xalign=0,
        )
        joins_empty_lbl.add_css_class("dim-label")
        joins_empty_lbl.add_css_class("caption")

        def refresh_joins_panel():
            child = joins_box.get_first_child()
            while child is not None:
                nxt = child.get_next_sibling()
                joins_box.remove(child)
                child = nxt
            if not joins:
                joins_box.append(joins_empty_lbl)
                return
            for join in joins:
                left, right = join["left"], join["right"]
                row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
                text = f"{left['table']}.{left['col']}  =  {right['table']}.{right['col']}"
                lbl = Gtk.Label(label=text, xalign=0)
                lbl.set_hexpand(True)
                remove_btn = Gtk.Button()
                remove_btn.set_icon_name("window-close-symbolic")
                remove_btn.add_css_class("flat")
                remove_btn.add_css_class("circular")
                remove_btn.connect("clicked", lambda _b, j=join: remove_join(j))
                row.append(lbl)
                row.append(remove_btn)
                joins_box.append(row)

        def remove_join(join):
            if join in joins:
                joins.remove(join)
            join["left"]["anchor"].set_active(False)
            join["right"]["anchor"].set_active(False)
            refresh_joins_panel()
            link_lines.queue_draw()

        refresh_joins_panel()

        # --- Query execution ---
        run_btn = Gtk.Button(label="Run Query")
        run_btn.add_css_class("suggested-action")
        run_btn.set_halign(Gtk.Align.START)

        sql_lbl = Gtk.Label(label="", xalign=0)
        sql_lbl.add_css_class("dim-label")
        sql_lbl.add_css_class("caption")
        sql_lbl.set_wrap(True)
        sql_lbl.set_selectable(True)

        results_lbl = Gtk.Label(label="Results", xalign=0)
        results_lbl.add_css_class("heading")

        results_scroll = Gtk.ScrolledWindow()
        results_scroll.add_css_class("card")
        results_scroll.set_min_content_height(160)
        results_scroll.set_vexpand(True)
        results_scroll.set_hexpand(True)

        def make_results_placeholder(text):
            lbl = Gtk.Label(label=text)
            lbl.add_css_class("dim-label")
            lbl.set_margin_top(24)
            lbl.set_margin_bottom(24)
            return lbl

        results_scroll.set_child(make_results_placeholder("Run the query to see results here."))

        def quote_ident(name):
            return f"`{name}`" if db.kind == "mysql" else f'"{name}"'

        def build_query():
            if not table_cards:
                return None, "Add at least one table to the canvas first."

            aliases = {id(card): f"t{i}" for i, card in enumerate(table_cards, start=1)}

            select_parts = []
            for card in table_cards:
                alias = aliases[id(card)]
                for col_name, widgets in card["columns"].items():
                    if widgets["check"].get_active():
                        select_parts.append(
                            f"{alias}.{quote_ident(col_name)} AS "
                            f'"{alias}_{col_name}"'
                        )
            if not select_parts:
                return None, "Check at least one column to select."

            base_card = table_cards[0]
            base_alias = aliases[id(base_card)]
            from_clause = f"{quote_ident(base_card['table'])} AS {base_alias}"

            joined_ids = {id(base_card)}
            remaining = list(joins)
            join_clauses = []
            progress = True
            while remaining and progress:
                progress = False
                for join in remaining[:]:
                    left, right = join["left"], join["right"]
                    if id(left["card"]) in joined_ids and id(right["card"]) not in joined_ids:
                        old_side, new_side = left, right
                    elif id(right["card"]) in joined_ids and id(left["card"]) not in joined_ids:
                        old_side, new_side = right, left
                    else:
                        continue
                    new_alias = aliases[id(new_side["card"])]
                    old_alias = aliases[id(old_side["card"])]
                    join_clauses.append(
                        f"INNER JOIN {quote_ident(new_side['card']['table'])} AS {new_alias} "
                        f"ON {old_alias}.{quote_ident(old_side['col'])} = "
                        f"{new_alias}.{quote_ident(new_side['col'])}"
                    )
                    joined_ids.add(id(new_side["card"]))
                    remaining.remove(join)
                    progress = True

            unreached = [c for c in table_cards if id(c) not in joined_ids]
            if unreached:
                names = ", ".join(c["table"] for c in unreached)
                return None, f"Link a column to connect: {names}"

            sql = "SELECT " + ", ".join(select_parts) + " FROM " + from_clause
            if join_clauses:
                sql += " " + " ".join(join_clauses)
            sql += f" LIMIT {_MAX_RESULT_ROWS}"
            return sql, None

        def render_results(columns, rows):
            if not columns:
                results_scroll.set_child(make_results_placeholder("Query executed — no rows."))
                return

            grid = Gtk.Grid()
            grid.set_row_spacing(1)
            grid.set_column_spacing(1)

            for c_idx, col_name in enumerate(columns):
                lbl = Gtk.Label(label=col_name, xalign=0)
                lbl.add_css_class("heading")
                lbl.set_margin_top(6)
                lbl.set_margin_bottom(6)
                lbl.set_margin_start(10)
                lbl.set_margin_end(10)
                grid.attach(lbl, c_idx, 0, 1, 1)

            for r_idx, row in enumerate(rows, start=1):
                for c_idx, value in enumerate(row):
                    text = "" if value is None else str(value)
                    lbl = Gtk.Label(label=text, xalign=0)
                    lbl.set_selectable(True)
                    lbl.set_max_width_chars(30)
                    lbl.set_ellipsize(Pango.EllipsizeMode.END)
                    lbl.set_margin_top(4)
                    lbl.set_margin_bottom(4)
                    lbl.set_margin_start(10)
                    lbl.set_margin_end(10)
                    grid.attach(lbl, c_idx, r_idx, 1, 1)

            results_scroll.set_child(grid)

        def do_run_query(_btn):
            sql, err = build_query()
            if err:
                set_error(err)
                return
            sql_lbl.set_text(sql)
            try:
                columns, rows = db.execute(sql)
            except Exception as e:
                set_error(str(e))
                return
            render_results(columns, rows)
            truncated = " (truncated)" if len(rows) == _MAX_RESULT_ROWS else ""
            set_status(
                f"{len(rows)} row(s){truncated} — "
                f"{datetime.datetime.now().strftime('%H:%M:%S')}"
            )

        run_btn.connect("clicked", do_run_query)

        # --- Join anchors ---
        def on_anchor_toggled(btn, table_name, col_name, card_info):
            if not btn.get_active():
                to_remove = [
                    j for j in joins if j["left"]["anchor"] is btn or j["right"]["anchor"] is btn
                ]
                for j in to_remove:
                    joins.remove(j)
                    other = j["right"] if j["left"]["anchor"] is btn else j["left"]
                    other["anchor"].set_active(False)
                if pending_link[0] and pending_link[0]["anchor"] is btn:
                    pending_link[0] = None
                if to_remove:
                    refresh_joins_panel()
                    link_lines.queue_draw()
                return

            anchor_info = {
                "card": card_info,
                "table": table_name,
                "col": col_name,
                "anchor": btn,
            }

            if pending_link[0] is None:
                pending_link[0] = anchor_info
                return

            other = pending_link[0]
            if other["card"] is card_info:
                # Linking a table to itself doesn't make sense here — reset.
                other["anchor"].set_active(False)
                btn.set_active(False)
                pending_link[0] = None
                return

            joins.append({"left": other, "right": anchor_info})
            pending_link[0] = None
            refresh_joins_panel()
            link_lines.queue_draw()

        # --- Table card on the canvas ---
        def build_table_card(table_name, columns):
            card_info = {"table": table_name, "frame": None, "columns": {}}

            frame = Gtk.Frame()
            frame.add_css_class("card")
            frame.set_size_request(240, -1)
            card_info["frame"] = frame

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
                    row_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=4)
                    check = Gtk.CheckButton(label=label)
                    check.set_active(True)
                    check.set_hexpand(True)
                    anchor = Gtk.ToggleButton()
                    anchor.set_icon_name("insert-link-symbolic")
                    anchor.add_css_class("flat")
                    anchor.add_css_class("circular")
                    anchor.set_tooltip_text(
                        "Link this column to another table's column to join them"
                    )
                    anchor.connect(
                        "toggled",
                        lambda b, t=table_name, c=col_name: on_anchor_toggled(
                            b, t, c, card_info
                        ),
                    )
                    row_box.append(check)
                    row_box.append(anchor)
                    fields_box.append(row_box)
                    card_info["columns"][col_name] = {"check": check, "anchor": anchor}
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

            def on_close(_b):
                fixed.remove(frame)
                if card_info in table_cards:
                    table_cards.remove(card_info)
                stale = [
                    j
                    for j in joins
                    if j["left"]["card"] is card_info or j["right"]["card"] is card_info
                ]
                for j in stale:
                    joins.remove(j)
                if stale:
                    refresh_joins_panel()
                link_lines.queue_draw()

            close_btn.connect("clicked", on_close)

            start_pos = [0.0, 0.0]
            drag = Gtk.GestureDrag()

            def on_drag_begin(_g, _x, _y):
                start_pos[0], start_pos[1] = fixed.get_child_position(frame)

            def on_drag_update(_g, dx, dy):
                fixed.move(frame, start_pos[0] + dx, start_pos[1] + dy)
                link_lines.queue_draw()

            drag.connect("drag-begin", on_drag_begin)
            drag.connect("drag-update", on_drag_update)
            header.add_controller(drag)

            return card_info

        def add_table_card(table_name, x, y):
            try:
                columns = db.list_columns(table_name)
            except Exception as e:
                set_error(str(e))
                return
            card_info = build_table_card(table_name, columns)
            table_cards.append(card_info)
            fixed.put(card_info["frame"], x, y)
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
        root.append(joins_lbl)
        root.append(joins_box)
        root.append(run_btn)
        root.append(sql_lbl)
        root.append(results_lbl)
        root.append(results_scroll)
        return root
