// Пути EFL задаёт только сборка с фичей `efl` (образ ворот): на хосте без EFL
// крейт собирается заглушкой и никого не линкует.
fn main() {
    println!("cargo::rerun-if-env-changed=EFL_PREFIX");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_EFL");
    if std::env::var_os("CARGO_FEATURE_EFL").is_none() {
        return;
    }
    let prefix = std::env::var("EFL_PREFIX").unwrap_or_else(|_| "/opt/svetlitsa-efl".into());
    println!("cargo:rustc-link-search=native={prefix}/lib");
    for lib in [
        "elementary",
        "edje",
        "efreet",
        "eet",
        "ecore",
        "evas",
        "eio",
        "efl",
        "eo",
        "eina",
    ] {
        println!("cargo:rustc-link-lib=dylib={lib}");
    }
}
