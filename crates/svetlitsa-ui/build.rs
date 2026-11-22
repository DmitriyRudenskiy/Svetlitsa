// bin hello линкуется против EFL только с фичей `efl` (образ ворот, ADR-0015).
fn main() {
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_EFL");
    if std::env::var_os("CARGO_FEATURE_EFL").is_none() {
        return;
    }
    // Rust для musl прописывает в PT_INTERP имя загрузчика glibc
    // (/lib/ld-linux-<arch>.so.1), которого в Alpine нет — бинарь гибнет с
    // «not found» при существующем файле. Указываем настоящий musl-загрузчик.
    // Проверяется smoke- прогоном (тикет 03): без этого флага окно не стартует.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("musl") {
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        println!("cargo::rustc-link-arg-bin=hello=-Wl,-dynamic-linker,/lib/ld-musl-{arch}.so.1");
    }
}
