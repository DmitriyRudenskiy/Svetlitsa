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
        Some("parity") => {
            if let Err(e) = parity(args.get(1).map(String::as_str)) {
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

/// Строка карты ТЗ 2: репозиторий → крейт → майлстоун.
struct MapRow {
    repo: String,
    kreyt: String,
    maylston: String,
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

/// Генерируемый блок: отпечаток зеркала + дерево модулей.
fn render_generated(row: &MapRow, head: &str, modules: &BTreeMap<String, Vec<String>>) -> String {
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
        let title = format!(
            "# Парити-лист `{}` ← `{}` ({})",
            row.kreyt, row.repo, row.maylston
        );
        let path = p(&format!("docs/parity/{}.md", row.kreyt));
        let existing = std::fs::read_to_string(&path).ok();
        let text = splice(
            &title,
            &render_generated(&row, &head, &modules),
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

/// Ворота парити (Q47.3): каждой позиции карты соответствует лист, отпечаток зеркала свежий.
/// Покрытие вердиктами считается и печатается, но красным пока не является: вердикт — работа
/// человека, а не генератора.
fn check_parity_sheets() -> Result<(), String> {
    if !p("vendor/mate").is_dir() {
        return Err(
            "vendor/mate нет — scripts/fetch-vendor.sh (парити-листы проверяются по зеркалам)"
                .to_string(),
        );
    }
    let mut problems = Vec::new();
    let mut verdicts = 0;
    let mut modules = 0;
    for row in load_map()? {
        let path = p(&format!("docs/parity/{}.md", row.kreyt));
        let Some(text) = std::fs::read_to_string(&path).ok() else {
            problems.push(format!(
                "{}: нет листа — `xtask parity {}`",
                row.kreyt, row.kreyt
            ));
            continue;
        };
        let head = mirror_head(&row.repo).unwrap_or_default();
        if head.is_empty() || !text.contains(&format!("HEAD `{head}`")) {
            problems.push(format!(
                "{}: лист устарел (зеркало `{head}`) — `xtask parity {}`",
                row.kreyt, row.kreyt
            ));
            continue;
        }
        modules += text[..text.find(GEN_END).unwrap_or(0)]
            .matches(".c`")
            .count();
        verdicts += text
            .rsplit_once(GEN_END)
            .map(|(_, tail)| tail.matches("| `").count())
            .unwrap_or_default();
    }
    println!(
        "парити: листов {}, модулей {}, вердиктов заполнено {verdicts} — покрытие {}%",
        load_map()?.len(),
        modules,
        verdicts * 100 / modules.max(1)
    );
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!("парити-листы:\n{}", problems.join("\n")))
    }
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
        };
        let fresh = render_generated(&row, "deadbee", &modules_of(&fixture()));
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
}
