//! Ворота проекта (`xtask ci`) — ТЗ 6.3, среда ворот — ТЗ 3.8, ADR-0003.
//! Исполняется в среде ворот. Удалённого CI нет (ADR-0006): прогон здесь и есть прогон.

use std::collections::{BTreeMap, BTreeSet};
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
    Parity,
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
            "parity" => Some(Self::Parity),
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
            Self::Parity => "parity-листы",
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
                    "неизвестный шаг ворот: {}\nшаги: doctor fmt test clippy deny lints licenses parity",
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
                Step::Parity,
            ],
        },
        Some("parity") if args.get(1).map(String::as_str) != Some("--check") => {
            if let Err(e) = parity(args.get(1).map(String::as_str)) {
                eprintln!("{e}");
                exit(1);
            }
            return;
        }
        Some("parity") => {
            if let Err(e) = check_parity_sheets() {
                eprintln!("{e}");
                exit(1);
            }
            return;
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
        Step::Parity => check_parity_sheets(),
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
    let mut generated = format!(
        "{GEN_BEGIN}\n\nИконок: {} — имена из зеркал М2-компонентов, которые есть в \n`mate-icon-theme` (ADR-0012). Пополняется PR'ом, а не молча.\n\n",
        used.len()
    );
    for name in &used {
        generated.push_str(&format!("- `{name}`\n"));
    }
    generated.push('\n');
    generated.push_str(GEN_END);
    // Тот же двухслойный формат, что у парити-листов: список виджетов М2 (Q36) — рукописный,
    // перегенерация иконок не имеет права его стирать.
    let existing = std::fs::read_to_string(&out_path).ok();
    let text = match splice(
        "# Закрытые списки `svetlitsa-ui`",
        &generated,
        existing.as_deref(),
        WIDGET_STUB,
    ) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{}: {e}", out_path.display());
            exit(1);
        }
    };
    match std::fs::write(&out_path, text) {
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

/// Два уровня парити (ТЗ 5.1, Q47): генератор фактичит, вердикты пишет человек. Границы
/// блоков — единственное, что он читает в уже существующем листе.
const GEN_BEGIN: &str = "<!-- BEGIN GENERATOR (xtask parity) — руками не править -->";
const GEN_END: &str = "<!-- END GENERATOR -->";

/// Каталоги зеркала, которые не являются переносимыми модулями: переводы, доки, данные,
/// автотесты оригинала.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".github",
    "po",
    "help",
    "doc",
    "docs",
    "m4",
    "build",
    "subprojects",
    "man",
    "its",
    "data",
    "icons",
    "pixmaps",
    "test",
    "tests",
];

/// Строка карты ТЗ 2 / 2.2: репозиторий → крейт → майлстоун → уровень парити.
#[derive(Clone)]
struct MapRow {
    repo: String,
    kreyt: String,
    maylston: String,
    level: Level,
}

/// Уровень парити (ТЗ 5.1): модули — для всех; функции — только для публичной поверхности
/// библиотеки; «свой API» — когда оригинал не устанавливает заголовков.
#[derive(Clone)]
enum Level {
    Modules,
    Functions(String),
    OwnApi,
}

fn parse_level(text: &str) -> Level {
    if let Some(dir) = text.strip_prefix("функции:") {
        Level::Functions(dir.trim().to_string())
    } else if text == "свой API" {
        Level::OwnApi
    } else {
        Level::Modules
    }
}

/// Карта ТЗ 2 — единственный источник соответствия «репозиторий ↔ крейт»; второй копии нет.
fn load_map() -> Result<Vec<MapRow>, String> {
    let text = std::fs::read_to_string(p("TASK.md")).map_err(|e| format!("TASK.md: {e}"))?;
    let mut inside = false;
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.starts_with("## ") {
            inside = line.starts_with("## 2. ");
            continue;
        }
        if !inside {
            continue;
        }
        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        if cols.len() < 7 || cols[1].parse::<u32>().is_err() {
            continue;
        }
        let repo = cols[2].trim_matches('`');
        let kreyt = cols[5]
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_matches('*');
        if repo.is_empty() || kreyt.is_empty() || kreyt == "—" {
            continue;
        }
        rows.push(MapRow {
            repo: repo.to_string(),
            kreyt: kreyt.to_string(),
            maylston: cols[6].to_string(),
            level: parse_level(cols.get(7).copied().unwrap_or("модули")),
        });
    }
    if rows.is_empty() {
        return Err("TASK.md: карта ТЗ 2 не разобралась — изменился формат таблицы?".to_string());
    }
    Ok(rows)
}

