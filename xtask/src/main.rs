//! Ворота проекта (`xtask ci`) — ТЗ 6.3, среда ворот — ТЗ 3.8, ADR-0003.
//! Исполняется в среде ворот. Удалённого CI нет (ADR-0006): прогон здесь и есть прогон.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use svetlitsa_icons::{GEN_BEGIN, GEN_END, ListError, parse_closed_list, well_formed};

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
    Icons,
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
    Step::Icons,
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
            "icons" => Some(Self::Icons),
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
            Self::Icons => "закрытый список иконок",
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
                    "неизвестный шаг ворот: {}\nшаги: doctor fmt test clippy deny lints licenses parity icons canon",
                    rest.join(" ")
                );
                exit(2);
            }
            None => ALL_STEPS.to_vec(),
        },
        Some("parity") if args.get(1).map(String::as_str) == Some("--draft") => {
            let Some(c) = args.get(2).map(String::as_str) else {
                eprintln!("parity: --draft требует один крейт: `xtask parity --draft <крейт>`");
                exit(1);
            };
            if let Err(e) = parity_draft(c) {
                eprintln!("{e}");
                exit(1);
            }
            return;
        }
        Some("parity") if args.get(1).map(String::as_str) == Some("--legend") => {
            if let Err(e) = parity_legend() {
                eprintln!("{e}");
                exit(1);
            }
            return;
        }
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
        Some("icons") if args.get(1).map(String::as_str) == Some("--check") => {
            if let Err(e) = check_icons() {
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
                    "неизвестная задача: {other}\nиспользование: xtask [ci|<шаг>|parity <компонент>|icons [--check]]"
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
        Step::Icons => check_icons(),
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

/// М2-компоненты, чьи зеркала дают имена иконок (ADR-0012).
const M2_REPOS: &[&str] = &[
    "mate-panel",
    "marco",
    "mate-screensaver",
    "mate-session-manager",
    "mate-settings-daemon",
    "mate-desktop",
    "mate-notification-daemon",
];

/// Имена, которые зеркало знает: `.png` из `mate-icon-theme`, пересечённые с freedesktop-
/// токенами М2-компонентов. Третьего источника имён нет (ADR-0014).
fn derived_icons(theme: &Path, repos: &[PathBuf]) -> BTreeSet<String> {
    let mut known: BTreeSet<String> = BTreeSet::new();
    walk(theme, &mut |path| {
        if let Some(stem) = stem(path, "png") {
            known.insert(stem);
        }
    });
    let mut used: BTreeSet<String> = BTreeSet::new();
    for repo in repos {
        walk(repo, &mut |path| {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            for token in icon_tokens(&text) {
                if known.contains(&token) {
                    used.insert(token);
                }
            }
        });
    }
    used
}

/// Генерируемый блок закрытого списка (ADR-0012). Число печатает прогон, литерала нет (Q61).
fn render_icons(used: &BTreeSet<String>) -> String {
    let mut out = format!(
        "{GEN_BEGIN}\n\nИконок: {} — имена из зеркал М2-компонентов, которые есть в \n`mate-icon-theme` (ADR-0012). Пополняется PR'ом, а не молча.\n\n",
        used.len()
    );
    for name in used {
        out.push_str(&format!("- `{name}`\n"));
    }
    out.push('\n');
    out.push_str(GEN_END);
    out
}

/// Состояние сверки закрытого списка, собранное из среды.
struct IconsState<'a> {
    /// Отпечаток зеркала (Q47.3); `None` — зеркала нет на машине.
    mirror: Option<&'a str>,
    /// Что знает зеркало.
    derived: &'a BTreeSet<String>,
    /// Что опубликовано в генерируемом блоке листа.
    listed: &'a BTreeSet<String>,
}

/// Вердикт сверки. Отсутствие зеркала — **красная строка**, а не пропуск: зелёный цвет от
/// нечего проверять — ровно тот самоход, каким сегодня сворачивается сверка парити-листов,
/// когда `vendor/mate` нет (Q57).
fn icons_verdict(s: &IconsState) -> Result<String, String> {
    let Some(sha) = s.mirror else {
        return Err("icons: зеркала нет — список не сверен".to_string());
    };
    let mut problems: Vec<String> = Vec::new();
    // Форма имени — правило крейта, а не ворот (Q77): зеркало может знать файл, чьё имя
    // приложение не примет. Молча положить такое имя в список — значит пообещать пустоту.
    for name in s.derived.iter().filter(|name| !well_formed(name)) {
        problems.push(format!(
            "испорченное имя «{name}»: зеркало знает имя, не проходящее форму"
        ));
    }
    for name in s.listed.difference(s.derived) {
        problems.push(format!(
            "лишнее имя «{name}»: в листе есть, зеркало его не знает"
        ));
    }
    for name in s.derived.difference(s.listed) {
        problems.push(format!(
            "отсутствующее имя «{name}»: зеркало знает, в листе нет"
        ));
    }
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    Ok(format!(
        "icons: {} имён · зеркало {sha} · список совпадает",
        s.derived.len()
    ))
}

/// Почему список не собрался — строка ворот, а не отладочный вывод. Разбирает лист
/// `svetlitsa-icons`: тот же код, что читает его приложение (Q77).
fn list_error(e: ListError) -> String {
    match e {
        ListError::NoBlock => "список не опубликован: в листе нет генерируемого блока".to_string(),
        ListError::Unclosed => "лист правился вне генератора: блок открыт и не закрыт".to_string(),
        ListError::Duplicate { name } => {
            format!("повтор имени «{name}»: закрытый список обязан быть множеством, а не списком")
        }
        ListError::IllFormed { name } => {
            format!("испорченное имя «{name}»: список обещает имя, не проходящее форму")
        }
    }
}

/// Шаг ворот `icons`: сверяет закрытый список с зеркалом, ничего не записывая.
fn check_icons() -> Result<(), String> {
    let theme = p("vendor/mate/mate-icon-theme");
    let repos: Vec<PathBuf> = M2_REPOS
        .iter()
        .map(|repo| p(&format!("vendor/mate/{repo}")))
        .collect();
    let derived = if theme.is_dir() {
        derived_icons(&theme, &repos)
    } else {
        BTreeSet::new()
    };
    let sheet = std::fs::read_to_string(p("docs/parity/svetlitsa-ui.md"))
        .map_err(|e| format!("docs/parity/svetlitsa-ui.md: {e}"))?;
    let listed = parse_closed_list(&sheet).map_err(list_error)?;
    let sha = mirror_head("mate-icon-theme");
    let line = icons_verdict(&IconsState {
        mirror: sha.as_deref(),
        derived: &derived,
        listed: &listed,
    })?;
    println!("{line}");
    Ok(())
}

/// Закрытый список иконок М2 (ADR-0012): имена из зеркал М2-компонентов, которые реально
/// существуют в `mate-icon-theme`. Заготовка — человек сокращает, молча не растёт.
fn icons() {
    let theme = p("vendor/mate/mate-icon-theme");
    if !theme.is_dir() {
        eprintln!("vendor/mate/mate-icon-theme нет — сначала scripts/fetch-vendor.sh");
        exit(1);
    }
    let repos: Vec<PathBuf> = M2_REPOS
        .iter()
        .map(|repo| p(&format!("vendor/mate/{repo}")))
        .collect();
    let used = derived_icons(&theme, &repos);
    let generated = render_icons(&used);
    let out_path = p("docs/parity/svetlitsa-ui.md");
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

// Границы генерируемого блока (`GEN_BEGIN`/`GEN_END`) — в `svetlitsa-icons`, который их читает:
// два определения одного формата означали бы возможное состояние «лист разошёлся с парсером».

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

/// Легенда статусов рукописного блока (Q83). Один текст для всех 31 листа: ворота сверяют
/// по нему `статус вне легенды`, а `xtask parity --legend` ставит его в листы. `готово`
/// здесь запрещено до первого зелёного smoke-прогона — иначе лист обещает статус, которым
/// в среде ворот нечем доказать.
const LEGEND_TEXT: &str = concat!(
    "Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после\n",
    "первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.\n",
    "`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено\n",
    "конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;\n",
    "`MVP` — обязательно к закрытию компонента.",
);

/// Маркер легенды в рукописном блоке: по нему ворота видят, что легенда в листе есть (Q83).
const LEGEND_MARKER: &str = "Статусы позиции:";

/// Прежняя легенда (до Q83) — первый абзац рукописного блока, который генератор писал сам.
/// Заменяется только она целиком; правка человека под замену не подпадает.
const LEGEND_OLD_PREFIX: &str = "Статусы: `нет` /";

/// Рукописный блок при создании листа: вердиктов ещё нет, и генератор их не додумывает.
fn handwritten_stub() -> String {
    format!(
        "\n\n## Вердикты (ручной блок — генератор его не трогает)\n{LEGEND_TEXT}\n\n\
| Позиция | Тип | Статус | Пометка |\n|---|---|---|---|\n"
    )
}

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
            &handwritten_stub(),
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        written += 1;
    }
    println!("parity: листов перегенерировано: {written}");
    Ok(())
}

/// `xtask parity --legend`: поставить легенду статусов (Q83) во все листы. Отдельный
/// явный флаг — не молчаливая перегенерация: рукописный блок генератор без команды не
/// трогает (Q47.1). Идемпотентен: лист с новой легендой остаётся как был. Меняется только
/// тот текст, который когда-то написал сам генератор (прежняя легенда); там, где легенду
/// правили руками, команда останавливается и просит человека.
fn parity_legend() -> Result<(), String> {
    let mut обновлено = 0;
    let mut как_было = 0;
    let mut руки = Vec::new();
    for row in load_map()? {
        let path = p(&format!("docs/parity/{}.md", row.kreyt));
        let текст =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let Some((generated, tail)) = текст.split_once(GEN_END) else {
            return Err(format!("{}: лист без генерируемого блока", row.kreyt));
        };
        if tail.contains(LEGEND_MARKER) && !legend_promise_without_smoke(tail) {
            как_было += 1;
            continue;
        }
        let Some(новый_хвост) = swap_legend(tail) else {
            руки.push(row.kreyt);
            continue;
        };
        // `str::lines` съедает завершающий перевод строки, а он в листе значим: без
        // возврата лист получил бы правку конца файла вместо правки легенды.
        let хвост = if tail.ends_with('\n') {
            format!("{новый_хвост}\n")
        } else {
            новый_хвост
        };
        std::fs::write(&path, format!("{generated}{GEN_END}{хвост}"))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        обновлено += 1;
    }
    println!(
        "parity (--legend): легенду проставлено: {обновлено} · уже было: {как_было} · требует человека: {}",
        руки.len()
    );
    if !руки.is_empty() {
        println!(
            "parity (--legend): эти листы правились руками, легенду в них вставляет человек: {}",
            руки.join(", ")
        );
    }
    Ok(())
}

/// Заменить легенду в хвосте листа: прежний абзац-заготовку генератора — на новую, а если
/// абзаца нет — вставить новую перед таблицей вердиктов. `None` — легенда правилась руками
/// (не маркер, не заготовка), и подменять её текст генератор не вправе.
fn swap_legend(tail: &str) -> Option<String> {
    let строки: Vec<&str> = tail.lines().collect();
    let позиция = строки
        .iter()
        .position(|l| l.trim_start().starts_with(LEGEND_OLD_PREFIX))?;
    let mut конец = позиция;
    while конец < строки.len() && !строки[конец].trim().is_empty() {
        конец += 1;
    }
    let mut новые = строки[..позиция].to_vec();
    новые.extend(LEGEND_TEXT.lines());
    новые.extend_from_slice(&строки[конец..]);
    Some(новые.join("\n"))
}

