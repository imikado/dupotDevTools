<div align="center">

<img src="src/assets/logo192.png" alt="Dupot Dev Tools" width="128" />

# Dupot Dev Tools

**The developer's Swiss Army knife for the Linux desktop.**

Format, convert, encode, decode, hash… and explore your SQL databases visually.
All in one native, fast, offline GTK4 / Libadwaita application.

![Python](https://img.shields.io/badge/Python-3.13-3776AB?logo=python&logoColor=white)
![GTK4](https://img.shields.io/badge/GTK-4-4A86CF?logo=gtk&logoColor=white)
![Libadwaita](https://img.shields.io/badge/Libadwaita-1-3584E4)
![License](https://img.shields.io/badge/License-LGPL--2.1-green)

![SQL Viewer](export/screenshots/sqlviewer_joins.png)

</div>

---

## ✨ Why Dupot Dev Tools?

How many times a day do you paste a JWT, a JSON blob or a SQL query into a random website just to read it?

Dupot Dev Tools gathers all those small everyday utilities in **one desktop app**:

- 🔒 **100% local** — your tokens, data and queries never leave your machine
- ⚡ **Native and lightweight** — Python + GTK4, no Electron, no browser
- 🎨 **Fits your desktop** — Libadwaita UI, light and dark themes
- 🧩 **Modular** — each tool is a self-contained folder; adding one takes minutes

## 🧰 Tools

| Category | Tool | What it does |
|---|---|---|
| 🗄️ **Database** | **SQL Viewer** | Browse SQLite, PostgreSQL and MySQL databases on an interactive canvas |
| 🎨 **Formatting** | JSON | Prettify or minify JSON |
| | SQL | Format SQL queries |
| | XML | Format XML documents |
| 🔄 **Convert** | YAML ↔ JSON | Convert in both directions |
| | HTML → Text | Strip HTML tags to plain text |
| 🔐 **Encoding** | Base64 | Encode / decode Base64 |
| | URL | Percent-encode / decode |
| | HTML entities | Encode / decode HTML entities |
| | JWT | Decode JWT tokens to readable JSON |
| 🔤 **String** | Hash | MD5, SHA-1, SHA-2, SHA-3, BLAKE2b digests |

## 🗄️ Spotlight: SQL Viewer

A visual query builder that lets you understand a database at a glance:

- **Saved connections** to SQLite, PostgreSQL and MySQL
- **Drag tables onto a canvas** as resizable cards
- **Draw joins** by linking columns between tables
- **Pick the columns** you want and get the SQL query generated for you
- **Run it** and view the results instantly

| New connection | Add a table | Query result |
|---|---|---|
| ![](export/screenshots/sqlviewer_new_connection.png) | ![](export/screenshots/sqlviewer_add_table.png) | ![](export/screenshots/sqlviewer_request.png) |

## 🚀 Getting started

### Debian / Ubuntu / Linux Mint

```bash
sudo apt install python3 python3-gi gir1.2-gtk-4.0 gir1.2-adw-1 gir1.2-gtksource-5 \
                 python3-yaml python3-sqlparse python3-html2text
git clone https://github.com/imikado/dupotDevTools.git
cd dupotDevTools
python3 src/main.py
```

Optional, for the SQL Viewer:

```bash
sudo apt install python3-psycopg2 python3-pymysql   # PostgreSQL / MySQL
```

### Nix

```bash
nix develop
python3 src/main.py
```

## 🧩 Add your own tool

Each tool lives in its own folder under `src/infrastructure/ui/features/<category>/<tool>/` and is discovered automatically at startup:

```
features/
└── string/
    └── hash/
        ├── card.json    # description + icon shown on the home page
        ├── feature.py   # the tool's UI
        └── icon.png
```

`card.json`:

```json
{
    "description": "Generate cryptographic hash digests",
    "icon_name": "security-high-symbolic"
}
```

`feature.py`:

```python
from gi.repository import Gtk
from domain.contract.feature_contract import FeatureContract


class Feature(FeatureContract):
    def get_widget(self):
        return Gtk.Label(label="Hello from my new tool!")
```

A new folder = a new category. No registration, no configuration.

## 🤝 Contributing

Ideas, bug reports and pull requests are very welcome!
Got a small tool you use every day? Turn it into a feature and open a PR — that's exactly what this project is built for.

## 📄 License

[LGPL-2.1](LICENSE) — © Michael Bertocchi · [dupot.org](https://www.dupot.org)