/// Отпечаток зеркала: короткий sha HEAD shallow-клона (Q47.3 — лист обязан ему соответствовать).
fn mirror_head(repo: &str) -> Option<String> {
    let dir = p(&format!("vendor/mate/{repo}"));
    let out = Command::new("git")
        .args([
            "-C",
            dir.to_string_lossy().as_ref(),
            "rev-parse",
            "--short",
            "HEAD",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Модули C-оригинала: каталог → имена `.c`-файлов. Один `.c` = один модуль (ТЗ 5.1).
fn modules_of(root: &Path) -> BTreeMap<String, Vec<String>> {
    let mut by_dir: BTreeMap<String, Vec<String>> = BTreeMap::new();
    walk_filtered(root, &mut |path| {
        if path.extension().and_then(|e| e.to_str()) != Some("c") {
            return;
        }
        let Some(file) = path.file_name().and_then(|n| n.to_str()) else {
            return;
        };
        // Автотесты оригинала — не переносимые модули, даже когда лежат рядом с кодом.
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if stem.starts_with("test") || stem.ends_with("-test") || stem.ends_with("_test") {
            return;
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        let dir = match rel.parent().and_then(|d| d.to_str()) {
            Some(d) if !d.is_empty() => d.to_string(),
            _ => ".".to_string(),
        };
        by_dir.entry(dir).or_default().push(file.to_string());
    });
    for files in by_dir.values_mut() {
        files.sort();
    }
    by_dir
}

/// Обход без каталогов из `SKIP_DIRS` (остальное — как `walk`).
fn walk_filtered(dir: &Path, sink: &mut impl FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| SKIP_DIRS.contains(&n))
            {
                continue;
            }
            walk_filtered(&path, sink);
        } else {
            sink(&path);
        }
    }
}

/// Публичные заголовки каталога: имена, перечисленные в билд-файлах (meson — в кавычках,
/// autotools — голыми токенами в `*_HEADERS`). Если билд-система ничего не перечисляет
/// (так делает libmateweather), берётся каталог целиком без `private` — и лист это пишет.
/// Второй элемент кортежа — true, когда сработал запасной путь.
fn public_headers(dir: &Path) -> (Vec<String>, bool) {
    let mut mentioned: BTreeSet<String> = BTreeSet::new();
    let mut present: BTreeSet<String> = BTreeSet::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (Vec::new(), false);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if name.ends_with(".h") {
            present.insert(name);
        } else if name.starts_with("Makefile.am") || name == "meson.build" {
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            mentioned.extend(header_tokens(&text));
        }
    }
    let public = |name: &String| !name.contains("private");
    let fallback = mentioned.is_empty();
    let chosen: Vec<String> = if fallback {
        present.iter().filter(|h| public(h)).cloned().collect()
    } else {
        mentioned
            .iter()
            .filter(|h| present.contains(*h) && public(h))
            .cloned()
            .collect()
    };
    (chosen, fallback)
}

/// Имена заголовков, упомянутые в билд-файле (без каталога: путь берётся из самого листа).
fn header_tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || "-_.+".contains(c)))
        .filter(|tok| tok.ends_with(".h") && tok.len() > 3)
        .map(|tok| tok.rsplit('/').next().unwrap_or(tok).to_string())
        .collect()
}

