use dupot_dev_tools::domain::contract::connection_repository::ConnectionRepository;
use dupot_dev_tools::domain::entity::database::{ConnectionForm, DbKind};
use dupot_dev_tools::infrastructure::api::connection_store_api::ConnectionStoreApi;

fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("dupot-store-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn missing_file_is_an_error() {
    let dir = temp_dir("missing");
    let store = ConnectionStoreApi::new(dir.join("state"));
    assert!(store.load(&dir.join("nope.json")).is_err());
}

#[test]
fn saves_and_reloads_without_password() {
    let dir = temp_dir("round-trip");
    let store = ConnectionStoreApi::new(dir.join("state"));
    let file = dir.join("sub").join("connections.json");
    let pg = ConnectionForm {
        name: "local pg".into(),
        database: "devtools".into(),
        user: "devtools".into(),
        password: "secret".into(),
        ..Default::default()
    }
    .into_config(DbKind::Postgresql, 0)
    .unwrap();
    let sqlite = ConnectionForm {
        path: "/tmp/app.db".into(),
        ..Default::default()
    }
    .into_config(DbKind::Sqlite, 1)
    .unwrap();

    store.save(&file, &[pg, sqlite.clone()]).unwrap();
    assert!(!std::fs::read_to_string(&file).unwrap().contains("secret"));

    let loaded = store.load(&file).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "local pg");
    assert_eq!(loaded[0].kind, DbKind::Postgresql);
    assert_eq!(loaded[0].param("database"), "devtools");
    assert_eq!(loaded[0].param("port"), "5432");
    assert!(loaded[0].needs_password());
    assert_eq!(loaded[1], sqlite);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn remembers_the_last_existing_file() {
    let dir = temp_dir("last");
    let store = ConnectionStoreApi::new(dir.join("state"));
    assert_eq!(store.last_path(), None);

    let file = dir.join("mine.json");
    store.save(&file, &[]).unwrap();
    store.set_last_path(&file).unwrap();
    assert_eq!(store.last_path(), Some(file.clone()));

    std::fs::remove_file(&file).unwrap();
    assert_eq!(store.last_path(), None, "a deleted file is not reopened");
    let _ = std::fs::remove_dir_all(dir);
}
