use chrono::Utc;
use dupot_dev_tools::domain::feature::string::uuid_generator::{
    IdFormat, IdKind, MAX_COUNT, format_uuid, generate, inspect,
};
use uuid::Uuid;

#[test]
fn generates_the_requested_count() {
    assert_eq!(
        generate(IdKind::UuidV4, 5, IdFormat::default())
            .lines()
            .count(),
        5
    );
    assert_eq!(
        generate(IdKind::UuidV4, 0, IdFormat::default())
            .lines()
            .count(),
        1
    );
    assert_eq!(
        generate(IdKind::UuidV4, MAX_COUNT + 10, IdFormat::default())
            .lines()
            .count(),
        MAX_COUNT
    );
}

#[test]
fn generated_uuids_have_the_right_version_and_are_unique() {
    let out = generate(IdKind::UuidV7, 50, IdFormat::default());
    let ids: Vec<Uuid> = out.lines().map(|l| Uuid::parse_str(l).unwrap()).collect();
    assert!(ids.iter().all(|id| id.get_version_num() == 7));
    let mut unique = ids.clone();
    unique.dedup();
    assert_eq!(unique.len(), ids.len());
    let v4 = generate(IdKind::UuidV4, 1, IdFormat::default());
    assert_eq!(Uuid::parse_str(&v4).unwrap().get_version_num(), 4);
}

#[test]
fn uuid_formats() {
    let id = Uuid::parse_str("a1b2c3d4-e5f6-4789-8abc-def012345678").unwrap();
    assert_eq!(
        format_uuid(id, IdFormat::default()),
        "a1b2c3d4-e5f6-4789-8abc-def012345678"
    );
    let all = IdFormat {
        uppercase: true,
        no_hyphens: true,
        braces: true,
    };
    assert_eq!(format_uuid(id, all), "{A1B2C3D4E5F647898ABCDEF012345678}");
}

#[test]
fn ulids_are_26_chars_and_case_follows_option() {
    let lower = generate(IdKind::Ulid, 1, IdFormat::default());
    assert_eq!(lower.len(), 26);
    assert_eq!(lower, lower.to_lowercase());
    let upper = generate(
        IdKind::Ulid,
        1,
        IdFormat {
            uppercase: true,
            ..Default::default()
        },
    );
    assert_eq!(upper, upper.to_uppercase());
}

#[test]
fn nil_uuid() {
    assert_eq!(
        generate(IdKind::Nil, 1, IdFormat::default()),
        "00000000-0000-0000-0000-000000000000"
    );
}

#[test]
fn inspects_a_v4_in_any_notation() {
    for input in [
        "a1b2c3d4-e5f6-4789-8abc-def012345678",
        "{A1B2C3D4-E5F6-4789-8ABC-DEF012345678}",
        "a1b2c3d4e5f647898abcdef012345678",
    ] {
        let out = inspect(input, &Utc).unwrap();
        assert!(out.contains("Version    4 — random"), "{input}");
        assert!(out.contains("Canonical  a1b2c3d4-e5f6-4789-8abc-def012345678"));
        assert!(!out.contains("Created"));
    }
}

#[test]
fn inspects_v7_creation_time() {
    // 0x0192_B09E_0900 ms = 2024-10-21T19:46:40Z
    let out = inspect("0192b09e-0900-7000-8000-000000000000", &Utc).unwrap();
    assert!(out.contains("Version    7 — time-ordered (Unix)"));
    assert!(out.contains("Created    2024-10-21 19:46:40.000 UTC"));
}

#[test]
fn inspects_ulid() {
    let out = inspect("01JAR9W2800000000000000000", &Utc).unwrap();
    assert!(out.contains("Type       ULID"));
    assert!(out.contains("Created    2024-10-21 19:46:40.000 UTC"));
}

#[test]
fn rejects_invalid_ids() {
    assert!(inspect("", &Utc).is_err());
    assert!(
        inspect("not-a-uuid", &Utc)
            .unwrap_err()
            .message()
            .contains("Not a valid")
    );
}