/// Объявления нестатических функций заголовка: комментарии и макросы вырезаются, объявление
/// собирается до `;` (многострочный прототип — одно объявление), имя — идентификатор перед `(`.
fn header_functions(text: &str) -> Vec<String> {
    let mut kept = String::new();
    for line in strip_c_comments(text).lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("typedef")
            || line.starts_with("G_")
            || line.starts_with("G_DECLARE")
            || line.contains("static")
            || regex_macro_line(line)
        {
            continue;
        }
        kept.push_str(line);
        kept.push(' ');
    }
    let mut out = Vec::new();
    for stmt in kept.split(';') {
        let Some(open) = stmt.find('(') else { continue };
        // `void (*cb)(...)` — член-указатель на функцию, не объявление
        if stmt[open + 1..].trim_start().starts_with('*') {
            continue;
        }
        let name: String = stmt[..open]
            .trim_end()
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .iter()
            .rev()
            .collect();
        let is_name = name.len() > 1
            && !name.chars().next().is_some_and(|c| c.is_ascii_digit())
            && name != name.to_uppercase()
            && !matches!(
                name.as_str(),
                "void"
                    | "int"
                    | "char"
                    | "long"
                    | "short"
                    | "float"
                    | "double"
                    | "unsigned"
                    | "signed"
                    | "struct"
                    | "enum"
                    | "union"
                    | "if"
                    | "for"
                    | "while"
                    | "return"
                    | "sizeof"
            );
        if is_name && !out.contains(&name) {
            out.push(name);
        }
    }
    out.sort();
    out
}

/// Строка вида `FAKE_AVAILABLE (2,0)` или `G_DEPRECATED_FOR(g)` — макрос, не объявление.
fn regex_macro_line(line: &str) -> bool {
    let head: String = line
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    !head.is_empty() && head == head.to_uppercase()
}

