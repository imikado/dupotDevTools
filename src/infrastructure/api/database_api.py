import sqlite3


class DatabaseConnectionError(Exception):
    pass


class DatabaseApi:
    """Thin wrapper around DB-API connections for SQLite / PostgreSQL / MySQL."""

    def __init__(self):
        self._conn = None
        self._kind = None

    def connect(self, kind: str, **params):
        self.close()

        if kind == "sqlite":
            path = params.get("path", "")
            if not path:
                raise DatabaseConnectionError("A database file path is required.")
            try:
                self._conn = sqlite3.connect(path)
            except sqlite3.Error as e:
                raise DatabaseConnectionError(str(e)) from e

        elif kind == "postgresql":
            try:
                import psycopg2
            except ImportError as e:
                raise DatabaseConnectionError(
                    "psycopg2 is not installed. Add it to the Python environment "
                    "to connect to PostgreSQL."
                ) from e
            try:
                self._conn = psycopg2.connect(
                    host=params.get("host") or "localhost",
                    port=int(params.get("port") or 5432),
                    dbname=params.get("database", ""),
                    user=params.get("user", ""),
                    password=params.get("password", ""),
                )
            except psycopg2.Error as e:
                raise DatabaseConnectionError(str(e)) from e

        elif kind == "mysql":
            try:
                import pymysql
            except ImportError as e:
                raise DatabaseConnectionError(
                    "pymysql is not installed. Add it to the Python environment "
                    "to connect to MySQL."
                ) from e
            try:
                self._conn = pymysql.connect(
                    host=params.get("host") or "localhost",
                    port=int(params.get("port") or 3306),
                    database=params.get("database", ""),
                    user=params.get("user", ""),
                    password=params.get("password", ""),
                )
            except pymysql.MySQLError as e:
                raise DatabaseConnectionError(str(e)) from e

        else:
            raise DatabaseConnectionError(f"Unknown database type: {kind}")

        self._kind = kind

    def is_connected(self) -> bool:
        return self._conn is not None

    def list_tables(self) -> list[str]:
        if not self._conn:
            return []
        cur = self._conn.cursor()
        try:
            if self._kind == "sqlite":
                cur.execute(
                    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
                )
            elif self._kind == "postgresql":
                cur.execute(
                    "SELECT table_name FROM information_schema.tables "
                    "WHERE table_schema='public' ORDER BY table_name"
                )
            elif self._kind == "mysql":
                cur.execute("SHOW TABLES")
            return [row[0] for row in cur.fetchall()]
        finally:
            cur.close()

    def list_columns(self, table: str) -> list[tuple[str, str]]:
        """Returns [(column_name, column_type), ...] for a table already
        reported by list_tables() (trusted, not free-typed user input)."""
        if not self._conn:
            return []
        cur = self._conn.cursor()
        try:
            if self._kind == "sqlite":
                cur.execute(f"PRAGMA table_info('{table}')")
                return [(row[1], row[2] or "") for row in cur.fetchall()]
            elif self._kind == "postgresql":
                cur.execute(
                    "SELECT column_name, data_type FROM information_schema.columns "
                    "WHERE table_name = %s ORDER BY ordinal_position",
                    (table,),
                )
                return [(row[0], row[1]) for row in cur.fetchall()]
            elif self._kind == "mysql":
                cur.execute(f"SHOW COLUMNS FROM `{table}`")
                return [(row[0], row[1]) for row in cur.fetchall()]
            return []
        finally:
            cur.close()

    def close(self):
        if self._conn is not None:
            try:
                self._conn.close()
            except Exception:
                pass
            self._conn = None
            self._kind = None