/// `xtask parity --draft <крейт>` (Q88): строки-скелет для позиций без вердикта — модулей
/// и публичных функций. Только по явному флагу и только одного крейта: молча вставить строку
/// в рукописный блок генератор не имеет права (Q47.1). Идемпотентен: под ключом, где строка
/// уже есть, второй строки не появляется; заполненный статус и прозу не трогает, потому что
/// такие ключи в перечень черновика не попадают.
fn parity_draft(component: &str) -> Result<(), String> {
    let rows: Vec<MapRow> = load_map()?
        .into_iter()
        .filter(|r| r.kreyt == component)
        .collect();
    if rows.is_empty() {
        return Err(format!("parity: крейта `{component}` в карте нет"));
    }
    // Черновик пишется по свежим ключам, иначе лист предложил бы позиции, которых в перечне нет.
    parity(Some(component))?;
    for row in rows {
        let path = p(&format!("docs/parity/{}.md", row.kreyt));
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (лист, добавлено) =
            sheet_with_draft(&text).map_err(|e| format!("{}: {e}", row.kreyt))?;
        let итог = match лист {
            Some(лист) => {
                std::fs::write(&path, &лист).map_err(|e| format!("{}: {e}", path.display()))?;
                лист
            }
            // Добавлять нечего: у каждой позиции строка вердикта уже есть.
            None => text,
        };
        // Черновиков — тем же счётом, что и ворота: идемпотентность обязана быть видно
        // числом, а не обещанием (Q88).
        let (_, хвост) = итог
            .split_once(GEN_END)
            .ok_or("лист без генерируемого блока")?;
        let (_, _, черновиков) = verdict_rows(хвост, true);
        println!(
            "parity (--draft {}): строк добавлено: {добавлено} · черновиков: {черновиков} · статус пустой: вердикт пишет человек (Q88)",
            row.kreyt
        );
    }
    Ok(())
}

/// Проверка листов парити (ТЗ 5.1). Правила:
/// - лист существует и не устарел (HEAD зеркала совпадает с отпечатком);
/// - модуль либо назван в вердиктах, либо лист не помечен закрытым — закрытие покрывает 100 %;
/// - нулевой лист обязан содержать непустые «Причина:» и «Переносится:»;
/// - лист не обязан быть полным: открытые листы красным не падают, но и закрываться не могут.
fn check_parity_sheets() -> Result<(), String> {
    let rows = load_map()?;
    let mut without_mirror: Vec<&str> = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    let (mut closed, mut open, mut positions_total, mut covered) = (0, 0, 0, 0);
    let (mut funcs_total, mut funcs_covered) = (0, 0);
    let mut drafts = 0;
    // «Позиций» и «уникальных каталог/файл» — разные числа (Q82): `uzelok` и `uzelok-core` /
    // смотрят в один `engrampa`, и 56 модулей считаются позициями двух крейтов, но одним
    // upstream-файлом. Печатать одно число — маскировать двойной счёт под счёт файлов.
    let mut upstream: BTreeSet<String> = BTreeSet::new();
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
            // Раньше здесь был `continue` с комментарием «вне зеркала — нечего и сверять»:
            // без зеркала он срабатывал на каждой строке, `problems` оставался пустым, и шаг
            // печатал зелёный итог, ничего не сверив. Пропуск собирается в вердикт (Q79).
            without_mirror.push(row.repo.as_str());
            continue;
        };
        let Some((generated, tail)) = text.split_once(GEN_END) else {
            problems.push(format!("{}: лист без генерируемого блока", row.kreyt));
            continue;
        };
        let (found, c) = sheet_problems(row, &head, generated, tail);
        problems.extend(found);
        positions_total += c.positions;
        covered += c.covered;
        funcs_total += c.functions;
        funcs_covered += c.functions_covered;
        drafts += c.drafts;
        upstream.extend(
            position_keys(generated)
                .into_iter()
                .map(|k| format!("{}/{}", row.repo, k)),
        );
        if parse_closing(tail).is_some() {
            closed += 1;
        } else {
            open += 1;
        }
    }
    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    // Вердикт о полноте сверки — после проблем по найденным листам: если зеркало на месте и
    // листы сверены, но часть строк карты всё же без зеркала, краснеть обязано именно это.
    parity_verdict(&ParityState {
        rows: rows.len(),
        without_mirror: &without_mirror,
        positions: positions_total,
    })?;
    let note = if positions_total == 0 {
        " (нет позиций)"
    } else {
        ""
    };
    // Две дроби, а не одна (Q87): «покрыто» про модули и «покрыто» про функции поверхности.
    // Ноль функций — напечатанный ноль: у 24 листов из 31 публичной C-поверхности нет.
    let funcs_note = if funcs_total == 0 { " (нет)" } else { "" };
    // Черновики `--draft` печатаются, пока они есть: строка без статуса не вердикт (Q88),
    // и молча прятать незаполненный список — значит выдавать незаполненное за пустоту.
    let draft_note = if drafts == 0 {
        String::new()
    } else {
        format!(" · черновиков: {drafts}")
    };
    println!(
        "parity: листов {}/{} · позиций: {} · уникальных каталог/файл: {} · покрыто {}/{}{} · функциями покрыто {}/{}{}{draft_note} · закрыто {} · открыто {}",
        closed + open,
        rows.len(),
        positions_total,
        upstream.len(),
        covered,
        positions_total,
        note,
        funcs_covered,
        funcs_total,
        funcs_note,
        closed,
        open
    );
    Ok(())
}

/// Что шаг `parity` увидел в дереве, отделённо от обхода файлов — по образцу `icons_verdict`:
/// негативные тесты (Q53) судят правило, а не его пересказ.
struct ParityState<'a> {
    rows: usize,
    without_mirror: &'a [&'a str],
    /// Суммарное число позиций перечня во всех сверенных листах (Q82): при нуле сводить
    /// покрытие нечего, и это красный цвет, а не `0/0` зелёным.
    positions: usize,
}

/// Правило (Q79, распространение Q67 на все шаги): шаг не зеленеет, когда сверять нечего.
/// Зелёный цвет от нечего проверять — не доказательство.
fn parity_verdict(s: &ParityState) -> Result<(), String> {
    if s.rows == 0 {
        return Err("parity: карта пустая — ни одного листа сверять нечего".to_string());
    }
    if s.without_mirror.len() == s.rows {
        return Err(format!(
            "parity: зеркала нет — ни один из {} листов не сверен (scripts/fetch-vendor.sh)",
            s.rows
        ));
    }
    if !s.without_mirror.is_empty() {
        return Err(format!(
            "parity: нет зеркала для: {} — сверка неполная",
            s.without_mirror.join(", ")
        ));
    }
    if s.positions == 0 {
        return Err(format!(
            "parity: позиций перечня нет ни в одном из {} листов — покрывать нечего (лист без перечня обязан объяснять причину, ТЗ 5)",
            s.rows
        ));
    }
    Ok(())
}

/// Легенда статусов рукописного блока (ТЗ 5). `готово` запрещён до первого зелёного
/// smoke-прогона — это правило тикета 04 (Q83); здесь статус обязан быть из легенды и
/// непустым (Q86).
const STATUS_LEGEND: &[&str] = &["нет", "в работе", "готово"];

fn status_ok(status: &str) -> bool {
    // `отложено` без причины в скобках — не статус: отложенное обязано быть перечлимым (ТЗ 5).
    STATUS_LEGEND.contains(&status) || (status.starts_with("отложено") && status.contains('('))
}

/// Причина, по которой статус не из легенды: отдельное сообщение для `отложено` без причины
/// и для судьбы листа, иначе человек читает «не из легенды» и не понимает, куда деть
/// «заменено конструкцией» (Q83).
fn status_problem(status: &str) -> String {
    if status.starts_with("отложено") {
        return concat!(
            "`отложено` без причины в скобках — не статус (ТЗ 5: отложенное обязано ",
            "быть перечислимым)",
        )
        .to_string();
    }
    if status.contains("заменено") || status.contains("переносится") || status.contains("переносим")
    {
        return format!(
            "статус «{status}» — судьба листа, а не готовность позиции: для неё поле `Судьба:` (Q83)"
        );
    }
    format!("статус «{status}» не из легенды (ТЗ 5)")
}

/// Доказан ли в этой среде зелёный smoke-прогон (ТЗ 6.9). На 2026-11-16 — нет: `xtask ci`
/// smoke-сценарии не исполняет (`docs/env-checklist.md`), поэтому «готово» и штамп «Закрыт»
/// красны независимо от того, насколько настоящий sha в штампе (Q83, Q89). Переключает
/// значение только раунд, который исполнит smoke и запишет прогон в журнал.
fn smoke_proven() -> bool {
    false
}

/// Легенда листа обещает `готово` без оговорки про smoke (Q83): ровно та причина, по которой
/// ворота и ТЗ 6.9 расходились — легенда жила в 31 листе и была сильнее правила.
fn legend_promise_without_smoke(tail: &str) -> bool {
    let Some(at) = tail
        .lines()
        .position(|l| l.trim_start().starts_with(LEGEND_MARKER))
    else {
        return false;
    };
    let абзац = tail.lines().skip(at).take_while(|l| !l.trim().is_empty());
    let текст = абзац.collect::<Vec<_>>().join(" ");
    текст.contains("готово") && !текст.contains("smoke")
}

/// Штамп закрытия из рукописного блока: `Закрыт: <ГГГГ-ММ-ДД>, main@<sha>[ · canon-rN]` (Q89).
#[derive(Debug, PartialEq, Eq)]
struct ClosingStamp<'a> {
    date: &'a str,
    sha: Option<&'a str>,
    tag: Option<&'a str>,
    raw: &'a str,
}

/// Найти штамп закрытия. `None` — лист открыт (штампa нет), и это не «плохого штампа нет».
fn parse_closing(tail: &str) -> Option<ClosingStamp<'_>> {
    let raw = tail
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("Закрыт:"))?;
    let остаток = raw["Закрыт:".len()..].trim();
    let (date, остальное) = остаток.split_once(',').unwrap_or((остаток, ""));
    let остальное = остальное.trim();
    let (sha, tag) = match остальное.split_once('·') {
        Some((sha, tag)) => (Some(sha.trim()), Some(tag.trim())),
        None => (Some(остальное), None),
    };
    let sha = sha.filter(|s| !s.is_empty() && *s != "—");
    let sha = sha.map(|s| s.strip_prefix("main@").unwrap_or(s));
    Some(ClosingStamp {
        date: date.trim(),
        sha,
        tag,
        raw,
    })
}

