//! Ворота проекта (`xtask ci`) — ТЗ 6.3, среда ворот — ТЗ 3.8, ADR-0003.
//! Исполняется в среде ворот. Удалённого CI нет (ADR-0006): прогон здесь и есть прогон.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

/// Корень **проверяемого** дерева: спрашиваем git, а не компиляцию. `CARGO_MANIFEST_DIR`
/// замораживает путь того, кто собирал бинарник, — и ворота судили бы не то дерево, из
/// которого их запустили: прогон в чужом checkout показал бы зелёный цвет главного.
/// Откат на путь сборки — только для дерева без git (тогда судится то же, что собрано).
fn root() -> &'static Path {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    // `git_in`, не `git`: `git` сам спрашивает `root()`, а `OnceLock` на повторный вход
    // в ту же инициализацию вешает поток — я это вешал первым прогоном.
    ROOT.get_or_init(|| root_of(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))))
}

/// Корень дерева **из которого запущен прогон**. Возврат сюда `CARGO_MANIFEST_DIR` — это
/// и есть тот самоход: бинарник помнил дерево, где родился, и судил его из чужого checkout.
/// Тест `корень_берут_из_прогона_а_не_из_сборки` краснеет на такой мутации.
fn root_of(cwd: &Path) -> PathBuf {
    match git_in(cwd, &["rev-parse", "--show-toplevel"]) {
        Some(top) if !top.is_empty() => PathBuf::from(top),
        _ => Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    }
}

fn p(rel: &str) -> PathBuf {
    root().join(rel)
}

