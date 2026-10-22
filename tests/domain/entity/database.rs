use dupot_dev_tools::domain::entity::database::{ConnectionForm, DbKind};

#[test]
fn default_ports() {
    assert_eq!(DbKind::Sqlite.default_port(), "");
    assert_eq!(DbKind::Postgresql.default_port(), "5432");
    assert_eq!(DbKind::Mysql.default_port(), "3306");
}

#[test]
fn sqlite_form_requires_a_path() {
    let err = ConnectionForm::default()
        .into_config(DbKind::Sqlite, 0)
        .unwrap_err();
    assert_eq!(err.message(), "A database file path is required.");
}

#[test]
fn sqlite_form_keeps_trimmed_path_and_default_name() {
    let form = ConnectionForm {
        path: "  /tmp/app.db ".into(),
        ..Default::default()
    };
    let config = form.into_config(DbKind::Sqlite, 2).unwrap();
    assert_eq!(config.name, "connection-3");
    assert_eq!(config.param("path"), "/tmp/app.db");
    assert_eq!(config.param("host"), "");
}

#[test]
fn server_form_fills_defaults() {
    let form = ConnectionForm {
        name: " prod ".into(),
        user: " admin ".into(),
        password: " secret ".into(),
        ..Default::default()
    };
    let config = form.into_config(DbKind::Mysql, 0).unwrap();
    assert_eq!(config.name, "prod");
    assert_eq!(config.host(), "localhost");
    assert_eq!(config.port().unwrap(), 3306);
    assert_eq!(config.param("user"), "admin");
    assert_eq!(
        config.param("password"),
        " secret ",
        "passwords are not trimmed"
    );
}

#[test]
fn invalid_port_is_reported() {
    let form = ConnectionForm {
        port: "abc".into(),
        ..Default::default()
    };
    let config = form.into_config(DbKind::Postgresql, 0).unwrap();
    assert_eq!(config.port().unwrap_err().message(), "Invalid port: abc");
}

#[test]
fn kind_ids_round_trip() {
    for kind in DbKind::ALL {
        assert_eq!(DbKind::from_id(kind.id()), Some(kind));
    }
    assert_eq!(DbKind::from_id("oracle"), None);
}

#[test]
fn password_is_needed_only_for_servers_without_one() {
    let sqlite = ConnectionForm {
        path: "/tmp/app.db".into(),
        ..Default::default()
    }
    .into_config(DbKind::Sqlite, 0)
    .unwrap();
    assert!(!sqlite.needs_password());

    let mut pg = ConnectionForm::default()
        .into_config(DbKind::Postgresql, 0)
        .unwrap();
    assert!(
        !pg.needs_password(),
        "an empty password typed in the form counts"
    );
    pg.forget_password();
    assert!(pg.needs_password());
    pg.set_password("secret".into());
    assert_eq!(pg.param("password"), "secret");
    assert!(!pg.needs_password());
}

#[test]
fn persisted_params_exclude_the_password() {
    let config = ConnectionForm {
        user: "bob".into(),
        password: "secret".into(),
        ..Default::default()
    }
    .into_config(DbKind::Postgresql, 0)
    .unwrap();
    let keys: Vec<_> = config.persisted_params().map(|(k, _)| k).collect();
    assert_eq!(keys, ["database", "host", "port", "user"]);
}