/// Проверка штампа закрытия (Q89). Валидный sha — необходимое, не достаточное условие:
/// без зелёного smoke закрытие не доказано, поэтому `smoke_proven` красит даже идеальный
/// штамп. `commit_exists`/`ancestor_of_main`/`tag_commit` — обёртки над git; тесты подставляют
/// свои ответы вместо настоящего репозитория (по образцу `canon_verdicts`).
fn closing_problems(
    stamp: &ClosingStamp,
    smoke_proven: bool,
    commit_exists: impl Fn(&str) -> bool,
    ancestor_of_main: impl Fn(&str) -> bool,
    tag_commit: impl Fn(&str) -> Option<String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    let дата = stamp.date;
    if дата.len() != 10
        || !дата[..4].bytes().all(|b| b.is_ascii_digit())
        || !дата[5..7].bytes().all(|b| b.is_ascii_digit())
        || !дата[8..10].bytes().all(|b| b.is_ascii_digit())
        || дата.as_bytes().get(4) != Some(&b'-')
        || дата.as_bytes().get(7) != Some(&b'-')
    {
        problems.push(format!(
            "штамп закрытия: дата «{дата}» не ГГГГ-ММ-ДД (Q89): {}",
            stamp.raw
        ));
    }
    let Some(sha) = stamp.sha else {
        problems.push(format!(
            "штамп закрытия без коммита — формат `Закрыт: <ГГГГ-ММ-ДД>, main@<sha>` (Q89): {}",
            stamp.raw
        ));
        problems.extend(smoke_problem(smoke_proven));
        return problems;
    };
    if sha.len() < 7 || sha.len() > 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        problems.push(format!(
            "штамп закрытия: «{sha}» не похоже на sha коммита (Q89): {}",
            stamp.raw
        ));
        problems.extend(smoke_problem(smoke_proven));
        return problems;
    }
    if !commit_exists(sha) {
        problems.push(format!(
            "штамп закрытия: коммита {sha} нет в репозитории (`git cat-file -e {sha}^{{commit}}`, Q89)"
        ));
    } else if !ancestor_of_main(sha) {
        problems.push(format!(
            "штамп закрытия: {sha} не предок main — закрытие обещает коммит другой линии (Q89)"
        ));
    }
    if let Some(tag) = stamp.tag {
        match tag_commit(tag) {
            None => problems.push(format!(
                "штамп закрытия: тег {tag} приписан, но его нет (`git rev-parse --verify --quiet {tag}^{{commit}}`, Q89)"
            )),
            Some(коммит) if !коммит.starts_with(sha) && !sha.starts_with(&коммит) => {
                problems.push(format!(
                    "штамп закрытия: {tag} указывает на {коммит}, а штамп обещает {sha} (Q89)"
                ));
            }
            Some(_) => {}
        }
    }
    problems.extend(smoke_problem(smoke_proven));
    problems
}

fn smoke_problem(smoke_proven: bool) -> Option<String> {
    (!smoke_proven).then(|| {
        concat!(
            "лист объявлен закрытым, а зелёного smoke-прогона нет (ТЗ 6.9, Q83/Q89): ",
            "валидный sha — необходимое, но не достаточное условие; в этом вороте smoke ",
            "не исполняется",
        )
        .to_string()
    })
}

/// Позиция перечня = модуль под ключом `каталог/файл` (Q85). Базовое имя ключом не является:
/// `main.c` в `skhodka` лежит в четырёх каталогах, `gpm-common.c` в `kerosinka` — в четырёх.
/// Каталог `.` (модули в корне зеркала) в ключ не печатается.
fn position_keys(generated: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in generated.lines().filter(|l| l.starts_with("| `")) {
        let mut cells = line.split('|').map(str::trim);
        let (Some(dir), Some(files)) = (cells.nth(1), cells.next()) else {
            continue;
        };
        let dir = dir.trim_matches('`');
        for file in files
            .split(',')
            .map(str::trim)
            .map(|f| f.trim_matches('`').trim_end_matches('|').trim())
        {
            if file.is_empty() {
                continue;
            }
            out.push(if dir == "." {
                file.to_string()
            } else {
                format!("{dir}/{file}")
            });
        }
    }
    out
}

