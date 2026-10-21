use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    embed_windows_icon();
    embed_build_info();
}

fn embed_windows_icon() {
    println!("cargo:rerun-if-changed=src/assets/app.ico");
    // Natively on Windows or when cross-compiling (winresource then uses
    // x86_64-w64-mingw32-windres).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("src/assets/app.ico")
            .compile()
            .expect("failed to embed the Windows icon");
    }
}

/// Exposes DUPOT_BUILD_DATE / DUPOT_BUILD_PACKAGE to the crate (read by
/// `domain::entity::app_info`). Packaging scripts pass them as env variables.
fn embed_build_info() {
    for var in ["DUPOT_BUILD_DATE", "DUPOT_BUILD_PACKAGE"] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let date = env_value("DUPOT_BUILD_DATE").unwrap_or_else(today);
    let package = env_value("DUPOT_BUILD_PACKAGE").unwrap_or_else(|| "source".into());

    println!("cargo:rustc-env=DUPOT_BUILD_DATE={date}");
    println!("cargo:rustc-env=DUPOT_BUILD_PACKAGE={package}");
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// Today's UTC date as YYYY-MM-DD (no date crate in build scripts).
fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as i64;
    // Civil date from days since 1970-01-01 (H. Hinnant's algorithm).
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}
