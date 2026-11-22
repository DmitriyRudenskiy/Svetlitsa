//! Ручной прогон M0 (тикет 03): окно «Светлица» под headless
//! Wayland-композитором (weston, ADR-0016) со списком позиций.
//!
//! Запуск только в образе ворот: `scripts/smoke/m0-hello.sh` либо
//! `cargo run -p svetlitsa-ui --bin hello --features efl -- --theme
//! /tmp/svetlitsa.edj`. Без фичи `efl` (например, на хосте) бинарь
//! собирается заглушкой.

#[cfg(not(feature = "efl"))]
fn main() -> std::process::ExitCode {
    eprintln!("сборка без фичи `efl`: окно недоступно; гоняй в образе ворот (ADR-0015)");
    std::process::ExitCode::from(2)
}

#[cfg(feature = "efl")]
fn main() -> std::process::ExitCode {
    let mut theme: Option<std::ffi::CString> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--theme" => match args.next() {
                Some(p) => match std::ffi::CString::new(p) {
                    Ok(c) => theme = Some(c),
                    Err(_) => eprintln!("некорректный путь темы (NUL в имени)"),
                },
                None => eprintln!("--theme без пути"),
            },
            other => eprintln!("неизвестный аргумент: {other}"),
        }
    }

    let items: Vec<String> = (1..=4).map(|i| format!("Позиция {i}")).collect();
    match svetlitsa_ui::smoke_window(theme.as_deref(), &items) {
        Ok(r) => {
            println!(
                "окно закрыто; clicked={} del={} eo_cb_main={}",
                r.clicked, r.del_seen, r.callback_in_main_thread
            );
            if r.del_seen {
                ExitCode::SUCCESS
            } else {
                eprintln!("Eo-колбэк удаления не сработал (выход по таймеру?)");
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("не удалось показать окно: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(feature = "efl")]
use std::process::ExitCode;