fn backticked(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// Строки вердиктов листа: (ключи строки, статус), проблемы формата и число черновиков.
type VerdictRows = (Vec<(Vec<String>, String)>, Vec<String>, usize);

/// Вердикт (Q86) = строка таблицы рукописного блока: первый столбец — один или несколько
/// ключей `каталог/файл` в обратных кавычках (групповой вердикт, Q84), третий — статус из
/// легенды. Проза («Причина: `src/main.c` …») и подстрока имени вердиктом не являются.
/// Пустой статус — черновик `--draft` (Q88): ключ счит занят, но вердикта нет и красным
/// строка не является: человек до строки ещё не дошёл.
/// Возвращает строки, проблемы и число черновиков.
fn verdict_rows(tail: &str, smoke_proven: bool) -> VerdictRows {
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    let mut drafts = 0;
    let mut gotovo_do_smoke = 0;
    for line in tail.lines().filter(|l| l.trim_start().starts_with("| ")) {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // | Позиция | Тип | Статус | Пометка | → ["", «Позиция», «Тип», «Статус», «Пометка», ""]
        if cells.len() < 5
            || matches!(cells[1], "Позиция" | "Функция")
            || cells[1].starts_with("---")
        {
            continue;
        }
        let keys = backticked(cells[1]);
        let status = cells[3].to_string();
        if keys.is_empty() {
            problems.push(format!(
                "строка вердикта без ключа `каталог/файл` в первом столбце: {}",
                line.trim()
            ));
            continue;
        }
        if !status_ok(&status) {
            if status.is_empty() {
                drafts += 1;
            } else {
                problems.push(format!("{}: {}", status_problem(&status), line.trim()));
                continue;
            }
        }
        if status == "готово" && !smoke_proven {
            gotovo_do_smoke += 1;
        }
        rows.push((keys, status));
    }
    if gotovo_do_smoke > 0 {
        problems.push(format!(
            "статус «готово» у {gotovo_do_smoke} позици(й) до первого зелёного smoke-прогона — \
             запрещён (ТЗ 6.9, Q83)"
        ));
    }
    (rows, problems, drafts)
}

/// Ключи, под которыми в таблице вердиктов уже есть строка: с вердиктом, с черновиком или
/// с пустым статусом (Q88 — идемпотентность черновика, Q84 — один ключ один вердикт).
fn listed_keys(tail: &str) -> BTreeSet<String> {
    verdict_rows(tail, true)
        .0
        .iter()
        .flat_map(|(keys, _)| keys.clone())
        .collect()
}

/// Строки-скелет для позиций без вердикта (Q88): по одной на позицию перечня — модули затем
/// функции поверхности. Статус пустой, чтобы черновик не притворялся вердиктом. Без явного
/// флага — ни одной строки: рукописный блок принадлежит человеку (Q47.1).
fn draft_rows(generated: &str, tail: &str) -> Vec<String> {
    let listed = listed_keys(tail);
    let mut rows = Vec::new();
    for (typ, keys) in [
        ("модуль", position_keys(generated)),
        ("функция", function_keys(generated)),
    ] {
        for key in keys {
            if !listed.contains(&key) {
                rows.push(format!(
                    "| `{key}` | {typ} |  | подсказка: статус из легенды |"
                ));
            }
        }
    }
    rows
}

/// Вставить строки в конец таблицы вердиктов рукописного блока. Своей таблицы у генератора
/// нет (Q47.1), поэтому отсутствие таблицы — красным, а не новая таблица от имени человека.
fn insert_draft_rows(tail: &str, rows: &[String]) -> Result<String, String> {
    if rows.is_empty() {
        return Ok(tail.to_string());
    }
    let mut lines: Vec<String> = tail.split('\n').map(String::from).collect();
    let head = lines
        .iter()
        .position(|l| l.trim() == "| Позиция | Тип | Статус | Пометка |")
        .ok_or("в рукописном блоке нет таблицы вердиктов — правь руками (Q47.1)")?;
    if !lines
        .get(head + 1)
        .is_some_and(|l| l.trim_start().starts_with("|--"))
    {
        return Err("у таблицы вердиктов нет строки разделителя".to_string());
    }
    let mut at = head + 2;
    while lines
        .get(at)
        .is_some_and(|l| l.trim_start().starts_with('|'))
    {
        at += 1;
    }
    lines.splice(at..at, rows.iter().cloned());
    Ok(lines.join("\n"))
}

/// Лист с вставленным черновиком и число строк; `None` — добавлять нечего. Граница
/// генератора обязана пережить перезапись: без неё следующий `parity --check` краснеет
/// «лист правился вне генератора», а человек получил бы сломанный лист вместо строк.
fn sheet_with_draft(text: &str) -> Result<(Option<String>, usize), String> {
    let Some((generated, tail)) = text.split_once(GEN_END) else {
        return Err("лист без генерируемого блока".to_string());
    };
    let rows = draft_rows(generated, tail);
    if rows.is_empty() {
        return Ok((None, 0));
    }
    let count = rows.len();
    Ok((
        Some(format!(
            "{generated}{GEN_END}{}",
            insert_draft_rows(tail, &rows)?
        )),
        count,
    ))
}

/// Сколько чего насчитал лист. Модули и функции — два числа, а не одно (Q87): сложение
/// «модулей .c» и «публичных функций» дало бы дробь, которую нельзя ни назвать, ни проверить.
#[derive(Default, Debug, PartialEq, Eq)]
struct SheetCounts {
    positions: usize,
    covered: usize,
    functions: usize,
    functions_covered: usize,
    /// Строки `--draft` без статуса: не вердикты, но и не потерянные (Q88).
    drafts: usize,
}

/// Позиции публичной поверхности (Q87): раздел `## Функции (N) — публичная поверхность
/// `каталог``, ключ `каталог::имя`. Функция принадлежит библиотеке, а не каталогу `.c`.
/// Раздел «ведётся руками» (`Level::OwnApi`) имён не содержит — ключей ноль.
fn function_keys(generated: &str) -> Vec<String> {
    let mut lines = generated.lines();
    let heading = match lines.find(|l| l.starts_with("## Функции")) {
        Some(h) => h,
        None => return Vec::new(),
    };
    // Каталог поверхности напечатан в заголовке обратными кавычками; без него ключ
    // был бы просто именем и столкнулся бы с одноимёнными функциями другой библиотеки.
    let dir = match backticked(heading).first() {
        Some(dir) => dir.clone(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in lines {
        if line.starts_with("## ") || line.contains(GEN_END) {
            break;
        }
        for name in backticked(line) {
            out.push(format!("{dir}::{name}"));
        }
    }
    out
}

/// Ключи строк вердиктов, которых нет в перечне листа (Q93): опечатка, переезд файла в
/// другой каталог или переименованная функция иначе дают «вердикт есть, покрытия нет» —
/// строку, неотличимую от ненаписанной. Черновик `--draft` сверяется наравне с вердиктом:
/// сдвинулся перечень — устаревшая строка красна, а не молчит. Ключ функции (`каталог::имя`)
/// сверяется, только если лист обязан иметь функции: у листа с руками ведённой поверхностью
/// их список пишет человек, и проверять ворота нечем (названный пробел).
fn unknown_verdict_keys(
    rows: &[(Vec<String>, String)],
    positions: &[String],
    functions: &[String],
    judge_functions: bool,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (keys, _) in rows {
        for key in keys {
            let известна = if key.contains("::") {
                !judge_functions || functions.contains(key)
            } else {
                positions.contains(key)
            };
            if !известна && !out.contains(key) {
                out.push(key.clone());
            }
        }
    }
    out
}

/// Не покрыты те ключи, которых нет среди вердиктов (Q86: точное совпадение ключа).
fn uncovered<'a>(keys: &'a [String], decided: &BTreeMap<&String, &String>) -> Vec<&'a String> {
    keys.iter().filter(|k| !decided.contains_key(k)).collect()
}

/// Правила одного листа (ТЗ 5), отделённые от обхода файлов: негативные тесты (Q53) гоняют
/// настоящее правило, а не его пересказ.
fn sheet_problems(
    row: &MapRow,
    head: &str,
    generated: &str,
    tail: &str,
) -> (Vec<String>, SheetCounts) {
    let mut problems = Vec::new();
    if !generated.contains(&format!("HEAD `{head}`")) {
        problems.push(format!(
            "{}: лист устарел — зеркало сдвинулось на `{head}` (обновить: `cargo run -p xtask -- parity`)",
            row.kreyt
        ));
    }
    // Позиции и вердикты (Q85, Q86): покрыт только тот ключ перечня, который назван в строке
    // таблицы точным совпадением. Ни подстрока (`menu.c` внутри `window-menu.c`), ни упоминание
    // в прозе «Причина:» вердиктом не являются.
    let positions = position_keys(generated);
    // Легенда статусов обязана быть в рукописном блоке каждого листа (Q83): без неё
    // «статус вне легенды» нечем проверить. Старая легенда, где `готово` обещано без оговорки,
    // красна по той же причине — она противоречит ТЗ 6.9.
    if !tail.contains(LEGEND_MARKER) {
        problems.push(format!(
            "{}: {} легенды статусов (Q83) — `cargo run -p xtask -- parity --legend`",
            row.kreyt,
            if tail.contains(LEGEND_OLD_PREFIX) {
                "рукописный блок сохранил прежнюю"
            } else {
                "в рукописном блоке нет"
            }
        ));
    } else if legend_promise_without_smoke(tail) {
        problems.push(format!(
            "{}: легенда обещает `готово` без зелёного smoke — противоречит ТЗ 6.9 (Q83)",
            row.kreyt
        ));
    }
    let (rows, verdict_problems, drafts) = verdict_rows(tail, smoke_proven());
    // Претензия к строке таблицы без имени листа в ленте ворот неотличима от чужой.
    problems.extend(
        verdict_problems
            .into_iter()
            .map(|s| format!("{}: {s}", row.kreyt)),
    );
    let mut decided: BTreeMap<&String, &String> = BTreeMap::new();
    for (keys, status) in &rows {
        if !status_ok(status) {
            continue; // черновик (Q88): статус не выбран, вердикта нет, покрытия нет
        }
        for key in keys {
            // Один ключ — ровно один вердикт (Q84): группа не должна прятать конфликт статусов.
            if let Some(prev) = decided.get(key) {
                if *prev != status {
                    problems.push(format!(
                        "{}: позиция `{key}` имеет два вердикта: «{prev}» и «{status}» (Q84)",
                        row.kreyt
                    ));
                }
            } else {
                decided.insert(key, status);
            }
        }
    }
    let missing = uncovered(&positions, &decided);
    // Функции — отдельный счёт (Q87): свои ключи, своё покрытие, и в `positions` они не входят.
    let functions = function_keys(generated);
    let missing_functions = uncovered(&functions, &decided);
    // Вердикт обязан называть позицию перечня этим самым ключом (Q93): без этой сверки
    // опечатка в ключе выглядит как написанный вердикт — покрытие молчит, ворота зелёные.
    let лишние_ключи = unknown_verdict_keys(
        &rows,
        &positions,
        &functions,
        !functions.is_empty() || matches!(row.level, Level::Modules),
    );
    if !лишние_ключи.is_empty() {
        let названные = лишние_ключи
            .iter()
            .take(3)
            .map(|s| format!("`{s}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let ещё = if лишние_ключи.len() > 3 {
            format!(" … и ещё {}", лишние_ключи.len() - 3)
        } else {
            String::new()
        };
        problems.push(format!(
            "{}: вердикт по ключу, которого нет в перечне листа: {названные}{ещё} — \
             позиция называется точным ключом перечня (Q93)",
            row.kreyt
        ));
    }
    // Судьба листа — поле, а не статус (Q83): оно не покрывает, но обязано быть целым и
    // называть позицию перечня — иначе текст ошибки отправляет в никуда.
    let ключи_судьбы: Vec<String> = positions.iter().chain(functions.iter()).cloned().collect();
    problems.extend(
        fate_problems(tail, &ключи_судьбы)
            .into_iter()
            .map(|s| format!("{}: {s}", row.kreyt)),
    );
    if !zero_sheet_ok(tail, positions.len()) {
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
    // Штамп закрытия (Q89): формат, коммит, предковость main, тег и отсутствующий smoke.
    let closing = parse_closing(tail);
    if let Some(stamp) = &closing {
        problems.extend(
            closing_problems(
                stamp,
                smoke_proven(),
                commit_exists,
                ancestor_of_main,
                tag_commit,
            )
            .into_iter()
            .map(|s| format!("{}: {s}", row.kreyt)),
        );
    }
    if closing.is_some() && !missing_functions.is_empty() {
        problems.push(format!(
            "{}: лист закрыт, но вердиктов нет на {} функци(й) поверхности: {} — закрывать нельзя (Q87)",
            row.kreyt,
            missing_functions.len(),
            missing_functions
                .iter()
                .take(3)
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if closing.is_some() && !missing.is_empty() {
        problems.push(format!(
            "{}: лист закрыт, но вердиктов нет на {} позици(й): {} — закрывать нельзя",
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
    (
        problems,
        SheetCounts {
            positions: positions.len(),
            covered: positions.len() - missing.len(),
            functions: functions.len(),
            functions_covered: functions.len() - missing_functions.len(),
            drafts,
        },
    )
}

/// Непустое поле рукописного блока: `Причина: <текст>` (ТЗ 5).
fn field_filled(tail: &str, field: &str) -> bool {
    tail.lines().any(|l| {
        l.split_once(field)
            .is_some_and(|(_, rest)| !rest.trim().is_empty())
    })
}

/// Поле `Судьба:` рукописного блока (ТЗ 6.3, Q83): `Судьба: <ключ> — <текст>`, по строке на
/// позицию. Поле ничего не покрывает: готовность живёт только в таблице вердиктов, иначе
/// «заменено конструкцией» закрывала бы позицию без вердикта человека. Ключ сверяется с
/// перечнем тем же правилом, что и вердикт (Q93), иначе опечатка живёт зелёной строкой.
fn fate_problems(tail: &str, keys: &[String]) -> Vec<String> {
    let mut problems = Vec::new();
    for line in tail.lines().map(str::trim) {
        let Some(остаток) = line.strip_prefix("Судьба:") else {
            continue;
        };
        let Some((ключ, текст)) = остаток.split_once('—') else {
            problems.push(format!(
                "поле `Судьба:` без текста судьбы — формат `Судьба: <ключ> — <текст>` (Q83): {line}"
            ));
            continue;
        };
        let ключ = ключ.trim().trim_matches('`');
        let текст = текст.trim();
        if ключ.is_empty() || текст.is_empty() {
            problems.push(format!(
                "поле `Судьба:` заполнено наполовину — формат `Судьба: <ключ> — <текст>` (Q83): {line}"
            ));
        } else if !keys.iter().any(|k| k == ключ) {
            problems.push(format!(
                "`Судьба:` по ключу, которого нет в перечне листа: `{ключ}` — позиция \
                 называется точным ключом перечня (Q93)"
            ));
        }
    }
    problems
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
    // Реестр читается из рабочего дерева, а `head` — из коммита. Некоммиченная правка
    // судится против прошлого состояния, и вердикт обязан это называть.
    let dirty = git_in(
        root(),
        &[
            "status",
            "--porcelain",
            "--",
            p("canon.toml").to_str().unwrap_or(""),
        ],
    )
    .is_some_and(|s| !s.trim().is_empty());
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
    if dirty {
        println!(
            "  · canon.toml изменён и не закоммичен — реестр прочитан из рабочего дерева, а голова взята из HEAD"
        );
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
            // Собственный тег — случай 4, а не чужеродный тег случая 1: дерево не может
            // содержать запись о собственном sha, и запись, сделанная сразу после тега,
            // смотрит ровно в него.
            Some(actual) if actual == head => notes.push(format!(
                "{tag}: вне сверки — собственный тег проверяемого состояния"
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
/// Коммит существует в этом репозитории (`git cat-file -e <sha>^{commit}`, Q89).
fn commit_exists(sha: &str) -> bool {
    commit_exists_in(&p("."), sha)
}

/// То же в указанном дереве: негативный кейс предковости строится на одноразовом
/// репозитории с чужой веткой, а не на рабочем дереве проекта (тикет 06).
fn commit_exists_in(dir: &Path, sha: &str) -> bool {
    git_in(dir, &["cat-file", "-e", &format!("{sha}^{{commit}}")]).is_some()
}

/// Коммит — предок `main` (`git merge-base --is-ancestor <sha> main`, Q89). В shallow-clone
/// без `main` ответ — «нет»: ворота не имеют права считать закрытие доказанным, когда
/// предковость нельзя проверить.
fn ancestor_of_main(sha: &str) -> bool {
    ancestor_of_main_in(&p("."), sha)
}

fn ancestor_of_main_in(dir: &Path, sha: &str) -> bool {
    git_in(dir, &["merge-base", "--is-ancestor", sha, "main"]).is_some()
}

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
    tag_commit_in(&p("."), tag)
}

/// Коммит, на который указывает тег (peel), в указанном дереве. `--verify` требует ровно
/// один объект (иначе git печатал бы и само имя), `--quiet` — чтобы отсутствующий тег был
/// `None`, а не руганью в stderr. Имя тега, начавшееся с флага, тоже даёт `None` — то есть
/// красную строку, а не ложное закрытие (Q89).
fn tag_commit_in(dir: &Path, tag: &str) -> Option<String> {
    git_in(
        dir,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{tag}^{{commit}}"),
        ],
    )
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
/// Больше не используется правилом листа (Q85: ключ — путь, а не имя); оставлен до тикета 02
/// как часть разбора поверхности.
#[allow(
    dead_code,
    reason = "имена больше не ключ; путь разбора поверхности — тикет 02 (Q87)"
)]
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
        let first = splice("# Титл", &fresh, None, &handwritten_stub()).unwrap();
        let with_verdicts = first.replace(
            "|---|---|---|---|",
            "|---|---|---|---|\n| `src/window.c` | модуль | готово | smoke |",
        );
        let again = splice(
            "# Новый титл",
            &fresh,
            Some(&with_verdicts),
            &handwritten_stub(),
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
                &handwritten_stub()
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
        let first = splice("# Титл", &with, None, &handwritten_stub()).unwrap();
        let hand = "\nПричина: публичной поверхности нет.\nПереносится: только API крейта.\n";
        let заготовка = handwritten_stub();
        let first = first.replacen(&заготовка, &format!("{заготовка}{hand}"), 1);
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
            position_keys(&generated).is_empty(),
            "фикстура обязана быть нулевой — иначе тест ничего не проверяет"
        );
        let bare = splice("# Титл", &generated, None, &handwritten_stub()).unwrap();
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
        let bare = splice("# Титл", &generated, None, &handwritten_stub()).unwrap();
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
        let sheet = splice("# Титл", &generated, None, &handwritten_stub()).unwrap();
        // Штамп с настоящим коммитом: проверка «sha существует / предок main» ходит в git,
        // и выдуманный sha краснил бы лист по другой причине.
        let коммит = коммит_репозитория();
        let closed = format!("{sheet}\nЗакрыт: 2026-11-15, main@{коммит}\n");
        let (g, tail) = closed.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail)
                .0
                .iter()
                .any(|s| s.contains("лист закрыт")),
            "храповик закрытия не сработал"
        );
        // Статусы — `нет`, не `готово`: закрытый лист с `готово` красен по smoke (Q83),
        // а здесь проверяется именно храповик покрытия.
        let full = format!(
            "{closed}| `src/panel.c` | модуль | нет | |\n| `src/window.c` | модуль | нет | |\n| `lib/util.c` | модуль | нет | |\n"
        );
        let (g, tail) = full.split_once(GEN_END).unwrap();
        let проблем = sheet_problems(&row, "deadbee", g, tail).0;
        assert!(
            !проблем.iter().any(|s| s.contains("закрывать нельзя")),
            "закрытие при полном покрытии не может ругаться покрытием: {проблем:?}"
        );
        // Зелёным такой лист быть не может: зелёного smoke-прогона в этом вороте нет (Q83).
        assert!(
            проблем.iter().any(|s| s.contains("smoke")),
            "закрытый лист без доказанного smoke обязан быть красным: {проблем:?}"
        );
        assert!(
            sheet_problems(&row, "stale1", g, tail)
                .0
                .iter()
                .any(|s| s.contains("лист устарел")),
            "уехавший отпечаток обязан краснеть"
        );
        // `готово` до первого зелёного smoke закрытый лист не спасает: sha коммита —
        // необходимое, но не достаточное условие (Q83).
        let gotovo = full.replace("| модуль | нет | |", "| модуль | готово | |");
        let (g, tail) = gotovo.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail)
                .0
                .iter()
                .any(|s| s.contains("smoke")),
            "`готово` до smoke обязано краснить закрытый лист"
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
            rotation_problem(std::slice::from_ref(&old), "new@k").unwrap(),
            "canon-r9: последний канон-тег подписан ключом `old@k`, действует `new@k` — ротация не доиграна"
        );
        assert!(rotation_problem(std::slice::from_ref(&fresh), "new@k").is_none());
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

        // Запись, сделанная сразу после тега, смотрит в сам проверяемый коммит. Это
        // случай 4, а не «чужеродный тег»: без этой ветки коммит, возвращающий запись в
        // реестр, краснеет по несуществующей причине.
        let own = vec![(
            "canon-r9".to_string(),
            "head-sha".to_string(),
            "k".to_string(),
        )];
        let (problems, notes) = canon_verdicts(
            &own,
            &["canon-r9".to_string()],
            |_| Some("head-sha".to_string()),
            "head-sha",
            |_| false,
        );
        assert!(
            problems.is_empty(),
            "собственный тег не может быть красным: {problems:?}"
        );
        assert_eq!(
            notes,
            vec!["canon-r9: вне сверки — собственный тег проверяемого состояния".to_string()]
        );
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

    fn имена(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// Перечень позиций с шапкой зеркала на нужном HEAD — чтобы судить правило листа
    /// без отдельной претензии «лист устарел».
    fn перечень(repo: &str, head: &str, table: &str) -> String {
        format!("\nЗеркало: `mate-desktop/{repo}` · HEAD `{head}` · модулей (`.c`): 0\n\n{table}")
    }

    /// Краснота без зеркала: зелёный цвет от нечего проверять — ровно тот самоход, каким
    /// сегодня сворачивается сверка парити-листов, когда `vendor/mate` нет (Q57).
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn без_зеркала_шаг_парити_красен() {
        let карта = vec!["marco", "mate-panel", "caja"];
        let err = parity_verdict(&ParityState {
            rows: карта.len(),
            without_mirror: &карта,
            positions: 3,
        })
        .expect_err("без зеркала ни один лист не сверен — это красная строка, а не пропуск");
        assert!(
            err.contains("зеркала нет"),
            "краснота обязана назвать причину: {err}"
        );
    }

    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn пропущенный_лист_карты_назван_в_вердикте() {
        let err = parity_verdict(&ParityState {
            rows: 3,
            without_mirror: &["marco"],
            positions: 9,
        })
        .expect_err("одна пропущенная строка карты — это неполная сверка");
        assert!(
            err.contains("marco"),
            "пропуск обязан быть назван по имени: {err}"
        );
    }

    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn пустая_карта_красна_а_не_зелена() {
        let пусто: Vec<&str> = Vec::new();
        let err = parity_verdict(&ParityState {
            rows: 0,
            without_mirror: &пусто,
            positions: 0,
        })
        .expect_err("пустая карта — не доказательство, а пустой прогон");
        assert!(
            err.contains("пустая"),
            "краснота обязана назвать причину: {err}"
        );
    }

    #[test]
    fn полное_зеркало_зелено() {
        assert!(
            parity_verdict(&ParityState {
                rows: 2,
                without_mirror: &[],
                positions: 5,
            })
            .is_ok(),
            "все строки карты с зеркалом — шаг зелёный"
        );
    }

    /// Метрика (Q82): лист без перечня позиций не даёт шагу зелёного цвета `0/0`.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn ноль_позиций_красен_а_не_зеленый_ноль() {
        let err = parity_verdict(&ParityState {
            rows: 4,
            without_mirror: &[],
            positions: 0,
        })
        .expect_err("нulle позиций — сводить покрытие нечем");
        assert!(
            err.contains("позиций") && err.contains("нечего"),
            "краснота обязана назвать причину: {err}"
        );
    }

    /// Ключ позиции — `каталог/файл` (Q85): базовые имена в одном листе не различимы.
    #[test]
    fn ключ_позиции_путь_а_не_имя() {
        let generated = перечень(
            "mate-session",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `applet` | `main.c` |\n| `capplet` | `main.c` |\n| `.` | `startup.c` |\n",
        );
        let keys = position_keys(&generated);
        assert_eq!(
            keys,
            vec!["applet/main.c", "capplet/main.c", "startup.c"],
            "одноимённые модули разных каталогов — разные позиции; корень без точки"
        );
    }

    /// Таблица перечня не должна терять строки и не должна брать в позиции список функций.
    #[test]
    fn перечень_считает_модули_а_не_поверхность() {
        let g = render_generated(
            &MapRow {
                repo: "mate-fake".to_string(),
                kreyt: "fake".to_string(),
                maylston: "М0".to_string(),
                level: Level::Functions("src".to_string()),
            },
            "deadbee",
            &modules_of(&fixture()),
            &["fake_widget_new".to_string(), "fake_widget_ref".to_string()],
            false,
        );
        assert_eq!(
            position_keys(&g).len(),
            3,
            "фикстура даёт три модуля — функции в перечень позиций не входят"
        );
    }

    /// Вердикт = строка таблицы (Q86): подстрока длинного имени короткую позицию не покрывает.
    /// Мутация «`contains` вместо точного совпадения» обязана это покраснить.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn подстрока_не_покрывает_позицию() {
        let row = MapRow {
            repo: "mate-menu".to_string(),
            kreyt: "menuvy".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        };
        let generated = перечень(
            "mate-menu",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `.` | `acpi-plugin.c`, `ibm-acpi-plugin.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `ibm-acpi-plugin.c` | модуль | в работе | |\n"
        ));
        let (problems, counts) = sheet_problems(&row, "deadbee", &generated, &tail);
        let (positions, covered) = (counts.positions, counts.covered);
        assert!(
            problems.is_empty(),
            "вердикт по ключу легален: {problems:?}"
        );
        assert_eq!(
            (positions, covered),
            (2, 1),
            "вердикт обязан покрить ровно одну позицию"
        );
        let verdict = sheet_problems(
            &row,
            "deadbee",
            &generated,
            &format!("{tail}\nЗакрыт: 2026-11-18\n"),
        )
        .0;
        assert!(
            verdict.iter().any(|s| s.contains("acpi-plugin.c")),
            "непокрытая позиция обязана быть названа ключом: {verdict:?}"
        );
    }

    /// Вердикт = строка таблицы (Q86): упоминание в прозе «Причина:» ничего не покрывает.
    /// Мутация «покрывает любое упоминание» обязана это покраснить.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn проза_вердиктом_не_считается() {
        let row = MapRow {
            repo: "mate-session".to_string(),
            kreyt: "skhodka".to_string(),
            maylston: "М2".to_string(),
            level: Level::OwnApi,
        };
        let generated = перечень(
            "mate-session",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `capplet` | `main.c` |\n",
        );
        let prose = &format!(
            "{}\nПричина: `capplet/main.c` — обёртка, переносится оверрайдом.\n",
            пустая_таблица()
        );
        let (problems, counts) = sheet_problems(&row, "deadbee", &generated, prose);
        let (positions, covered) = (counts.positions, counts.covered);
        assert!(
            problems.is_empty(),
            "объяснённый лист «руками» легален: {problems:?}"
        );
        assert_eq!(
            (positions, covered),
            (1, 0),
            "упоминание в прозе — не вердикт (Q86)"
        );
    }

    /// Групповой вердикт (Q84) раскрывается поимённо: одна строка покрывает названные ключи.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn групповой_вердикт_раскрывается_поимённо() {
        let row = MapRow {
            repo: "mate-menus".to_string(),
            kreyt: "traktir".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        };
        let generated = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c`, `menu-monitor.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-monitor.c`, `libmenu/menu-util.c` | модуль | нет | |\n"
        ));
        let (problems, counts) = sheet_problems(&row, "deadbee", &generated, &tail);
        let (positions, covered) = (counts.positions, counts.covered);
        assert!(
            problems.is_empty(),
            "групповой вердикт легален: {problems:?}"
        );
        assert_eq!((positions, covered), (2, 2), "строка назвала обе позиции");
    }

    /// Повтор ключа с другим статусом красен (Q84) — иначе группа прячет конфликт.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn конфликт_статусов_одного_ключа_красен() {
        let row = MapRow {
            repo: "mate-menus".to_string(),
            kreyt: "traktir".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        };
        let generated = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | нет | |\n| `libmenu/menu-util.c` | модуль | готово | |\n"
        ));
        let (problems, _) = sheet_problems(&row, "deadbee", &generated, &tail);
        assert!(
            problems.iter().any(|s| s.contains("два вердикта")),
            "конфликт статусов обязан краснеть: {problems:?}"
        );
    }

    /// Вердикт по ключу, которого нет в перечне листа, красен и называет этот ключ (Q93):
    /// иначе опечатка в имени файла выглядит как написанный вердикт — ворота зелёные, а
    /// покрытие молчит (`лишний` ключ не покрывает ничего, и это надо видеть).
    #[test]
    fn вердикт_по_ключу_вне_перечня_красен() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu_util.c` | модуль | нет | |\n"
        ));
        let (problems, counts) = sheet_problems(&row, "deadbee", &g, &tail);
        assert!(
            problems
                .iter()
                .any(|s| s.contains("нет в перечне") && s.contains("libmenu/menu_util.c")),
            "ключ не из перечня обязан краснить лист и называться: {problems:?}"
        );
        assert_eq!(
            (counts.positions, counts.covered),
            (1, 0),
            "вердикт по несуществующей позиции покрытие не двигает"
        );
    }

    /// Черновик `--draft` сверяется с перечнем наравне с вердиктом (Q93): сдвинулся перечень
    /// — устаревшая строка-скелет красна, а не ждёт, пока человек напишет над ней вердикт.
    #[test]
    fn черновик_со_старым_ключом_красен() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль |  | подсказка |\n| `libmenu/staryj.c` | модуль |  | подсказка |\n"
        ));
        let (problems, counts) = sheet_problems(&row, "deadbee", &g, &tail);
        assert!(
            problems
                .iter()
                .any(|s| s.contains("нет в перечне") && s.contains("libmenu/staryj.c")),
            "устаревший черновик обязан краснеть: {problems:?}"
        );
        assert!(
            !problems.iter().any(|s| s.contains("libmenu/menu-util.c")),
            "свежий черновик не обязан краснеть из-за соседа: {problems:?}"
        );
        assert_eq!(
            (counts.positions, counts.covered, counts.drafts),
            (1, 0, 2),
            "обе строки остались черновиками, вердиктов нет"
        );
    }

    /// Ключ функции сверяется с генерируемым разделом «Функции» (Q93): функция могла
    /// стать статической или уехать из заголовка — вердикт над ней обязан краснеть.
    #[test]
    fn ключ_функции_сверяется_с_разделом_функций() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М1".to_string(),
            level: Level::Functions("src".to_string()),
        };
        let g = render_generated(
            &row,
            "deadbee",
            &modules_of(&fixture()),
            &["fake_widget_new".to_string()],
            false,
        );
        let проблемы = |ключ: &str| {
            let tail = хвост(&format!("{ЗАГОЛОВОК}| `{ключ}` | функция | нет | |\n"));
            sheet_problems(&row, "deadbee", &g, &tail).0
        };
        let свой = проблемы("src::fake_widget_new");
        assert!(
            свой.iter().all(|s| !s.contains("нет в перечне")),
            "функция из перечня поверхности не обязана краснеть: {свой:?}"
        );
        let лишко = проблемы("src::fake_widget_removed");
        assert!(
            лишко
                .iter()
                .any(|s| s.contains("нет в перечне") && s.contains("src::fake_widget_removed")),
            "вердикт по функции, которой нет в поверхности, красен: {лишко:?}"
        );
    }

    /// Функционный ключ там, где его быть не может, красен; но лист, который ведёт
    /// поверхность руками (Q56), воротам не проверяем — названный пробел, а не зелёный.
    #[test]
    fn лист_без_функционального_слоя_не_придумывает_функций() {
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu::menu_verbose` | функция | нет | |\n"
        ));
        let (problems, _) = sheet_problems(&лист_модулей(), "deadbee", &g, &tail);
        assert!(
            problems.iter().any(|s| s.contains("нет в перечне")),
            "у листа без функционального слоя ключ функции красен: {problems:?}"
        );

        let руками = MapRow {
            level: Level::OwnApi,
            ..лист_модулей()
        };
        let (problems, _) = sheet_problems(&руками, "deadbee", &g, &tail);
        assert!(
            !problems.iter().any(|s| s.contains("нет в перечне")),
            "у листа с руками ведённой поверхностью список функций пишет человек, \n             воротам нечем проверять (названный пробел): {problems:?}"
        );
    }

    /// Статус вне легенды или пустой красен (Q86): пустой вердикт не даёт покрытию расти.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn статус_вне_легенды_красен() {
        let row = MapRow {
            repo: "mate-menus".to_string(),
            kreyt: "traktir".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        };
        let generated = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        // Пустой статус здесь не проверяется: с Q88 он черновик, а не статус вне легенды
        // (`пустой_статус_черновик_а_не_вердикт`).
        let status = "почти готово";
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | {status} | |\n"
        ));
        let (problems, counts) = sheet_problems(&row, "deadbee", &generated, &tail);
        assert!(
            problems.iter().any(|s| s.contains("легенды")),
            "статус не из легенды: статус «{status}» обязан краснеть: {problems:?}"
        );
        assert_eq!(counts.covered, 0, "покрытию расти не на чем");
    }

    /// Пустой статус — черновик `--draft` (Q88), а не строка вне легенды: вердиктом он не
    /// становится, но и красным не является — человек до строки ещё не дошёл.
    #[test]
    fn пустой_статус_черновик_а_не_вердикт() {
        let row = MapRow {
            repo: "mate-menus".to_string(),
            kreyt: "traktir".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        };
        let generated = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль |  | |\n"
        ));
        let (problems, c) = sheet_problems(&row, "deadbee", &generated, &tail);
        assert!(
            problems.is_empty(),
            "черновик не обязан быть красным: {problems:?}"
        );
        assert_eq!(
            (c.positions, c.covered, c.drafts),
            (1, 0, 1),
            "ключ занят строкой, но вердикта нет: покрытие 0/1, черновик посчитан"
        );
    }

    /// `--draft` (Q88): по одной строке на позицию без вердикта — модули, затем функции
    /// поверхности; статус пустой, чтобы черновик не притворялся вердиктом.
    #[test]
    fn черновик_это_строки_модулей_и_функций() {
        let g = лист_библиотеки(&["fake_widget_new", "fake_widget_ref"]);
        assert_eq!(
            draft_rows(&g, ПУСТАЯ_ТАБЛИЦА),
            vec![
                "| `lib/util.c` | модуль |  | подсказка: статус из легенды |",
                "| `src/panel.c` | модуль |  | подсказка: статус из легенды |",
                "| `src/window.c` | модуль |  | подсказка: статус из легенды |",
                "| `libfake::fake_widget_new` | функция |  | подсказка: статус из легенды |",
                "| `libfake::fake_widget_ref` | функция |  | подсказка: статус из легенды |",
            ],
            "порядок перечня, статус пустой, у функций свой тип"
        );
    }

    /// Идемпотентность (Q88): повторный прогон не добавляет ни строки; заполненный статус и
    /// проза рукописного блока остаются как были.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn черновик_идемпотентен_заполненное_не_трогает() {
        let g = лист_библиотеки(&["fake_widget_new"]);
        let tail = format!(
            "{ПУСТАЯ_ТАБЛИЦА}| `src/window.c` | модуль | в работе | | ждём API |\n\n\
             Причина: `lib/util.c` — не переносим.\n"
        );
        let rows = draft_rows(&g, &tail);
        assert_eq!(
            rows,
            vec![
                "| `lib/util.c` | модуль |  | подсказка: статус из легенды |",
                "| `src/panel.c` | модуль |  | подсказка: статус из легенды |",
                "| `libfake::fake_widget_new` | функция |  | подсказка: статус из легенды |",
            ],
            "позиция с вердиктом черновику не отдаётся"
        );
        let after = insert_draft_rows(&tail, &rows).unwrap();
        assert!(
            after.contains("| `src/window.c` | модуль | в работе | | ждём API |"),
            "заполненный статус не трогать: {after}"
        );
        assert!(
            after.contains("Причина: `lib/util.c`"),
            "прозу рукописного блока не трогать: {after}"
        );
        let второй = draft_rows(&g, &after);
        assert!(
            второй.is_empty(),
            "второй прогон не добавляет строк: {второй:?}"
        );
        assert_eq!(insert_draft_rows(&after, &второй).unwrap(), after);
    }

    /// Без флага — ни одной строки (Q88, Q47.1): обычный путь перегенерации рукописный блок
    /// не трогает, даже когда таблица вердиктов пуста.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn без_флага_черновик_строк_не_появляется() {
        let fresh = лист_библиотеки(&["fake_widget_new"]);
        let first = splice("# Титл", &fresh, None, &handwritten_stub()).unwrap();
        let again = splice("# Титл", &fresh, Some(&first), "").unwrap();
        assert_eq!(
            again, first,
            "перегенерация без флага обязана оставить хвост байт в байт"
        );
        let (_, хвост) = first.split_once(GEN_END).unwrap();
        assert!(
            !draft_rows(&fresh, хвост).is_empty(),
            "фикстура обязана быть без вердиктов, иначе тест ничего не проверяет"
        );
    }

    /// Лист обязан пережить перезапись (sign-off тикета 03): граница генератора на месте,
    /// по одной строке на позицию, второй прогон не меняет файл.
    #[test]
    #[allow(
        clippy::expect_used,
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn черновик_переписывает_лист_не_теряя_границы() {
        let fresh = лист_библиотеки(&["fake_widget_new"]);
        let лист = splice("# Титл", &fresh, None, &handwritten_stub()).unwrap();
        let (один, добавлено) = sheet_with_draft(&лист).unwrap();
        let один = один.expect("лист без единого вердикта обязан получить черновик");
        assert_eq!(добавлено, 4, "три модуля и одна функция поверхности");
        let (generated, tail) = один
            .split_once(GEN_END)
            .expect("граница генератора обязана пережить перезапись");
        assert_eq!(
            tail.matches("| `").count(),
            4,
            "по одной строке на позицию: {tail}"
        );
        assert_eq!(
            position_keys(generated).len(),
            3,
            "генерируемый блок не тронут"
        );
        let (второй, ещё) = sheet_with_draft(&один).unwrap();
        assert!(
            второй.is_none() && ещё == 0,
            "второй прогон не добавляет строк"
        );
    }

    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn закрытый_список_красен_без_зеркала() {
        let empty = BTreeSet::new();
        let err = icons_verdict(&IconsState {
            mirror: None,
            derived: &empty,
            listed: &имена(&["edit-cut"]),
        })
        .expect_err("без зеркала список не сверен — это красная строка, а не пропуск");
        assert!(
            err.contains("зеркала нет"),
            "краснота обязана назвать причину: {err}"
        );
        // Пустой список и пустое зеркало — тот же случай: зелёный от нечего проверять.
        assert!(
            icons_verdict(&IconsState {
                mirror: None,
                derived: &empty,
                listed: &empty,
            })
            .is_err(),
            "ни имён, ни зеркала — сверки нет, и это нельзя показать зелёным"
        );
    }

    /// Лишнее имя красит: в листе оно есть, а зеркало его не знает — список обещает то,
    /// чего не существует.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn лишнее_имя_красит_сверку_иконок() {
        let err = icons_verdict(&IconsState {
            mirror: Some("deadbee"),
            derived: &имена(&["edit-cut"]),
            listed: &имена(&["edit-cut", "user-desktop"]),
        })
        .expect_err("имени нет в зеркале");
        assert!(err.contains("лишнее имя «user-desktop»"), "{err}");
    }

    /// Отсутствующее имя красит: зеркало его знает, а в листе нет — М2 обещает меньше,
    /// чем забирает жизнь. Вторая половина двусторонней сверки (Q62).
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn отсутствующее_имя_красит_сверку_иконок() {
        let err = icons_verdict(&IconsState {
            mirror: Some("deadbee"),
            derived: &имена(&["edit-cut", "user-desktop"]),
            listed: &имена(&["edit-cut"]),
        })
        .expect_err("зеркало знает имя, которого нет в списке");
        assert!(err.contains("отсутствующее имя «user-desktop»"), "{err}");
    }

    /// Зелёная строка несёт число из прогона и отпечаток зеркала: число без команды, которая
    /// его сняла, не факт (Q61), а лист обязан соответствовать отпечатку (Q47.3).
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn зелёная_строка_иконок_несёт_число_и_отпечаток() {
        let line = icons_verdict(&IconsState {
            mirror: Some("deadbee"),
            derived: &имена(&["edit-copy", "edit-cut"]),
            listed: &имена(&["edit-copy", "edit-cut"]),
        })
        .expect("списки совпадают");
        assert_eq!(line, "icons: 2 имён · зеркало deadbee · список совпадает");
    }

    /// Мини-зеркало кормит сверку без `vendor/mate/` (тот в `.gitignore`), а битое имя в нём
    /// красит сверку именно как правило формы, а не как дифф: имена поданы с обеих сторон
    /// одинаковые.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn битое_имя_в_мини_зеркале_красит_сверку() {
        let mirror = p("xtask/tests/fixtures/icons-mirror");
        let derived = derived_icons(
            &mirror.join("mate-icon-theme"),
            &[mirror.join("mate-panel")],
        );
        assert_eq!(
            derived,
            имена(&["edit-cut", "edit--copy"]),
            "мини-зеркало читается без vendor/mate; имени нет в теме — не имя"
        );
        let err = icons_verdict(&IconsState {
            mirror: Some("deadbee"),
            derived: &derived,
            listed: &derived,
        })
        .expect_err("зеркало знает имя, которое приложение не примет");
        assert!(err.contains("испорченное имя «edit--copy»"), "{err}");
    }

    /// Имена зависимостей из манифеста. `[dev-dependencies]` вне правила: зависимость теста
    /// не попадает в собранную библиотеку и не тянет ничего к потребителю. Имя стоит либо
    /// ключом в секции `dependencies` / `build-dependencies` / `target.*.dependencies`,
    /// либо хвостом заголовка (`[dependencies.foo]`).
    fn manifest_dependencies(manifest: &str) -> Vec<String> {
        let mut names = Vec::new();
        let mut keys = false;
        for line in manifest.lines() {
            let line = line.trim();
            if let Some(head) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let head = head.trim_matches(|c| c == '\'' || c == '"').to_string();
                keys = false;
                if head.contains("dev-dependencies") {
                    continue;
                }
                if let Some((parent, tail)) = head.rsplit_once('.') {
                    if tail != "dependencies"
                        && (parent == "dependencies" || parent.ends_with(".dependencies"))
                    {
                        names.push(tail.to_string());
                        continue;
                    }
                    keys = head.ends_with(".dependencies");
                    continue;
                }
                keys = head == "dependencies" || head == "build-dependencies";
                continue;
            }
            if !keys || line.starts_with('#') {
                continue;
            }
            if let Some((name, _)) = line.split_once('=') {
                let name = name.trim().trim_matches(|c| c == '\'' || c == '"');
                if !name.is_empty() {
                    names.push(name.to_string());
                }
            }
        }
        names
    }

    /// Разборщик правила графа сам проверяется: правило, у которого граф всегда пуст,
    /// зелёно при любом содержимом манифеста (Q57).
    #[test]
    fn зависимости_манифеста_видны_разборщику_графа() {
        let got = manifest_dependencies(
            "[package]\nname = \"x\"\n\n\
             [dependencies]\n\
             # комментарий не имя\n\
             edit-lib = { path = \"../edit\" }\n\n\
             [dependencies.svetlitsa-efl]\nversion = \"1\"\n\n\
             [target.'cfg(unix)'.dependencies]\nlibc-sys = \"1\"\n\n\
             [dev-dependencies]\ntmpfile = \"1\"\n\n\
             [build-dependencies]\nbindgen = \"1\"\n\n\
             [lints]\nworkspace = true\n",
        );
        assert_eq!(
            got,
            vec![
                "edit-lib".to_string(),
                "svetlitsa-efl".to_string(),
                "libc-sys".to_string(),
                "bindgen".to_string(),
            ],
            "граф читается в обеих формах записи, тестовые зависимости вне его"
        );
    }

    /// Шестая строка воробей-таблицы (раунд 10, Q68). Доказательством является не пустой
    /// `Cargo.toml`, а этот тест: стоит положить в граф `svetlitsa-efl` или любой `*-sys`,
    /// либо завести `build.rs`, как он краснеет. Иначе «слой имён собирается там, где EFL нет»
    /// остаётся обещанием, которое никто не проверяет.
    ///
    /// ponytail: проверяются прямые зависимости манифеста. Пока у крейта ни одной, транзитивного
    /// замыкания нет по построению; с первой же зависимостью переходить на `cargo metadata --deps`.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn граф_слоя_имён_не_знает_про_efl_и_не_собирается_кодом() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("корень воркспейса")
            .to_path_buf();
        let crate_dir = root.join("crates/svetlitsa-icons");
        let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
            .expect("манифест слоя имён читается");

        let deps = manifest_dependencies(&manifest);
        for dep in &deps {
            assert!(
                dep != "svetlitsa-efl",
                "слой имён потянул `svetlitsa-efl`: предикат больше нельзя собрать там, где EFL нет"
            );
            assert!(
                !dep.ends_with("-sys"),
                "слой имён потянул системную привязку `{dep}`: она требует заголовков и ломает М1"
            );
        }
        assert!(
            !crate_dir.join("build.rs").exists(),
            "у слоя имён появился build.rs: сборка стала зависеть от окружения, а не только от кода"
        );
    }

    // --- Тикет 02 раунда 12: функции публичной поверхности считаются отдельно (Q87) ---

    /// Генерируемый блок листа библиотеки: три модуля фикстуры + две функции поверхности.
    fn лист_библиотеки(functions: &[&str]) -> String {
        render_generated(
            &MapRow {
                repo: "mate-fake".to_string(),
                kreyt: "fake".to_string(),
                maylston: "М1".to_string(),
                level: Level::Functions("libfake".to_string()),
            },
            "deadbee",
            &modules_of(&fixture()),
            &functions.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
            false,
        )
    }

    fn лист_без_поверхности() -> String {
        render_generated(
            &MapRow {
                repo: "mate-fake".to_string(),
                kreyt: "fake".to_string(),
                maylston: "М1".to_string(),
                level: Level::Modules,
            },
            "deadbee",
            &modules_of(&fixture()),
            &[],
            false,
        )
    }

    /// Шапка таблицы вердиктов.
    const ЗАГОЛОВОК: &str = "| Позиция | Тип | Статус | Пометка |\n|---|---|---|---|\n";

    /// Рукописный блок тестового листа: обязательная легенда статусов (Q83) плюс таблица.
    /// Тестовые хвосты собираются вокруг неё, иначе каждый тест отвлекается на красную легенду.
    fn хвост(таблица: &str) -> String {
        format!(
            "\n## Вердикты (ручной блок — генератор его не трогает)\n{LEGEND_TEXT}\n\n{таблица}"
        )
    }

    /// Настоящий коммит этого репозитория: штамп закрытия проверяет sha через git, и
    /// выдуманный sha краснил бы лист по другой причине.
    #[allow(
        clippy::expect_used,
        reason = "без git тест всё равно ничего не проверяет"
    )]
    fn коммит_репозитория() -> String {
        let от = Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(p("."))
            .output()
            .expect("`git rev-parse` доступен");
        assert!(от.status.success(), "git rev-parse HEAD молчит");
        String::from_utf8_lossy(&от.stdout).trim().to_string()
    }

    /// Таблица вердиктов без легенды — для тестов черновика, где легенда не проверяется.
    const ПУСТАЯ_ТАБЛИЦА: &str = "\n| Позиция | Тип | Статус | Пометка |\n|---|---|---|---|\n";

    /// Таблица вердиктов без строк — как в только что созданном листе.
    fn пустая_таблица() -> String {
        хвост(ЗАГОЛОВОК)
    }

    fn все_модули() -> String {
        хвост(&format!(
            "{ЗАГОЛОВОК}| `lib/util.c` | модуль | нет | |\n| `src/panel.c` | модуль | нет | |\n| `src/window.c` | модуль | нет | |\n"
        ))
    }

    /// Ключ функции — `каталог::имя` (Q87): функция принадлежит библиотеке, а не каталогу
    /// `.c`, и токены перечня модулей в список функций не попадают.
    #[test]
    fn ключ_функции_каталог_двоеточие_имя() {
        let g = лист_библиотеки(&["fake_widget_new", "fake_widget_ref"]);
        assert_eq!(
            function_keys(&g),
            vec!["libfake::fake_widget_new", "libfake::fake_widget_ref"],
            "ключ функции — библиотека::имя, а не голое имя: одинаковые имена двух библиотек \
             слились бы в одну позицию"
        );
        assert!(
            function_keys(&лист_без_поверхности()).is_empty(),
            "у листа уровня «модули» публичной поверхности нет — ключей ноль"
        );
    }

    /// Две дроби, а не одна (Q87): покрытая функция двигает только свой счётчик.
    /// Мутация «модули и функции в один счётчик» обязана покраснить именно этот тест.
    #[test]
    fn функции_считаются_отдельно_от_модулей() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М1".to_string(),
            level: Level::Functions("libfake".to_string()),
        };
        let g = лист_библиотеки(&["fake_widget_new", "fake_widget_ref"]);
        let tail = format!(
            "{}| `libfake::fake_widget_new` | функция | в работе | |\n",
            все_модули()
        );
        let (problems, c) = sheet_problems(&row, "deadbee", &g, &tail);
        assert!(problems.is_empty(), "открытый лист легален: {problems:?}");
        assert_eq!(
            (c.positions, c.covered, c.functions, c.functions_covered),
            (3, 3, 2, 1),
            "модули 3/3 и функции 1/2 — два независимых числа; сложение дало бы (5,4)"
        );
    }

    /// Лист библиотеки не закрывается, пока поверхность не покрыта `M/M` (Q87): иначе
    /// «закрыт» значил бы, что публичных функций не касались.
    #[test]
    fn лист_функций_не_закрывается_без_покрытия_функций() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М1".to_string(),
            level: Level::Functions("libfake".to_string()),
        };
        let g = лист_библиотеки(&["fake_widget_new", "fake_widget_ref"]);
        let tail = format!(
            "{}\nЗакрыт: 2026-11-22, коммит порта `012abcd`\n",
            все_модули()
        );
        let (problems, c) = sheet_problems(&row, "deadbee", &g, &tail);
        assert_eq!(
            (c.positions, c.covered, c.functions, c.functions_covered),
            (3, 3, 2, 0),
            "модули покрыты полностью — краснеть обязано именно отсутствие функций"
        );
        assert!(
            problems
                .iter()
                .any(|s| s.contains("функци(й)") && s.contains("libfake::fake_widget_new")),
            "закрытие при непокрытой поверхности красное и называет функцию: {problems:?}"
        );
    }

    /// Ноль поверхности — напечатанный ноль, а не красный лист: у 24 листов из 31 публичных
    /// C-функций нет, и выдумывать их генератор не имеет права (Q87).
    #[test]
    fn лист_без_поверхности_не_придумывает_функций() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        };
        let g = лист_без_поверхности();
        let (problems, c) = sheet_problems(&row, "deadbee", &g, &все_модули());
        assert!(
            problems.is_empty(),
            "лист без поверхности легален: {problems:?}"
        );
        assert_eq!(
            (c.positions, c.covered, c.functions, c.functions_covered),
            (3, 3, 0, 0),
            "ноль функций при полном покрытии модулей — не проблема"
        );
    }

    // ——— Q83: легенда, `готово` и smoke ———

    /// Строка карты для листа модулей (`mate-menus` / `traktir`).
    fn лист_модулей() -> MapRow {
        MapRow {
            repo: "mate-menus".to_string(),
            kreyt: "traktir".to_string(),
            maylston: "М1".to_string(),
            level: Level::Modules,
        }
    }

    /// Таблица вердиктов без легенды: правило «статус вне легенды» нечем проверить, поэтому
    /// красен весь лист (Q83).
    #[test]
    fn легенда_обязательна_в_листе() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = format!("{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | нет | |\n");
        let проблемы = sheet_problems(&row, "deadbee", &g, &tail).0;
        assert!(
            проблемы.iter().any(|s| s.contains("легенды статусов")),
            "лист без легенды обязан быть красным: {проблемы:?}"
        );
    }

    /// Прежняя легенда обещала `готово` без оговорки про smoke — ровно та надпись в 31 листе,
    /// которая спорила с ТЗ 6.9. такая легенда красна (Q83).
    #[test]
    fn легенда_без_оговорки_про_smoke_красна() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        // Легенда без слова `smoke` — не та, что пишет генератор, а прежняя надпись листа.
        let tail = format!(
            "\n## Вердикты (ручной блок — генератор его не трогает)\n\
             Статусы позиции: `нет` / `в работе` / `отложено (причина)` / `готово`.\n\n\
             {ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | нет | |\n"
        );
        let проблемы = sheet_problems(&row, "deadbee", &g, &tail).0;
        assert!(
            проблемы.iter().any(|s| s.contains("противоречит ТЗ 6.9")),
            "легенда без smoke обязана краснить лист: {проблемы:?}"
        );
    }

    /// `отложено` без причины в скобках — не статус, и человек обязан получить причину,
    /// а не общий текст «не из легенды» (Q83).
    #[test]
    fn отложено_без_причины_красно() {
        assert!(!status_ok("отложено"), "обещание без причины не статус");
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | отложено | |\n"
        ));
        let проблемы = sheet_problems(&row, "deadbee", &g, &tail).0;
        assert!(
            проблемы.iter().any(|s| s.contains("не статус")),
            "`отложено` без причины красен воротами: {проблемы:?}"
        );
        let проблема = status_problem("отложено");
        assert!(проблема.contains("причины"), "{проблема}");
        assert!(!проблема.contains("не из легенды"), "{проблема}");
        assert!(
            status_ok("отложено (ждём API)"),
            "причина в скобках — статус"
        );
    }

    /// «Заменено конструкцией» — судьба листа, а не готовность позиции (Q83): подсказка
    /// обязана говорить про поле `Судьба:`.
    #[test]
    fn судьба_листа_не_статус_позиции() {
        let проблема = status_problem("заменено конструкцией");
        assert!(проблема.contains("Судьба:"), "{проблема}");
        assert!(проблема.contains("судьба листа"), "{проблема}");
    }

    /// Поле `Судьба:` существует не только в тексте ошибки (ТЗ 6.3, Q83): валидная строка
    /// `Судьба: <ключ> — <текст>` не красна и покрытие не двигает — судьба не вердикт.
    #[test]
    fn судьба_валидна_и_не_покрывает_позицию() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = format!(
            "{}\nСудьба: `libmenu/menu-util.c` — заменено конструкцией `MenuTree`\n",
            хвост(&format!(
                "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | нет | |\n"
            ))
        );
        let (проблемы, счёт) = sheet_problems(&row, "deadbee", &g, &tail);
        assert!(
            !проблемы.iter().any(|s| s.contains("Судьба")),
            "валидная судьба не обязана краснеть: {проблемы:?}"
        );
        assert_eq!(
            (счёт.positions, счёт.covered),
            (1, 1),
            "покрытие держит таблица, а поле — при нём"
        );
    }

    /// Ключ судьбы обязан быть позицией перечня (Q93): отосланная в поле опечатка иначе
    /// живёт зелёной строкой и ничего не объясняет.
    #[test]
    fn судьба_по_ключу_вне_перечня_красна() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = format!(
            "{}\nСудьба: `libmenu/net-util.c` — заменено конструкцией\n",
            хвост(&format!(
                "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | нет | |\n"
            ))
        );
        let (проблемы, _) = sheet_problems(&row, "deadbee", &g, &tail);
        assert!(
            проблемы.iter().any(|s| s.contains("Судьба")
                && s.contains("нет в перечне")
                && s.contains("libmenu/net-util.c")),
            "ключ судьбы вне перечня красен: {проблемы:?}"
        );
    }

    /// Строка без ` — ` между ключом и текстом — не судьба, а незаполненное поле: красна,
    /// иначе лист выглядит разобранным там, где человек остановился.
    #[test]
    fn судьба_без_текста_красна() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        for строка in [
            "Судьба: `libmenu/menu-util.c`",
            "Судьба: `libmenu/menu-util.c` — ",
            "Судьба:  — заменено конструкцией",
        ] {
            let tail = format!(
                "{}\n{строка}\n",
                хвост(&format!(
                    "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | нет | |\n"
                ))
            );
            let (проблемы, _) = sheet_problems(&row, "deadbee", &g, &tail);
            assert!(
                проблемы.iter().any(|s| s.contains("Судьба")),
                "строка «{строка}» обязана быть красной: {проблемы:?}"
            );
        }
    }

    /// `готово` живёт только после зелёного smoke-прогона (ТЗ 6.9, Q83): до него статус
    /// красен, после — обычный.
    #[test]
    fn готово_до_smoke_красно() {
        let tail = хвост(&format!(
            "{ЗАГОЛОВОК}| `libmenu/menu-util.c` | модуль | готово | |\n"
        ));
        let (rows, до_smoke, _) = verdict_rows(&tail, false);
        assert_eq!(rows.len(), 1, "строка прочитана");
        assert!(
            до_smoke.iter().any(|s| s.contains("готово")),
            "`готово` до smoke обязано краснить: {до_smoke:?}"
        );
        let (_, после_smoke, _) = verdict_rows(&tail, true);
        assert!(
            после_smoke.is_empty(),
            "после smoke статус обычный: {после_smoke:?}"
        );
    }

    // ——— Q89: штамп закрытия ———

    /// Штамп читается целиком: дата, sha без префикса `main@`, тег канона.
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    #[test]
    fn штамп_закрытия_читается() {
        let штамп = parse_closing("\nЗакрыт: 2026-11-16, main@012abcd4 · canon-r12\n").unwrap();
        assert_eq!(штамп.date, "2026-11-16");
        assert_eq!(штамп.sha, Some("012abcd4"));
        assert_eq!(штамп.tag, Some("canon-r12"));
        assert_eq!(штамп.raw, "Закрыт: 2026-11-16, main@012abcd4 · canon-r12");
    }

    /// Лист без штампа открыт — и это не «плохого штампа нет» (Q89).
    #[test]
    fn лист_без_штампа_открыт() {
        assert!(parse_closing(&хвост(ЗАГОЛОВОК)).is_none());
    }

    /// «Коммита нет» вместо sha — не закрытие (Q89).
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    #[test]
    fn штамп_без_коммита_красен() {
        let штамп = parse_closing("\nЗакрыт: 2026-11-16, —\n").unwrap();
        let проблемы = closing_problems(&штамп, true, |_| true, |_| true, |_| None);
        assert!(
            проблемы.iter().any(|s| s.contains("без коммита")),
            "{проблемы:?}"
        );
    }

    /// Дата штампа обязана быть датой (Q89): «когда-нибудь» проверяемым условием не является.
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    #[test]
    fn дата_штампа_должна_быть_датой() {
        let штамп = parse_closing("\nЗакрыт: 2026-11, main@012abcd4\n").unwrap();
        let проблемы = closing_problems(&штамп, true, |_| true, |_| true, |_| None);
        assert!(
            проблемы.iter().any(|s| s.contains("не ГГГГ-ММ-ДД")),
            "{проблемы:?}"
        );
    }

    /// Предковость `main` проверяется: коммит другой линии закрытие не доказывает (Q89).
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    #[test]
    fn штамп_обязан_быть_предком_main() {
        let штамп = parse_closing("\nЗакрыт: 2026-11-16, main@012abcd4\n").unwrap();
        let проблемы = closing_problems(&штамп, true, |_| true, |_| false, |_| None);
        assert!(
            проблемы.iter().any(|s| s.contains("не предок main")),
            "{проблемы:?}"
        );
    }

    /// Тег приписан — значит он обязан существовать и указывать на тот же коммит (Q89).
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    #[test]
    fn тег_штампа_обязан_совпадать_с_коммитом() {
        let штамп = parse_closing("\nЗакрыт: 2026-11-16, main@012abcd4 · canon-r12\n").unwrap();
        let проблемы = closing_problems(&штамп, true, |_| true, |_| true, |_| None);
        assert!(
            проблемы
                .iter()
                .any(|s| s.contains("тега canon-r12") || s.contains("тег canon-r12")),
            "отсутствующий тег красен: {проблемы:?}"
        );
        let проблемы = closing_problems(
            &штамп,
            true,
            |_| true,
            |_| true,
            |_| Some("ffffffffffffffff".to_string()),
        );
        assert!(
            проблемы.iter().any(|s| s.contains("canon-r12")),
            "чужой тег красен: {проблемы:?}"
        );
    }

    /// Настоящая проводка: `sheet_problems` обязан звать git, а не принимать штамп на слово
    /// (Q89). Выдуманный sha и несуществующий тег красны именно через ворота листа.
    #[test]
    fn штамп_закрытия_сверяется_с_git() {
        let row = лист_модулей();
        let g = перечень(
            "mate-menus",
            "deadbee",
            "| Каталог | Модули |\n|---|---|\n| `libmenu` | `menu-util.c` |\n",
        );
        let tail = format!(
            "{}\n| `libmenu/menu-util.c` | модуль | нет | |\n\n\
             Закрыт: 2026-11-16, main@deadbeefdeadbeef · canon-r99\n",
            хвост(ЗАГОЛОВОК)
        );
        let проблемы = sheet_problems(&row, "deadbee", &g, &tail).0;
        assert!(
            проблемы
                .iter()
                .any(|s| s.contains("deadbeefdeadbeef") && s.contains("нет в репозитории")),
            "коммита нет — штамп красен: {проблемы:?}"
        );
        assert!(
            проблемы.iter().any(|s| s.contains("canon-r99")),
            "тега нет — штамп красен: {проблемы:?}"
        );
    }

    /// Негативный кейс на настоящем git (Q89): воробей-проверка подставляет в `closing_problems`
    /// свои ответы, и обёртки над git могли отвечать «всегда истина» — зелёным был бы и штамп
    /// по коммиту чужой ветки. Здесь одноразовый репозиторий с настоящей веткой `side`.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn предок_main_сверяется_настоящим_git() {
        let метка = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "svetlitsa-xtask-git-{}-{метка}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            let mut полные = vec![
                "-c",
                "user.email=xtask@localhost",
                "-c",
                "user.name=xtask",
                "-c",
                "commit.gpgsign=false",
            ];
            полные.extend_from_slice(args);
            git_in(&dir, &полные)
        };
        assert!(
            git(&["init", "-q", "-b", "main"]).is_some(),
            "`git init` недоступен"
        );
        assert!(git(&["commit", "-q", "--allow-empty", "-m", "main"]).is_some());
        let предок = git(&["rev-parse", "HEAD"]).unwrap();
        assert!(git(&["checkout", "-q", "-b", "side"]).is_some());
        assert!(git(&["commit", "-q", "--allow-empty", "-m", "side"]).is_some());
        let чужой = git(&["rev-parse", "HEAD"]).unwrap();
        assert!(git(&["tag", "canon-t", &предок]).is_some());

        assert!(commit_exists_in(&dir, &предок), "коммит main существует");
        assert!(
            !commit_exists_in(&dir, "012abcd40000000000000000000000000000000"),
            "выдуманного коммита нет"
        );
        assert!(
            ancestor_of_main_in(&dir, &предок),
            "коммит main — предок main"
        );
        assert!(
            !ancestor_of_main_in(&dir, &чужой),
            "коммит чужой ветки не предок main — ради этого кейса и затевался прогон"
        );
        assert_eq!(
            tag_commit_in(&dir, "canon-t").as_deref(),
            Some(предок.as_str()),
            "peel тега обязан называть тот же коммит"
        );

        let строка = format!("\nЗакрыт: 2026-11-16, main@{чужой} · canon-t\n");
        let штамп = parse_closing(&строка).unwrap();
        let проблемы = closing_problems(
            &штамп,
            true,
            |s| commit_exists_in(&dir, s),
            |s| ancestor_of_main_in(&dir, s),
            |t| tag_commit_in(&dir, t),
        );
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            проблемы.iter().any(|s| s.contains("не предок main")),
            "штамп по коммиту чужой ветки красен на настоящем git: {проблемы:?}"
        );
    }

    /// Валидный sha — необходимое, но не достаточное условие: без зелёного smoke закрытие
    /// не доказано и лист красен (Q83/Q89).
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    #[test]
    fn валидный_sha_без_smoke_не_закрытие() {
        let штамп = parse_closing("\nЗакрыт: 2026-11-16, main@012abcd4 · canon-r12\n").unwrap();
        let все_хорошо = closing_problems(
            &штамп,
            true,
            |_| true,
            |_| true,
            |_| Some("012abcd4".to_string()),
        );
        assert!(все_хорошо.is_empty(), "{все_хорошо:?}");
        let проблемы = closing_problems(
            &штамп,
            false,
            |_| true,
            |_| true,
            |_| Some("012abcd4".to_string()),
        );
        assert!(
            проблемы.len() == 1 && проблемы[0].contains("smoke"),
            "единственная причина — отсутствие smoke: {проблемы:?}"
        );
    }
}
