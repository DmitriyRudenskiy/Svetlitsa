//! Ворота проекта (`xtask ci`) — ТЗ 6.3, среда ворот — ТЗ 3.8, ADR-0003.
//! Исполняется в среде ворот. Удалённого CI нет (ADR-0006): прогон здесь и есть прогон.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

/// Путь от корня репозитория: `cargo test` запускается из каталога крейта.
fn p(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."))
        .join(rel)
}

/// Крейты, не наследующие `[workspace.lints]` (ADR-0010). Расширение — только через ADR.
const LINT_ALLOWLIST: &[&str] = &["svetlitsa-efl"];

#[derive(Debug, PartialEq)]
enum Step {
    Doctor,
    Fmt,
    Clippy,
    Test,
    Deny,
    Lints,
    Licenses,
}

impl Step {
    fn parse(arg: &str) -> Option<Self> {
        match arg {
            "--doctor" => Some(Self::Doctor),
            "fmt" => Some(Self::Fmt),
            "clippy" => Some(Self::Clippy),
            "test" => Some(Self::Test),
            "deny" => Some(Self::Deny),
            "lints" => Some(Self::Lints),
            "licenses" => Some(Self::Licenses),
            _ => None,
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Doctor => "doctor",
            Self::Fmt => "fmt",
            Self::Clippy => "clippy",
            Self::Test => "test",
            Self::Deny => "deny",
            Self::Lints => "lints",
            Self::Licenses => "licenses.csv",
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (cmd, rest) = args
        .split_first()
        .map(|(f, r)| (Some(f.as_str()), r))
        .unwrap_or((None, &[]));
    let parse_all = |rest: &[String]| -> Option<Vec<Step>> {
        rest.iter()
            .map(|a| Step::parse(a))
            .collect::<Option<Vec<Step>>>()
    };
    let step = cmd;
    let all = match step {
        None | Some("ci") => match if rest.is_empty() {
            None
        } else {
            Some(parse_all(rest))
        } {
            Some(Some(steps)) if !steps.is_empty() => steps,
            Some(_) => {
                eprintln!(
                    "неизвестный шаг ворот: {}\nшаги: doctor fmt test clippy deny lints licenses",
                    rest.join(" ")
                );
                exit(2);
            }
            None => vec![
                Step::Doctor,
                Step::Fmt,
                Step::Clippy,
                Step::Test,
                Step::Deny,
                Step::Lints,
                Step::Licenses,
            ],
        },
        Some("parity") => {
            parity(args.get(1).map(String::as_str));
        }
        Some("icons") => {
            icons();
            return;
        }
        Some(other) => match Step::parse(other) {
            Some(s) => vec![s],
            None => {
                eprintln!(
                    "неизвестная задача: {other}\nиспользование: xtask [ci|<шаг>|parity <компонент>|icons]"
                );
                exit(2);
            }
        },
    };
    let mut failed: Vec<&str> = Vec::new();
    for s in &all {
        println!("\n=== {} ===", s.name());
        if let Err(e) = run_step(s) {
            eprintln!("--- {e}");
            failed.push(s.name());
            // Падение doctor'а останавливает итерацию до сборки (ТЗ 6.3): красные ворота
            // от среды нельзя «починить» правкой теста.
            if *s == Step::Doctor {
                break;
            }
        }
    }
    if failed.is_empty() {
        println!("\nворота зелёные");
    } else {
        eprintln!("\nкрасные ворота: {}", failed.join(", "));
        exit(1);
    }
}

fn run_step(step: &Step) -> Result<(), String> {
    match step {
        Step::Doctor => doctor(),
        Step::Fmt => cargo(&["fmt", "--all", "--check"]),
        Step::Clippy => cargo(&[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ]),
        Step::Test => cargo(&["test", "--workspace"]),
        Step::Deny => {
            if which("cargo").is_none() || !has_subcommand("deny") {
                println!(
                    "cargo-deny не установлен — шаг пропущен (ставится в среду ворот, ТЗ 3.8)"
                );
                return Ok(());
            }
            cargo(&["deny", "check"])
        }
        Step::Lints => check_lint_inheritance(),
        Step::Licenses => check_licenses_csv(),
    }
}

/// Окружение среды ворот (ТЗ 3.8). Падение останавливает итерацию до сборки: нехватка
/// RAM/места — проблема среды, а не кода, и правкой теста не лечится.
fn doctor() -> Result<(), String> {
    let mut problems = Vec::new();
    let want = std::fs::read_to_string(p("rust-toolchain.toml"))
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.strip_prefix("channel = ")
                    .map(|v| v.trim_matches('"').to_string())
            })
        })
        .ok_or("rust-toolchain.toml: нет channel")?;
    let out = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| format!("rustc не найден: {e}"))?;
    let have = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !have.contains(&want) {
        problems.push(format!("rustc: ожидается {want}, найден «{have}»"));
    }
    for (bin, why) in [
        ("git", "зеркала и origin"),
        ("rsync", "перенос дерева в среду сборки"),
    ] {
        if which(bin).is_none() {
            problems.push(format!("{bin} не найден — {why}"));
        }
    }
    // EFL/Elementary и Xvfb нужны только сборке UI и smoke-прогонам (ТЗ 3.2, 10.3).
    if pkg_config_version("elementary").is_none() {
        println!(
            "подсказка: Elementary 1.29 по pkg-config не найден — окно и smoke не собрать (scripts/dev-vm/efl-build.sh)"
        );
    }
    if which("Xvfb").is_none() {
        println!("подсказка: Xvfb не найден — smoke-прогоны и скриншоты не запустить (ТЗ 10.2)");
    }
    for (bin, why) in [
        ("xdotool", "ввод в smoke-сценариях"),
        ("import", "снимки экрана в smoke-сценариях"),
    ] {
        if which(bin).is_none() {
            println!("подсказка: {bin} не найден — {why} (ТЗ 10.2, ADR-0011)");
        }
    }
    let free = free_gib(&p(".").display().to_string())?;
    if free < 60.0 {
        problems.push(format!(
            "свободно {free:.1} ГиБ — сборка EFL и target/ не помещаются (ТЗ 3.8: нужно ≥ 60 ГиБ)"
        ));
    }
    if problems.is_empty() {
        println!("окружение в порядке ({have})");
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

fn check_lint_inheritance() -> Result<(), String> {
    let outside = crates_outside_lints()?;
    let allowed: BTreeSet<&str> = LINT_ALLOWLIST.iter().copied().collect();
    let found: BTreeSet<&str> = outside.iter().map(String::as_str).collect();
    if found != allowed {
        return Err(format!(
            "крейты вне [lints] workspace != allowlist: найдено {}, разрешено {allowed:?}",
            found
                .iter()
                .map(|s| format!("«{s}»"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    println!("вне [lints] workspace: {found:?}");
    Ok(())
}

fn crates_outside_lints() -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let dir = &p("crates");
    let entries = std::fs::read_dir(dir).map_err(|e| format!("crates/: {e}"))?;
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    for name in names {
        let toml = std::fs::read_to_string(p(&format!("crates/{name}/Cargo.toml")))
            .map_err(|e| format!("{name}/Cargo.toml: {e}"))?;
        if !toml.contains("[lints]\nworkspace = true") {
            out.push(name);
        }
    }
    Ok(out)
}

/// `docs/licenses.csv` коммитится и генерируется на машине с зеркалами (ADR-0002).
/// В воротах проверяется сам файл: есть, схема читается, каждый крейт упомянут.
fn check_licenses_csv() -> Result<(), String> {
    let csv = std::fs::read_to_string(p("docs/licenses.csv")).map_err(|_| {
        "docs/licenses.csv отсутствует — сгенерируйте `xtask licenses` на машине с vendor/mate"
            .to_string()
    })?;
    let mut lines = csv.lines();
    let header = lines.next().unwrap_or_default();
    if !header.starts_with("crate,") {
        return Err(format!("docs/licenses.csv: неожиданная шапка «{header}»"));
    }
    let listed: BTreeSet<String> = lines
        .filter_map(|l| l.split(',').next())
        .map(|s| s.trim().to_string())
        .collect();
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(p("crates")).map_err(|e| format!("crates/: {e}"))? {
        let name = entry
            .ok()
            .filter(|e| e.path().join("Cargo.toml").is_file())
            .map(|e| e.file_name().to_string_lossy().to_string());
        if let Some(name) = name
            && !listed.contains(&name)
        {
            missing.push(name);
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "в docs/licenses.csv нет строк для: {}",
            missing.join(", ")
        ));
    }
    println!(
        "docs/licenses.csv: {} записей, все крейты упомянуты",
        listed.len()
    );
    Ok(())
}

/// Закрытый список иконок М2 (ADR-0012): имена из зеркал М2-компонентов, которые реально
/// существуют в `mate-icon-theme`. Заготовка — человек сокращает, молча не растёт.
fn icons() {
    let m2 = [
        "mate-panel",
        "marco",
        "mate-screensaver",
        "mate-session-manager",
        "mate-settings-daemon",
        "mate-desktop",
        "mate-notification-daemon",
    ];
    let theme = p("vendor/mate/mate-icon-theme");
    if !theme.is_dir() {
        eprintln!("vendor/mate/mate-icon-theme нет — сначала scripts/fetch-vendor.sh");
        exit(1);
    }
    let mut known: BTreeSet<String> = BTreeSet::new();
    walk(&theme, &mut |path| {
        if let Some(stem) = stem(path, "png") {
            known.insert(stem);
        }
    });
    let mut used: BTreeSet<String> = BTreeSet::new();
    for repo in m2 {
        walk(&p(&format!("vendor/mate/{repo}")), &mut |path| {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            for token in icon_tokens(&text) {
                if known.contains(&token) {
                    used.insert(token);
                }
            }
        });
    }
    let out_path = p("docs/parity/svetlitsa-ui.md");
    let mut out = String::from(
        "# Закрытые списки `svetlitsa-ui`\n\nЗаготовка `xtask icons`: имена иконок из зеркал М2-компонентов, \nкоторые есть в `mate-icon-theme`. Пополняется PR'ом, а не молча (ADR-0012).\n\n",
    );
    out.push_str(&format!("Иконок: {}\n\n", used.len()));
    for name in &used {
        out.push_str(&format!("- `{name}`\n"));
    }
    match std::fs::write(&out_path, out) {
        Ok(()) => println!("{}: {} имён иконок", out_path.display(), used.len()),
        Err(e) => {
            eprintln!("{}: {e}", out_path.display());
            exit(1);
        }
    }
}

/// Кандидаты на freedesktop-имя иконки: kebab-case с дефисом, длина ≥ 4.
fn icon_tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
        .filter(|chunk| {
            chunk.len() >= 4
                && chunk.contains('-')
                && !chunk.starts_with('-')
                && !chunk.ends_with('-')
        })
        .map(str::to_string)
        .collect()
}

/// Имя файла без расширения, если расширение подходит.
fn stem(path: &Path, ext: &str) -> Option<String> {
    path.file_name()?
        .to_str()?
        .strip_suffix(&format!(".{ext}"))
        .map(str::to_string)
}

fn walk(dir: &Path, sink: &mut impl FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            walk(&path, sink);
        } else {
            sink(&path);
        }
    }
}

/// Заготовка парити-листа: два уровня объёма (ТЗ 5.1). Модули берутся из имён файлов C-оригинала.
fn parity(component: Option<&str>) -> ! {
    let Some(name) = component else {
        eprintln!("использование: xtask parity <компонент>");
        exit(2);
    };
    let src = &p("vendor/mate");
    if !src.is_dir() {
        eprintln!(
            "vendor/mate нет — сначала scripts/fetch-vendor.sh (зеркала дешёвые, живут на машине с бюджетом)"
        );
        exit(1);
    }
    println!(
        "заготовка docs/parity/{name}.md появится, когда будет выбран C-оригинал: см. карту ТЗ 2"
    );
    exit(0);
}

fn cargo(args: &[&str]) -> Result<(), String> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .map_err(|e| format!("cargo {}: {e}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo {} завершился неуспешно", args.join(" ")))
    }
}

fn has_subcommand(name: &str) -> bool {
    Command::new("cargo")
        .arg(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|p| p.join(bin))
        .find(|p| p.is_file())
}

fn pkg_config_version(package: &str) -> Option<String> {
    let out = Command::new("pkg-config")
        .args(["--modversion", package])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn free_gib(path: &str) -> Result<f64, String> {
    let out = Command::new("df")
        .args(["-kP", path])
        .output()
        .map_err(|e| format!("df: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let avail = text
        .lines()
        .nth(1)
        .and_then(|l| l.split_whitespace().rev().nth(2))
        .and_then(|v| v.parse::<f64>().ok())
        .ok_or("df: не разобрать свободное место")?;
    Ok(avail / 1024.0 / 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allowlist ADR-0010 совпадает с деревом: единственный вне наследования — svetlitsa-efl.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn allowlist_sovpadaet_s_derevom() {
        assert_eq!(
            crates_outside_lints().unwrap(),
            vec!["svetlitsa-efl".to_string()]
        );
    }
}