/// git-команда в указанном дереве. `None` — команды нет или она завершилась ошибкой.
fn git_in(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Крейты, не наследующие `[workspace.lints]` (ADR-0010). Расширение — только через ADR.
const LINT_ALLOWLIST: &[&str] = &["svetlitsa-efl"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Doctor,
    Fmt,
    Clippy,
    Test,
    Deny,
    Lints,
    Licenses,
    Parity,
    Canon,
}

/// Порядок шагов `xtask ci`. `--local` вычёркивает из него только средозависимый `doctor` (Q57):
/// остальные шаги обязаны быть красными/зелёными на любой машине, иначе ворота нельзя показать
/// там, где среды ворот ещё нет.
const ALL_STEPS: &[Step] = &[
    Step::Doctor,
    Step::Fmt,
    Step::Clippy,
    Step::Test,
    Step::Deny,
    Step::Lints,
    Step::Licenses,
    Step::Parity,
    Step::Canon,
];

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
            "canon" => Some(Self::Canon),
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
            Self::Canon => "канон",
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
        None | Some("ci") if rest.iter().any(|a| a == "--local") => ALL_STEPS
            .iter()
            .copied()
            .filter(|s| *s != Step::Doctor)
            .collect(),
        None | Some("ci") => match if rest.is_empty() {
            None
        } else {
            Some(parse_all(rest))
        } {
            Some(Some(steps)) if !steps.is_empty() => steps,
            Some(_) => {
                eprintln!(
                    "неизвестный шаг ворот: {}\nшаги: doctor fmt test clippy deny lints licenses parity canon",
                    rest.join(" ")
                );
                exit(2);
            }
            None => ALL_STEPS.to_vec(),
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
            // По одной строке на неисправность: блок «--- …» с несколькими проблемами
            // внутри нельзя ни посчитать, ни грепнуть, и отчёт начинает врать числом.
            for line in e.lines() {
                eprintln!("--- {line}");
            }
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
        Step::Canon => check_canon(),
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
    let (mut closed, mut open, mut modules_total, mut covered) = (0, 0, 0, 0);
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
        let (found, modules, verdicts) = sheet_problems(row, &head, generated, tail);
        problems.extend(found);
        modules_total += modules;
        covered += verdicts;
        if tail.contains("\nЗакрыт:") {
            closed += 1;
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

/// Правила одного листа (ТЗ 5), отделённые от обхода файлов: негативные тесты (Q53) гоняют
/// настоящее правило, а не его пересказ. Возвращает проблемы и счётчики (модули, покрыто).
fn sheet_problems(
    row: &MapRow,
    head: &str,
    generated: &str,
    tail: &str,
) -> (Vec<String>, usize, usize) {
    let mut problems = Vec::new();
    if !generated.contains(&format!("HEAD `{head}`")) {
        problems.push(format!(
            "{}: лист устарел — зеркало сдвинулось на `{head}` (обновить: `cargo run -p xtask -- parity`)",
            row.kreyt
        ));
    }
    let modules = module_names(generated);
    let missing: Vec<&String> = modules
        .iter()
        .filter(|m| !tail.contains(m.as_str()))
        .collect();
    if !zero_sheet_ok(tail, modules.len()) {
        problems.push(format!(
            "{}: нулевой лист без полей «Причина:» / «Переносится:» (ТЗ 5)",
            row.kreyt
        ));
    }
    // Q56: «руками» обязано отличаться от «забыли» — у листа с собственной поверхностью
    // причина отсутствия сгенерированного перечня стоит в рукописном блоке.
    if matches!(row.level, Level::OwnApi) && !field_filled(tail, "Причина:") {
        problems.push(format!(
            "{}: поверхность ведётся руками, но рукописный блок не объясняет почему (ТЗ 2.2, Q56)",
            row.kreyt
        ));
    }
    if tail.contains("\nЗакрыт:") && !missing.is_empty() {
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
    (problems, modules.len(), modules.len() - missing.len())
}

/// Непустое поле рукописного блока: `Причина: <текст>` (ТЗ 5).
fn field_filled(tail: &str, field: &str) -> bool {
    tail.lines().any(|l| {
        l.split_once(field)
            .is_some_and(|(_, rest)| !rest.trim().is_empty())
    })
}

/// Ворота канона (Q54): каждая запись `canon.toml` обязана совпадать с фактическим тегом,
/// и каждый тег `canon-*` обязан иметь запись. Двусторонне: односторонняя проверка молча
/// теряет забытый тег. Не зависит от среды — работает и при красном `doctor` (Q55).
fn check_canon() -> Result<(), String> {
    let text = std::fs::read_to_string(p("canon.toml")).map_err(|e| format!("canon.toml: {e}"))?;
    let identity = canon_field(&text, "identity").unwrap_or_else(|| {
        eprintln!("canon.toml: нет поля identity — подпись проверять не на что");
        String::new()
    });
    let records = canon_records(&text, &identity);
    let head = tag_commit("HEAD").unwrap_or_default();
    let (mut problems, notes) = canon_verdicts(&records, &canon_tags(), tag_commit, &head, |c| {
        is_ancestor(c)
    });
    // Q58: помеченное состояние должно быть проверяемым, а не доверенным. Файл
    // allowed_signers лежит в каноне, поэтому проверка не зависит от настроек клона.
    problems.extend(
        records
            .iter()
            .filter_map(|(tag, _, key)| tag_signature(tag, key)),
    );
    if let Some(p) = rotation_problem(&records, &identity) {
        problems.push(p);
    }
    // Случай 4 печатается всегда: молчание означало бы, что тег ушёл из сверки незаметно.
    for note in &notes {
        println!("  · {note}");
    }
    if problems.is_empty() {
        println!(
            "canon: записей {}, голова {}, теги совпадают",
            records.len(),
            records
                .last()
                .map(|(t, _, _)| t.as_str())
                .unwrap_or("нет записей")
        );
        return Ok(());
    }
    Err(problems.join("\n"))
}

/// Само правило (Q54), отдельно от git: запись ↔ тег в обе стороны.
///
/// Четыре случая сверки (Q59), относительно проверяемого состояния `head`; «строго
/// предшествует» = предок `head` и не равен ему. Причина, по которой наивная двусторонняя
/// сверка не имела ни одного удовлетворения, — собственная запись: дерево не содержит
/// собственный sha, поэтому случай 4 существует и он не краснота, а строка «вне сверки».
/// Цена — тег на HEAD без записи ловится ближайшим следующим коммитом; расхождение sha
/// ловится немедленно, случаем 2.
fn canon_verdicts(
    records: &[(String, String, String)],
    tags: &[String],
    resolve: impl Fn(&str) -> Option<String>,
    head: &str,
    ancestor: impl Fn(&str) -> bool,
) -> (Vec<String>, Vec<String>) {
    let strict = |commit: &str| commit != head && ancestor(commit);
    let mut problems = Vec::new();
    let mut notes = Vec::new();
    for (tag, sha, _) in records {
        match resolve(tag) {
            None => problems.push(format!("{tag}: записан в canon.toml, но тега нет")),
            Some(actual) if actual != *sha => problems.push(format!(
                "{tag}: canon.toml говорит `{sha}`, тег показывает `{actual}`"
            )),
            Some(actual) if !strict(&actual) => problems.push(format!(
                "{tag}: запись о теге вне линии проверяемого состояния — реестр зовёт чужеродный тег"
            )),
            Some(_) => {}
        }
    }
    for tag in tags {
        if records.iter().any(|(t, _, _)| t == tag) {
            continue;
        }
        let Some(commit) = resolve(tag) else {
            continue;
        };
        if strict(&commit) {
            problems.push(format!(
                "{tag}: тег есть, записи в canon.toml нет — реестр неполон"
            ));
        } else if commit == head {
            notes.push(format!(
                "{tag}: вне сверки — собственный тег проверяемого состояния"
            ));
        } else {
            notes.push(format!(
                "{tag}: вне сверки — тег не предок проверяемого состояния"
            ));
        }
    }
    (problems, notes)
}

/// Подпись тега (Q58): `git verify-tag` против записанного в каноне `allowed_signers`.
/// Возвращает проблему, если подписи нет, она чужая или её нечем проверить.
fn tag_signature(tag: &str, identity: &str) -> Option<String> {
    let signers = p("canon/allowed_signers");
    // Якорь доверия лежит в дереве. Если его в этом дереве ещё нет, то «принципал не
    // тот» было бы враньём: сверять было не с чем, и это другая неисправность.
    if !signers.exists() {
        return Some(format!(
            "{tag}: в проверяемом дереве нет canon/allowed_signers — якорь доверия отсутствует"
        ));
    }
    let out = Command::new("git")
        .args([
            "-c",
            &format!("gpg.ssh.allowedSignersFile={}", signers.display()),
            "verify-tag",
            tag,
        ])
        .current_dir(p("."))
        .output()
        .ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if identity.trim().is_empty() {
        return Some(format!(
            "{tag}: принципал не с чем сверять — в canon.toml нет поля identity"
        ));
    }
    if out.status.success() && text.contains(&format!("for {identity}")) {
        return None;
    }
    if text.contains("non-tag object") {
        return Some(format!("{tag}: тег легковесный — подписи нет в принципе"));
    }
    // «Подпись не проходит» и «подпись действительна, но принципал не тот» — разные
    // неисправности. Цитировать Good-подпись как отказ — это ворота, сообщающие не то,
    // что они измерили (проверено на дереве canon-r8, где identity отсутствовал).
    let quote = text
        .lines()
        .find(|l| l.contains("signature"))
        .or_else(|| text.lines().find(|l| !l.trim().is_empty()))
        .unwrap_or("пустой вывод")
        .trim();
    if text.contains("Good \"git\" signature") {
        return Some(format!(
            "{tag}: подпись действительна, но принципал не `{identity}` ({quote})"
        ));
    }
    Some(format!("{tag}: подпись не проходит: {quote}"))
}

/// Был ли коммит доступен проверяемому состоянию (то есть мог ли он быть записан).
fn is_ancestor(commit: &str) -> bool {
    Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(p("."))
        .output()
        .is_ok_and(|o| o.status.success())
}

fn canon_tags() -> Vec<String> {
    let Ok(out) = Command::new("git")
        .args(["tag", "-l", "canon-*"])
        .current_dir(p("."))
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

/// Скалярное поле шапки реестра (`identity = "..."`), до первой секции `[[canon]]`.
fn canon_field(text: &str, key: &str) -> Option<String> {
    let head = text.split("[[canon]]").next().unwrap_or("");
    head.lines().find_map(|l| {
        l.trim()
            .strip_prefix(&format!("{key} = \""))
            .map(|v| v.trim_end_matches('"').to_string())
    })
}

/// Записи `[[canon]]` как `(tag, sha, key)` — без TOML-парсера, формат реестра нарочно
/// плоский. `key` — принципал, подписавший эту запись; если поля нет, действовал тогдашний
/// `identity` (поле нужно ротации, ТЗ 6.3).
fn canon_records(text: &str, identity: &str) -> Vec<(String, String, String)> {
    let mut out: Vec<(String, String, String)> = Vec::new();
    let mut last_tag: Option<String> = None;
    let mut last_key: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("tag = \"") {
            last_tag = Some(rest.trim_end_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("key = \"") {
            last_key = Some(rest.trim_end_matches('"').to_string());
        } else if let Some(rest) = line.strip_prefix("sha = \"")
            && let Some(tag) = last_tag.take()
        {
            let key = last_key
                .take()
                .unwrap_or_else(|| identity.trim().to_string());
            out.push((tag, rest.trim_end_matches('"').to_string(), key));
        }
    }
    out
}

/// Ротация ключа (ТЗ 6.3): записи старше вывода ключа остаются проверяемыми, но
/// **последний** канон-тег обязан быть подписан действующим `identity`. Иначе ротация
/// проверялась бы только в день ротации: тег, подписанный выведенным ключом, прошёл бы
/// вечно. Мутация — убрать сравнение — краснит тест.
fn rotation_problem(records: &[(String, String, String)], identity: &str) -> Option<String> {
    let (tag, _, key) = records.last()?;
    (key != identity).then(|| {
        format!(
            "{tag}: последний канон-тег подписан ключом `{key}`, действует `{identity}` — ротация не доиграна"
        )
    })
}

fn tag_commit(tag: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--quiet", &format!("{tag}^{{commit}}")])
        .current_dir(p("."))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
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

    /// Негатив (Q53): правило обязано краснеть без полей и зеленеть с ними. Прогон идёт
    /// через настоящее правило листа и настоящий генерируемый блок нулевой фикстуры.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn нулевой_лист_красен_без_полей_и_зелён_с_ними() {
        let row = MapRow {
            repo: "mate-empty".to_string(),
            kreyt: "pustosh".to_string(),
            maylston: "М0".to_string(),
            level: Level::Modules,
        };
        let generated = render_generated(
            &row,
            "deadbee",
            &modules_of(&p("xtask/tests/fixtures/mate-empty")),
            &[],
            false,
        );
        assert!(
            module_names(&generated).is_empty(),
            "фикстура обязана быть нулевой — иначе тест ничего не проверяет"
        );
        let bare = splice("# Титл", &generated, None, HANDWRITTEN_STUB).unwrap();
        let (_, tail) = bare.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", &generated, tail)
                .0
                .iter()
                .any(|s| s.contains("нулевой лист")),
            "правило нулевого листа не сработало"
        );
        let explained =
            format!("{bare}\nПричина: обёртка сборки, кода нет.\nПереносится: оверрайд — М5.\n");
        let (_, tail) = explained.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", &generated, tail)
                .0
                .is_empty(),
            "объяснённый нулевой лист обязан быть зелёным"
        );
    }

    /// Негатив (Q56): «поверхность ведём руками» обязано отличаться от «забыли».
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn рукава_без_причины_красны() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М0".to_string(),
            level: Level::OwnApi,
        };
        let generated = render_generated(&row, "deadbee", &modules_of(&fixture()), &[], false);
        let bare = splice("# Титл", &generated, None, HANDWRITTEN_STUB).unwrap();
        let (g, tail) = bare.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail)
                .0
                .iter()
                .any(|s| s.contains("ведётся руками")),
            "лист «руками» без причины обязан краснеть"
        );
        let explained = format!("{bare}\nПричина: заголовков нет.\n");
        let (g, tail) = explained.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail).0.is_empty(),
            "с причиной — зелёный"
        );
    }

    /// Негатив (Q51): закрытие без 100 % вердиктов красно, с ними — зелено. Заодно —
    /// уехавший отпечаток.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn закрытие_требует_все_вердикты() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М0".to_string(),
            level: Level::Modules,
        };
        let generated = render_generated(&row, "deadbee", &modules_of(&fixture()), &[], false);
        let sheet = splice("# Титл", &generated, None, HANDWRITTEN_STUB).unwrap();
        let closed = format!("{sheet}\nЗакрыт: 2026-11-15, коммит `deadbee`\n");
        let (g, tail) = closed.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail)
                .0
                .iter()
                .any(|s| s.contains("лист закрыт")),
            "храповик закрытия не сработал"
        );
        let full = format!(
            "{closed}| `panel.c` | модуль | готово | |\n| `window.c` | модуль | готово | |\n| `util.c` | модуль | готово | |\n"
        );
        let (g, tail) = full.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail).0.is_empty(),
            "закрытие при полном покрытии обязано быть зелёным"
        );
        assert!(
            sheet_problems(&row, "stale1", g, tail)
                .0
                .iter()
                .any(|s| s.contains("лист устарел")),
            "уехавший отпечаток обязан краснеть"
        );
    }

    /// Реестр канона: ни одна запись не теряется, sha полные, формат разобран (Q54).
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn реестр_канона_разбирается_целиком() {
        let text = std::fs::read_to_string(p("canon.toml")).unwrap();
        let identity = canon_field(&text, "identity").unwrap();
        let records = canon_records(&text, &identity);
        assert_eq!(
            records.len(),
            text.matches("[[canon]]").count(),
            "запись без sha теряется молча"
        );
        assert!(
            records
                .iter()
                .all(|(tag, sha, key)| tag.starts_with("canon-")
                    && sha.len() == 40
                    && sha.bytes().all(|b| b.is_ascii_hexdigit())
                    && !key.is_empty()),
            "sha обязан быть полным — короткий даёт рассинхрон, который никто не заметит"
        );
        // Ротации ещё не было: поле `key` не проставлено, и каждая запись обязана
        // наследовать действующий identity — иначе «общий ключ» был бы домыслом ворот.
        assert!(
            records.iter().all(|(_, _, key)| key == &identity),
            "запись без `key` обязана наследовать действующий identity"
        );
    }

    /// Ротация ключа (ТЗ 6.3): последний канон-тег подписан действующим ключом. Мутация —
    /// заменить сравнение на `false` — краснит тест.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn ключ_последней_записи_обязан_быть_действующим() {
        let old = ("canon-r9".to_string(), "a".repeat(40), "old@k".to_string());
        let fresh = ("canon-r10".to_string(), "b".repeat(40), "new@k".to_string());
        assert_eq!(
            rotation_problem(&[old.clone()], "new@k").unwrap(),
            "canon-r9: последний канон-тег подписан ключом `old@k`, действует `new@k` — ротация не доиграна"
        );
        assert!(rotation_problem(&[fresh.clone()], "new@k").is_none());
        // Прежние записи с выведенным ключом остаются зелёными: ворота судят последнюю.
        assert!(rotation_problem(&[old, fresh], "new@k").is_none());
        assert!(rotation_problem(&[], "new@k").is_none());
    }

    /// Якорь путей (самоход раунда 9): корень дерева спрашивается у git из cwd прогона.
    /// Мутация — вернуть `CARGO_MANIFEST_DIR` — краснит тест.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn корень_берут_из_прогона_а_не_из_сборки() {
        let dir = std::env::temp_dir().join(format!("svetlitsa-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("canon.toml"), "head = \"поддельный\"\n").unwrap();
        let init = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .status()
            .is_ok();
        assert!(
            init,
            "нет git в среде: тест не прогнан, а зелёным быть не может (серое «сделано» — Q57)"
        );
        let got = root_of(&dir);
        // Снимать пути надо до удаления каталога: canonicalize() на мёртвом пути даёт
        // NotFound, и тест краснеет по несуществующей причине.
        let got = got.canonicalize().unwrap();
        let want = dir.canonicalize().unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(got, want);
        assert_ne!(
            got,
            root().canonicalize().unwrap(),
            "корень вернулся из контекста сборки — тот самый самоход"
        );
    }

    /// Четыре случая сверки (Q59) на одном наборе: 1 и 3 красны, 2 красен немедленно,
    /// 4 — строка «вне сверки», а не молчание. Зелёное состояние проверяется отдельно.
    #[test]
    fn сверка_канона_четыре_случая() {
        let resolve = |tag: &str| match tag {
            "canon-r1" => Some("c".repeat(40)),
            "canon-rX" => Some("off-line".to_string()),
            "canon-r2" => Some("older".to_string()),
            "canon-head" => Some("head-sha".to_string()),
            "canon-future" => Some("future".to_string()),
            _ => None,
        };
        let records = vec![
            ("canon-r1".to_string(), "a".repeat(40), "k".to_string()),
            (
                "canon-rX".to_string(),
                "off-line".to_string(),
                "k".to_string(),
            ),
            ("canon-missing".to_string(), "e".repeat(40), "k".to_string()),
        ];
        let tags = [
            "canon-r1",
            "canon-rX",
            "canon-r2",
            "canon-head",
            "canon-future",
        ]
        .map(str::to_string)
        .to_vec();
        let (problems, notes) = canon_verdicts(&records, &tags, resolve, "head-sha", |c| {
            c != "future" && c != "off-line"
        });
        assert_eq!(
            problems.len(),
            4,
            "расхождение sha + чужеродный тег + запись без тега + тег без записии"
        );
        assert!(problems.iter().any(|s| s.contains("canon.toml говорит")));
        assert!(
            problems
                .iter()
                .any(|s| s.contains("вне линии проверяемого"))
        );
        assert!(problems.iter().any(|s| s.contains("но тега нет")));
        assert!(problems.iter().any(|s| s.contains("реестр неполон")));
        assert_eq!(notes.len(), 2, "оба случая «вне сверки» обязаны печататься");
        assert!(notes.iter().any(|s| s.contains("собственный тег")));
        assert!(notes.iter().any(|s| s.contains("не предок")));
    }

    /// Зелёное состояние: записи про строгих предков, теги записаны, «вне сверки» пуст.
    #[test]
    fn сверка_канона_зелёна_на_своем_состоянии() {
        let records = vec![("canon-r1".to_string(), "older".to_string(), "k".to_string())];
        let tags = vec!["canon-r1".to_string()];
        let (problems, notes) = canon_verdicts(
            &records,
            &tags,
            |_| Some("older".to_string()),
            "head-sha",
            |_| true,
        );
        assert!(problems.is_empty(), "{problems:?}");
        assert!(notes.is_empty(), "{notes:?}");
    }

    #[test]
    /// Тест предиката полей, а не правила: он остаётся зелёным, даже когда правило
    /// отключено от ворот (проверено мутацией), и потому не считается проверкой правила —
    /// правило проверяет `нулевой_лист_красен_без_полей_и_зелён_с_ними`.
    fn предикат_нулевого_листа_полей() {
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