/// ponytail: состояния нет — `//` внутри строкового литерала заголовка съест строку до конца;
/// в публичных заголовках MATE таких строк не встречалось. Апгрейд: конечный автомат со строками.
fn strip_c_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut block = false;
    let mut lines = text.chars().peekable();
    while let Some(c) = lines.next() {
        if block {
            if c == '*' && lines.peek() == Some(&'/') {
                lines.next();
                block = false;
            }
            continue;
        }
        if c == '/' && lines.peek() == Some(&'*') {
            lines.next();
            block = true;
            continue;
        }
        if c == '/' && lines.peek() == Some(&'/') {
            for n in lines.by_ref() {
                if n == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// Генерируемый блок: отпечаток зеркала + дерево модулей + (для библиотек) публичные функции.
fn render_generated(
    row: &MapRow,
    head: &str,
    modules: &BTreeMap<String, Vec<String>>,
    functions: &[String],
    surface_fallback: bool,
) -> String {
    let total: usize = modules.values().map(Vec::len).sum();
    let mut out = format!("{GEN_BEGIN}\n");
    out.push_str(&format!(
        "\nЗеркало: `mate-desktop/{}` · HEAD `{head}` · модулей (`.c`): {total}\n\n",
        row.repo
    ));
    out.push_str("| Каталог | Модули |\n|---|---|\n");
    for (dir, files) in modules {
        let list: Vec<String> = files.iter().map(|f| format!("`{f}`")).collect();
        out.push_str(&format!("| `{dir}` | {} |\n", list.join(", ")));
    }
    match &row.level {
        Level::Functions(dir) => {
            out.push_str(&format!(
                "\n## Функции ({}) — публичная поверхность `{}`{}\n\n",
                functions.len(),
                dir,
                if surface_fallback {
                    " (билд-система не перечисляет заголовки — взят каталог без private)"
                } else {
                    ""
                }
            ));
            out.push_str(&functions
                .iter()
                .map(|f| format!("`{f}`"))
                .collect::<Vec<_>>()
                .join(", "));
            out.push('\n');
        }
        Level::OwnApi => out.push_str(
            "\n## Функции\n\nОригинал не устанавливает публичных заголовков — C-прототипа поверхности \nнет. Поверхность — собственный Rust API крейта, перечень ведётся руками (правило \nнулевого листа, ТЗ 5.1).\n",
        ),
        Level::Modules => {}
    }
    out.push_str(GEN_END);
    out
}

/// Рукописный блок листа `svetlitsa-ui`: закрытый список виджетов М2 (Q36) — его ведёт человек.
const WIDGET_STUB: &str = "

## Виджеты М2 (ручной блок — генератор его не трогает)

Закрытый список виджетов Elementary, которые обязана покрыть тема «Лубок» к М2 (ADR-0012).
Пополняется PR'ом, а не молча.

| Виджет | Элементы | Тема покрыла |
|---|---|---|
";

/// Рукописный блок при создании листа: вердиктов ещё нет, и генератор их не додумывает.
const HANDWRITTEN_STUB: &str = "\n\n## Вердикты (ручной блок — генератор его не трогает)\n\
Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию\n\
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение\n\
исполнителя.\n\n| Позиция | Тип | Статус | Пометка |\n|---|---|---|---|\n";

/// Собрать лист: заголовок и генерируемый блок свежие, всё после `GEN_END` — как было.
fn splice(
    title: &str,
    generated: &str,
    existing: Option<&str>,
    stub: &str,
) -> Result<String, String> {
    let tail = match existing {
        None => stub.to_string(),
        Some(text) => {
            let Some(pos) = text.find(GEN_END) else {
                return Err(
                    "лист правился вне генератора (нет границ блока) — объедините вручную"
                        .to_string(),
                );
            };
            text[pos + GEN_END.len()..].to_string()
        }
    };
    Ok(format!("{title}\n{generated}{tail}"))
}

/// `xtask parity [крейт]` — перегенерировать фактуру парити-листов (ручной блок сохраняется).
fn parity(component: Option<&str>) -> Result<(), String> {
    if !p("vendor/mate").is_dir() {
        return Err("vendor/mate нет — сначала scripts/fetch-vendor.sh".to_string());
    }
    let mut written = 0;
    for row in load_map()? {
        if component.is_some_and(|c| c != row.kreyt) {
            continue;
        }
        let Some(head) = mirror_head(&row.repo) else {
            return Err(format!(
                "{}: зеркала нет в vendor/mate — scripts/fetch-vendor.sh",
                row.repo
            ));
        };
        let modules = modules_of(&p(&format!("vendor/mate/{}", row.repo)));
        let mut functions: Vec<String> = Vec::new();
        let mut surface_fallback = false;
        if let Level::Functions(dir) = &row.level {
            // Строка 2.2: путь зеркала уже и есть каталог поверхности (`caja/libcaja-extension`);
            // строка 2 — поверхность живёт в подкаталоге репозитория.
            let hdir = if row.repo.contains('/') {
                p(&format!("vendor/mate/{}", row.repo))
            } else {
                p(&format!("vendor/mate/{}/{}", row.repo, dir))
            };
            let (headers, fallback) = public_headers(&hdir);
            surface_fallback = fallback;
            for name in headers {
                let text = std::fs::read_to_string(hdir.join(&name)).unwrap_or_default();
                functions.extend(header_functions(&text));
            }
            functions.sort();
            functions.dedup();
        }
        let title = format!(
            "# Парити-лист `{}` ← `{}` ({})",
            row.kreyt, row.repo, row.maylston
        );
        let path = p(&format!("docs/parity/{}.md", row.kreyt));
        let existing = std::fs::read_to_string(&path).ok();
        let text = splice(
            &title,
            &render_generated(&row, &head, &modules, &functions, surface_fallback),
            existing.as_deref(),
            HANDWRITTEN_STUB,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        written += 1;
    }
    println!("parity: листов перегенерировано: {written}");
    Ok(())
}

/// Проверка листов парити (ТЗ 5.1). Правила:
/// - лист существует и не устарел (HEAD зеркала совпадает с отпечатком);
/// - модуль либо назван в вердиктах, либо лист не помечен закрытым — закрытие покрывает 100 %;
/// - нулевой лист обязан содержать непустые «Причина:» и «Переносится:»;
/// - лист не обязан быть полным: открытые листы красным не падают, но и закрываться не могут.
fn check_parity_sheets() -> Result<(), String> {
    let rows = load_map()?;
    let mut problems: Vec<String> = Vec::new();
    let mut closed = 0;
    let mut open = 0;
    let mut modules_total = 0;
    let mut covered = 0;
    for row in &rows {
        let path = p(&format!("docs/parity/{}.md", row.kreyt));
        let Ok(text) = std::fs::read_to_string(&path) else {
            problems.push(format!(
                "{}: нет листа парити ({})",
                row.kreyt,
                path.display()
            ));
            continue;
        };
        let Some(head) = mirror_head(&row.repo) else {
            continue; // вне зеркала — нечего и сверять
        };
        let Some((generated, tail)) = text.split_once(GEN_END) else {
            problems.push(format!("{}: лист без генерируемого блока", row.kreyt));
            continue;
        };
        if !generated.contains(&format!("HEAD `{head}`")) {
            problems.push(format!(
                "{}: лист устарел — зеркало сдвинулось на `{head}` (обновить: `cargo run -p xtask -- parity`)",
                row.kreyt
            ));
        }
        let modules = module_names(generated);
        modules_total += modules.len();
        let missing: Vec<&String> = modules
            .iter()
            .filter(|m| !tail.contains(m.as_str()))
            .collect();
        covered += modules.len() - missing.len();
        if !zero_sheet_ok(tail, modules.len()) {
            problems.push(format!(
                "{}: нулевой лист без полей «Причина:» / «Переносится:» (ТЗ 5.1)",
                row.kreyt
            ));
        }
        if tail.contains("\nЗакрыт:") {
            closed += 1;
            if !missing.is_empty() {
                problems.push(format!(
                    "{}: лист закрыт, но вердиктов нет на {} модуля(ей): {} — закрывать нельзя",
                    row.kreyt,
                    missing.len(),
                    missing
                        .iter()
                        .take(3)
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        } else {
            open += 1;
        }
    }
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    println!(
        "parity: листов {}/{} · модулей .c: {} · вердиктами покрыто {} · закрыто {} · открыто {}",
        closed + open,
        rows.len(),
        modules_total,
        covered,
        closed,
        open
    );
    Ok(())
}

/// Правило нулевого листа (ТЗ 5.1): пустой перечень обязан объяснять себя непустыми
/// «Причина:» и «Переносится:».
fn zero_sheet_ok(tail: &str, modules: usize) -> bool {
    modules > 0
        || ["Причина:", "Переносится:"].iter().all(|field| {
            tail.lines().any(|l| {
                l.split_once(field)
                    .is_some_and(|(_, rest)| !rest.trim().is_empty())
            })
        })
}

/// Имена модулей из генерируемого блока: всё в обратных кавычках, кончающееся на `.c`/`.h`.
/// Раздел «Функции» таких токенов не содержит — там имена без расширения.
fn module_names(generated: &str) -> Vec<String> {
    generated
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|t| t.len() > 3 && (t.ends_with(".c") || t.ends_with(".h")))
        .map(str::to_string)
        .collect()
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

    fn fixture() -> PathBuf {
        p("xtask/tests/fixtures/mate-fake")
    }

    /// Генератор видит модули, а не переводы/данные фикстуры (Q47.4 — логика проверяется здесь).
    #[test]
    fn modulov_fikstury() {
        let got = modules_of(&fixture());
        assert_eq!(
            got.keys().map(String::as_str).collect::<Vec<_>>(),
            vec!["lib", "src"]
        );
        assert_eq!(
            got["src"],
            vec!["panel.c".to_string(), "window.c".to_string()]
        );
        assert_eq!(got["lib"], vec!["util.c".to_string()]);
        assert!(
            !got["src"].contains(&"test-window.c".to_string()),
            "автотесты оригинала не переносимые модули"
        );
    }

    /// Перегенерация трогает только свой блок; вердикты человека остаются (Q47.1).
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn rukopisnyy_blok_vyivaet() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М0".to_string(),
            level: Level::Modules,
        };
        let fresh = render_generated(&row, "deadbee", &modules_of(&fixture()), &[], false);
        let first = splice("# Титл", &fresh, None, HANDWRITTEN_STUB).unwrap();
        let with_verdicts = first.replace(
            "|---|---|---|---|",
            "|---|---|---|---|\n| `src/window.c` | модуль | готово | smoke |",
        );
        let again = splice(
            "# Новый титл",
            &fresh,
            Some(&with_verdicts),
            HANDWRITTEN_STUB,
        )
        .unwrap();
        assert!(
            again.starts_with("# Новый титл"),
            "заголовок обязан обновиться"
        );
        assert!(again.contains("| `src/window.c` | модуль | готово | smoke |"));
        assert_eq!(again.matches(GEN_BEGIN).count(), 1);
        // Лист без границ — чужая правка, молча перетирать её нельзя.
        assert!(
            splice(
                "# Титл",
                &fresh,
                Some("# Титл\nручной текст\n"),
                HANDWRITTEN_STUB
            )
            .is_err()
        );
    }

    /// Фикстура: заголовок с одной публичной функцией (многострочный прототип), export-
    /// макросом, static, typedef-ом и членом-указателем. Ассерт — ровно одна функция.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn поверхность_заголовка_это_только_публичные_функции() {
        let text = std::fs::read_to_string(p("xtask/tests/fixtures/include/fake-widget.h"))
            .expect("фикстура заголовка");
        assert_eq!(header_functions(&text), vec!["fake_widget_new".to_string()]);
    }

    /// Публичные заголовки: билд-система назвала — берём названное; не назвала — каталог
    /// целиком без `private` (запасной путь, лист обязан об этом сказать).
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn публичные_заголовки_из_билд_системы_и_запасным_путём() {
        let (listed, fallback) = public_headers(&p("xtask/tests/fixtures/include"));
        assert_eq!(listed, vec!["fake-widget.h".to_string()]);
        assert!(!fallback, "meson.build фикстуры называет заголовок");
        let (loose, fallback) = public_headers(&p("xtask/tests/fixtures/priv"));
        assert_eq!(
            loose,
            vec!["loose.h".to_string()],
            "`*-private.h` отсекается"
        );
        assert!(fallback, "билд-файла нет — это запасной путь");
    }

    /// Регрессия раунда 6: повторная перегенерация сохраняет ручной блок. Худший переход —
    /// лист с функциями становится листом без них, и обратно.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn сплайс_переживает_смену_уровня_парити() {
        let modules = modules_of(&fixture());
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М0".to_string(),
            level: Level::Functions("include".to_string()),
        };
        let with = render_generated(
            &row,
            "deadbee",
            &modules,
            &["fake_widget_new".to_string()],
            false,
        );
        let first = splice("# Титл", &with, None, HANDWRITTEN_STUB).unwrap();
        let hand = "\nПричина: публичной поверхности нет.\nПереносится: только API крейта.\n";
        let first = first.replacen(HANDWRITTEN_STUB, &format!("{HANDWRITTEN_STUB}{hand}"), 1);
        let plain = MapRow {
            level: Level::Modules,
            ..row.clone()
        };
        let without = render_generated(&plain, "deadbee", &modules, &[], false);
        let second = splice("# Титл", &without, Some(&first), "").unwrap();
        let third = splice("# Титл", &with, Some(&second), "").unwrap();
        for text in [&second, &third] {
            assert!(
                text.contains("Причина: публичной поверхности нет."),
                "ручной блок потерян"
            );
            assert_eq!(text.matches(GEN_BEGIN).count(), 1);
            assert_eq!(text.matches(GEN_END).count(), 1);
        }
        assert!(
            !second.contains("fake_widget_new"),
            "раздел функций обязан исчезнуть"
        );
        assert!(
            third.contains("fake_widget_new"),
            "раздел функций обязан вернуться"
        );
    }

    #[test]
    fn нулевой_лист_без_объяснения_невалиден() {
        assert!(!zero_sheet_ok("\n## Вердикты\n", 0));
        assert!(
            !zero_sheet_ok("\nПричина:\nПереносится: вот это.\n", 0),
            "пустое поле — не ответ"
        );
        assert!(zero_sheet_ok(
            "\nПричина: обёртка сборки.\nПереносится: М5.\n",
            0
        ));
        assert!(
            zero_sheet_ok("\nчто угодно\n", 7),
            "есть модули — поля не нужны"
        );
    }
}
