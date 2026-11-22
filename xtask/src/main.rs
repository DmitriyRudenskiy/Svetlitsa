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
    Deps,
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
    Step::Deps,
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
            "deps" => Some(Self::Deps),
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
            Self::Deps => "зависимости платформы",
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
        Some("smoke") => {
            let Some(scenario) = args.get(1).filter(|s| !s.is_empty()).map(String::as_str) else {
                eprintln!(
                    "использование: cargo run -q -p xtask -- smoke <сценарий>\n\
                     сценарии — файлы в scripts/smoke/ (ТЗ 10.2), отпечаток прогона — в artifacts/"
                );
                exit(2);
            };
            if let Err(e) = smoke(scenario) {
                eprintln!("{e}");
                exit(1);
            }
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
        Step::Deps => check_dependency_policy(),
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
    // Пин — это **минимальный** rustc (ТЗ 10.1: «означает минимальную версию и
    // проверяется xtask ci --doctor …, а не устанавливает её»), поэтому сравниваем
    // как «не меньше», а не как точное совпадение: Alpine в образе даёт свежее пина.
    match (parse_rustc(&have), parse_rustc(&want)) {
        (Some(ime), Some(pin)) if ime < pin => {
            problems.push(format!("rustc: нужно не меньше {want}, найден «{have}»"))
        }
        (Some(_), Some(_)) => println!("rustc: пин {want} (минимум) пройден"),
        _ => problems.push(format!(
            "rustc: версию не разобрать: «{have}», пин «{want}»"
        )),
    }
    // `rsync` в этом списке был со времён переноса дерева в ВМ (до ADR-0013):
    // дерево монтируется в контейнер, переносить его нечем. `ssh-keygen` — наоборот,
    // требовался всегда: им проверяются штампы канона (Q58).
    for (bin, why) in [
        ("git", "зеркала и origin"),
        ("ssh-keygen", "проверка штампов канона (Q58)"),
    ] {
        if which(bin).is_none() {
            problems.push(format!("{bin} не найден — {why}"));
        }
    }
    // EFL/Elementary и композитор нужны только сборке UI и smoke-прогонам (ТЗ 3.2, 10.3).
    if pkg_config_version("elementary").is_none() {
        println!(
            "подсказка: Elementary 1.29 по pkg-config не найден — окно и smoke не собрать (scripts/dev-vm/efl-build.sh)"
        );
    }
    // Xvfb, xdotool и `import` в этом списке не было с 21.11.2026: они говорят по X11, а X11-сессии
    // в проекте нет (ADR-0016). Голое окно smoke-прогонов ставит headless-композитор (ТЗ 10.2).
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        println!(
            "подсказка: WAYLAND_DISPLAY не задан — smoke-прогоны и скриншоты не запустить; нужна голова от headless-композитора (`cage --` или `weston -Bheadless`) (ТЗ 10.2, ADR-0003)"
        );
    }
    if which("cage").is_none() && which("weston").is_none() {
        println!(
            "подсказка: ни `cage`, ни `weston` не найдены — нечем поднять голову smoke-прогонов (ТЗ 10.2)"
        );
    }
    if which("wayland-scanner").is_none() {
        println!(
            "подсказка: wayland-scanner не найден — привязки протоколов сеанса несгенерируемы (ТЗ 3.6)"
        );
    }
    // Место меряем там, где сборка и пишет: ФС контейнера и тома EFL/dist
    // (`scripts/build-in-docker.sh`). Дерево, примонтированное извне, через `df`
    // показывает чужой том (df_is_host_shadow), поэтому печатается отдельной
    // подсказкой, а не входит в вердикт.
    let mut measured: Vec<(String, f64)> = Vec::new();
    for path in [".", "/opt/svetlitsa-efl", "/opt/dist"] {
        match df_free(path) {
            Ok((src, gib)) if df_is_host_shadow(&src) => println!(
                "подсказка: {path} отдаёт {gib:.1} ГиБ источника {src} — это тень тома Docker Desktop, а не место сборки (ADR-0013)"
            ),
            Ok((src, gib)) => {
                println!("свободно {gib:.1} ГиБ на {path} (источник {src})");
                measured.push((path.to_string(), gib));
            }
            Err(e) => println!("подсказка: {e} — точка вне среды сборки, пропущена"),
        }
    }
    match measured.iter().min_by(|a, b| a.1.total_cmp(&b.1)) {
        Some((path, gib)) if *gib < MIN_FREE_GIB => problems.push(format!(
            "свободно {gib:.1} ГиБ на {path} — сборка EFL и target/ не помещаются (ТЗ 3.8: нужно ≥ {MIN_FREE_GIB:.0} ГиБ)"
        )),
        Some((path, gib)) => println!(
            "место в среде сборки: {gib:.1} ГиБ — самая тесная точка {path}, минимум из {}, порог {MIN_FREE_GIB:.0} ГиБ (ТЗ 3.8)",
            measured.len()
        ),
        None => println!(
            "подсказка: свободное место в среде сборки измерить нечем (одни тени); на хосте его печатает scripts/build-in-docker.sh"
        ),
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
/// Пакеты, которым вход в дерево закрыт (ТЗ 3.3, ADR-0016, ADR-0019). Правило — не «нуля C»,
/// а «одна привязка на подсистему»: запрещены именно X11-стекка и чужие UI-тулкиты.
const FORBIDDEN_DEPS: &[(&str, &str)] = &[
    ("x11rb", "X11-клиент: X11-ветки портов нет (ADR-0016)"),
    ("x11", "привязка Xlib: X11-сессии нет (ADR-0016)"),
    ("xcb", "XCB: X11-сессии нет (ADR-0016)"),
    ("x11-dl", "загрузка libX11: X11-сессии нет (ADR-0016)"),
    ("gtk", "UI только на EFL (ТЗ 3.3)"),
    ("gtk4", "UI только на EFL (ТЗ 3.3)"),
    (
        "libwnck",
        "список окон ведёт композитор `starshina` (ADR-0019)",
    ),
];

/// Ровно один звуковой слой (ADR-0022): pulse- и pipewire-привязки одновременно — это две
/// FFI-прослойки и два набора зависимостей ради стека, который проект использовать не будет.
const PULSE_DEPS: &[&str] = &[
    "libpulse-binding",
    "libpulse-sys",
    "libpulse_simple_binding",
];
const PIPEWIRE_DEPS: &[&str] = &["pipewire", "libpipewire-sys", "wireplumber"];

/// Имена пакетов из манифеста: ключи таблиц `[dependencies]`, `[dev-dependencies]`,
/// `[build-dependencies]`, `[workspace.dependencies]`. Разбор построчный: ради одного правила
/// ворот не тащить toml-зависимость в собственный инструмент.
fn manifest_dep_names(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut in_deps = false;
    for line in text.lines() {
        let line = line.trim();
        if let Some(head) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
            in_deps = head.ends_with("dependencies");
            continue;
        }
        if line.starts_with('#') || !in_deps {
            continue;
        }
        if let Some(name) = line.split_once('=').map(|(k, _)| k.trim())
            && !name.is_empty()
        {
            names.insert(name.replace('_', "-"));
        }
    }
    names
}

/// Что политика видит в собранном множестве пакетов — отдельно от обхода дерева (по образцу
/// `icons_verdict`: негативные тесты судят правило, а не его пересказ).
fn dependency_problems(deps: &BTreeSet<String>) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, why) in FORBIDDEN_DEPS {
        if deps.contains(*name) {
            problems.push(format!("{name}: запрещено — {why}"));
        }
    }
    let pulse: Vec<&str> = PULSE_DEPS
        .iter()
        .copied()
        .filter(|d| deps.contains(*d))
        .collect();
    let pw: Vec<&str> = PIPEWIRE_DEPS
        .iter()
        .copied()
        .filter(|d| deps.contains(*d))
        .collect();
    if !pulse.is_empty() && !pw.is_empty() {
        problems.push(format!(
            "звук: одновременно {} и {} — держим один слой (ADR-0022)",
            pulse.join(", "),
            pw.join(", ")
        ));
    }
    problems
}

/// Шаг ворот `deps`: X11-стекки и двух звуковых слоёв в дереве нет (ТЗ 3.3, ADR-0016, ADR-0022).
fn check_dependency_policy() -> Result<(), String> {
    let mut manifests = vec![p("Cargo.toml")];
    if let Ok(rd) = std::fs::read_dir(p("crates")) {
        for e in rd.flatten() {
            let m = e.path().join("Cargo.toml");
            if m.is_file() {
                manifests.push(m);
            }
        }
    }
    let mut deps = BTreeSet::new();
    for m in &manifests {
        if let Ok(text) = std::fs::read_to_string(m) {
            deps.extend(manifest_dep_names(&text));
        }
    }
    let problems = dependency_problems(&deps);
    if !problems.is_empty() {
        return Err(problems.join("; "));
    }
    println!(
        "зависимости: {} пакетов в манифестах — X11-стекки и двух звуковых слоёв нет (ADR-0016, ADR-0022)",
        deps.len()
    );
    Ok(())
}

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
/// Строка карты «Вне MATE» (ТЗ 2.3): C-оригинала нет по замыслу, поэтому зеркала у неё нет —
/// и требовать его нельзя. Правила рукописного блока исполняются как у всех.
fn has_mirror(row: &MapRow) -> bool {
    row.repo != "—"
}

/// HEAD из уже написанного листа — для строк без зеркала, где сравнивать отпечаток не с чем.
fn extract_head(text: &str) -> Option<String> {
    let rest = text.split_once("HEAD `")?.1;
    Some(rest.split('`').next().unwrap_or_default().to_string())
}

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

/// Механизмы X11 в C-оригинале: то, что в Wayland-порт **не переносится**, а заменяется.
/// Список закрытый и ведётся вместе с ТЗ 5.1 (ADR-0016): позиция парити-листа, помеченная
/// `X-механизм оригинала`, обязана называть Wayland-замену из правой колонки или причину отказа.
const X_MECHANISMS: &[(&str, &str)] = &[
    (
        "XOpenDisplay",
        "клиент X-сервера → клиент композитора (`wl_display`)",
    ),
    (
        "XkbGetKeyboard",
        "XKB-расширение → `xkb_keymap` композитору (ТЗ 9.3)",
    ),
    (
        "XkbLockGroup",
        "смена группы процесса → состояние ввода у композитора",
    ),
    (
        "XGrabKey",
        "глобальный grab → клавиатурные привязки композитора",
    ),
    (
        "XSendEvent",
        "посылка событий → протокол сеанса `org.svetlitsa.Session`",
    ),
    (
        "XConvertSelection",
        "X-selection → `zwlr-data-control` («Памятка», ТЗ 9.31)",
    ),
    ("XRRSetCrtcConfig", "RandR → `wlr-output-management`"),
    ("XShapeCombineRegion", "XShape → слой-маска композитора"),
    (
        "_NET_CLIENT_LIST",
        "EWMH → `org.svetlitsa.Session` + `ext-workspace-v1`",
    ),
    (
        "XEMBED",
        "XEmbed-трей → `org.kde.StatusNotifierItem` (ТЗ 9.7)",
    ),
    (
        "XSETTINGS",
        "XSETTINGS → `svetlitsa-config` + портал `Settings` (М5)",
    ),
    (
        "XScreenSaver",
        "XScreenSaver-extension → `ext-idle-notify-v1`/`ext-session-lock-v1`",
    ),
    (
        "DPMSForceLevel",
        "DPMS из клиента → управление выводом на стороне композитора",
    ),
    ("Xft.dpi", "Xft-настройки шрифта → `svetlitsa-config`"),
    (
        "ecore_x_",
        "`Ecore_X` → `Ecore_Wl2` (ADR-0004, уточнение 2026-11-21)",
    ),
];

/// Счётчик файлов оригинала, где встречается каждый механизм. Пустое отображение — «не найдено»,
/// отсутствие ключа — «не проверялось» (зеркала нет).
fn x_mechanisms_of(root: &Path) -> BTreeMap<&'static str, usize> {
    let mut found: BTreeMap<&'static str, usize> = BTreeMap::new();
    walk_filtered(root, &mut |path| {
        if !matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("c") | Some("h") | Some("cpp")
        ) {
            return;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        for (symbol, _) in X_MECHANISMS {
            if text.contains(*symbol) {
                *found.entry(*symbol).or_insert(0) += 1;
            }
        }
    });
    found
}

/// Раздел генерируемого блока: какие X-механизмы в оригинале найдены и чем они заменяются.
fn render_x_mechanisms(found: &BTreeMap<&'static str, usize>) -> String {
    let mut out = String::from("\n## X-механизмы оригинала (в порт не переносятся — ADR-0016)\n\n");
    if found.is_empty() {
        out.push_str(
            "Ни один из маркеров не найден — проверено по списку `X_MECHANISMS` (`xtask`).\n",
        );
        return out;
    }
    out.push_str("| Механизм | Файлов | Wayland-замена |\n|---|---|---|\n");
    for (symbol, note) in X_MECHANISMS {
        if let Some(n) = found.get(*symbol) {
            out.push_str(&format!("| `{symbol}` | {n} | {note} |\n"));
        }
    }
    out
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
    xmech: &BTreeMap<&'static str, usize>,
    functions: &[String],
    surface_fallback: bool,
) -> String {
    let total: usize = modules.values().map(Vec::len).sum();
    let mut out = format!("{GEN_BEGIN}\n");
    if row.repo == "—" {
        out.push_str(
            "\nЗеркало: **нет** — строка карты «Вне MATE» (ТЗ 2.3): C-оригинала нет, фактуру и X-механизмы ведёт человек.\n\n",
        );
    } else {
        out.push_str(&format!(
            "\nЗеркало: `mate-desktop/{}` · HEAD `{head}` · модулей (`.c`): {total}\n\n",
            row.repo
        ));
    }
    out.push_str("| Каталог | Модули |\n|---|---|\n");
    for (dir, files) in modules {
        let list: Vec<String> = files.iter().map(|f| format!("`{f}`")).collect();
        out.push_str(&format!("| `{dir}` | {} |\n", list.join(", ")));
    }
    if row.repo == "—" {
        out.push_str(
            "\n## X-механизмы оригинала (в порт не переносятся — ADR-0016)\n\nC-оригинала нет — проверять нечего (строка карты «Вне MATE», ТЗ 2.3).\n",
        );
    } else {
        out.push_str(&render_x_mechanisms(xmech));
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
        // Строка «Вне MATE» (ТЗ 2.3) перегенерируется и без зеркал — но только по имени:
        // молча проглотить отсутствие vendor/mate на полном прогоне было бы «почти зелёное».
        let only_without_mirror = match component {
            Some(c) => load_map()?.iter().any(|r| r.kreyt == c && !has_mirror(r)),
            None => false,
        };
        if !only_without_mirror {
            return Err("vendor/mate нет — сначала scripts/fetch-vendor.sh".to_string());
        }
    }
    let mut written = 0;
    for row in load_map()? {
        if component.is_some_and(|c| c != row.kreyt) {
            continue;
        }
        let Some(head) = mirror_head(&row.repo) else {
            if has_mirror(&row) {
                return Err(format!(
                    "{}: зеркала нет в vendor/mate — scripts/fetch-vendor.sh",
                    row.repo
                ));
            }
            // Строка «Вне MATE»: генерируемый блок без фактуры зеркала, рукописный — как есть.
            let title = format!(
                "# Парити-лист `{}` ← вне MATE ({})",
                row.kreyt, row.maylston
            );
            let path = p(&format!("docs/parity/{}.md", row.kreyt));
            let existing = std::fs::read_to_string(&path).ok();
            let text = splice(
                &title,
                &render_generated(&row, "—", &BTreeMap::new(), &BTreeMap::new(), &[], false),
                existing.as_deref(),
                HANDWRITTEN_STUB,
            )
            .map_err(|e| format!("{}: {e}", row.kreyt))?;
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", row.kreyt))?;
            written += 1;
            continue;
        };
        let mirror = p(&format!("vendor/mate/{}", row.repo));
        let modules = modules_of(&mirror);
        let xmech = x_mechanisms_of(&mirror);
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
            &render_generated(&row, &head, &modules, &xmech, &functions, surface_fallback),
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
    // Отпечатки прогонов живут в `artifacts/` (не в git): право на `готово` даёт прогон
    // на этой машине ворот, а не обещание (ТЗ 6.9, Q83).
    let proven_in_artifacts = |path: &str| smoke_run_problem(&p(path));
    let mut without_mirror: Vec<&str> = Vec::new();
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
            // Раньше здесь был `continue` с комментарием «вне зеркала — нечего и сверять»:
            // без зеркала он срабатывал на каждой строке, `problems` оставался пустым, и шаг
            // печатал зелёный итог, ничего не сверив. Пропуск собирается в вердикт (Q79).
            if !has_mirror(row) {
                // Кроме зеркала нет и отпечатка: HEAD сверяется с самим собой, остальные
                // правила листа (нулевой лист, причина ручной поверхности, вердикты) — полные.
                let own = extract_head(&text).unwrap_or_default();
                let Some((generated, tail)) = text.split_once(GEN_END) else {
                    problems.push(format!("{}: лист без генерируемого блока", row.kreyt));
                    continue;
                };
                let (found, modules, verdicts) =
                    sheet_problems(row, &own, generated, tail, &proven_in_artifacts);
                problems.extend(found);
                modules_total += modules;
                covered += verdicts;
                if tail.contains("\nЗакрыт:") {
                    closed += 1;
                } else {
                    open += 1;
                }
                continue;
            }
            without_mirror.push(row.repo.as_str());
            continue;
        };
        let Some((generated, tail)) = text.split_once(GEN_END) else {
            problems.push(format!("{}: лист без генерируемого блока", row.kreyt));
            continue;
        };
        let (found, modules, verdicts) =
            sheet_problems(row, &head, generated, tail, &proven_in_artifacts);
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
    // Вердикт о полноте сверки — после проблем по найденным листам: если зеркало на месте и
    // листы сверены, но часть строк карты всё же без зеркала, краснеть обязано именно это.
    parity_verdict(&ParityState {
        rows: rows.len(),
        without_mirror: &without_mirror,
    })?;
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

/// Что шаг `parity` увидел в дереве, отделённо от обхода файлов — по образцу `icons_verdict`:
/// негативные тесты (Q53) судят правило, а не его пересказ.
struct ParityState<'a> {
    rows: usize,
    without_mirror: &'a [&'a str],
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
    Ok(())
}

/// Правила одного листа (ТЗ 5), отделённые от обхода файлов: негативные тесты (Q53) гоняют
/// настоящее правило, а не его пересказ. Возвращает проблемы и счётчики (модули, покрыто).
/// `proven` — проверка отпечатка прогона (ТЗ 6.9, Q83); в воротах это файл из `artifacts/`,
/// в тестах — своя функция, чтобы правило не читало диск машины.
fn sheet_problems(
    row: &MapRow,
    head: &str,
    generated: &str,
    tail: &str,
    proven: &dyn Fn(&str) -> Option<String>,
) -> (Vec<String>, usize, usize) {
    let mut problems = Vec::new();
    // Пустой head — строка карты «Вне MATE» (ТЗ 2.3): отпечатка зеркала нет по замыслу,
    // сравнивать не с чем; все остальные правила листа исполняются.
    if !head.is_empty() && !generated.contains(&format!("HEAD `{head}`")) {
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
    problems.extend(gotovo_without_run_problems(&row.kreyt, tail, proven));
    (problems, modules.len(), modules.len() - missing.len())
}

/// Непустое поле рукописного блока: `Причина: <текст>` (ТЗ 5).
fn field_filled(tail: &str, field: &str) -> bool {
    tail.lines().any(|l| {
        l.split_once(field)
            .is_some_and(|(_, rest)| !rest.trim().is_empty())
    })
}

// ────────────────────────────── smoke (ТЗ 6.9, 10.2, Q83) ──────────────────────────────

/// Разобранный отпечаток прогона — то, что даёт строке листа право на `готово`.
/// Литерала, который можно переключить правкой, в коде нет: только файл прогона (Q83).
#[derive(Debug, Default, PartialEq)]
struct SmokeRun {
    /// Код возврата сценария.
    exit: Option<i32>,
    /// Снимок, обязанной лежать рядом (ТЗ 10.3): путь из отпечатка.
    screenshot: Option<String>,
}

/// Разбор отпечатка `artifacts/smoke-<сценарий>.txt` (`ключ: значение`, пустое значение —
/// «не было»). JSON не заводим: полей пять, а у `xtask` одна зависимость (закрытый список).
fn smoke_run_parse(text: &str) -> SmokeRun {
    let mut run = SmokeRun::default();
    for line in text.lines() {
        match line.split_once(':') {
            Some(("выход", v)) => run.exit = v.trim().parse().ok(),
            Some(("снимок", v)) => {
                run.screenshot = Some(v.trim().to_string()).filter(|s| !s.is_empty());
            }
            _ => {}
        }
    }
    run
}

/// Чем отпечаток не даёт права на `готово`; `None` — право есть.
fn smoke_run_problem(path: &Path) -> Option<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Some(format!("нет отпечатка прогона {}", path.display()));
    };
    let run = smoke_run_parse(&text);
    if run.exit != Some(0) {
        return Some(format!(
            "{}: прогон завершился кодом {:?}",
            path.display(),
            run.exit
        ));
    }
    match run.screenshot.as_deref() {
        None => Some(format!(
            "{}: в отпечатке нет снимка — права на «готово» нет (ТЗ 10.3, 6.9)",
            path.display()
        )),
        Some(shot) if std::fs::metadata(root().join(shot)).is_ok_and(|m| m.len() > 0) => None,
        Some(shot) => Some(format!(
            "{}: снимок {shot} пуст или отсутствует (ТЗ 10.3)",
            path.display()
        )),
    }
}

/// Строки со статусом `готово`, у которых права на него нет: в пометке обязан быть путь
/// к отпечатку прогона, и тот обязан читаться (ТЗ 6.9, Q83).
fn gotovo_without_run_problems(
    kreyt: &str,
    tail: &str,
    proven: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for line in tail.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        // `Позиция | Тип | Статус | Пометка`; разделитель `|---|` и текст вне таблицы мимо.
        if cells.len() < 6 || cells[3] != "готово" {
            continue;
        }
        let position = cells[1].trim_matches('`');
        match cells[4].split_whitespace().find(|w| w.starts_with("artifacts/")) {
            None => problems.push(format!(
                "{kreyt}: «{position}» помечена «готово» без отпечатка прогона — права нет (ТЗ 6.9, Q83)"
            )),
            Some(fingerprint) => {
                problems.extend(
                    proven(fingerprint).map(|p| format!("{kreyt}: «{position}» помечена «готово», но {p}")),
                );
            }
        }
    }
    problems
}

/// `xtask smoke <сценарий>` — обёртка над `scripts/smoke/<сценарий>.sh` (ТЗ 10.2): чужой
/// код возврата плюс отпечаток прогона в `artifacts/`. Красный прогон отпечатка не оставляет.
fn smoke(scenario: &str) -> Result<(), String> {
    let script = p(&format!("scripts/smoke/{scenario}.sh"));
    if !script.is_file() {
        let known = std::fs::read_dir(p("scripts/smoke"))
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().to_str().map(str::to_string))
                    .filter(|n| n.ends_with(".sh"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        return Err(format!(
            "нет сценария {}; сценарии в scripts/smoke/: {known}",
            script.display()
        ));
    }
    let dir = p("artifacts/smoke");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let started = std::time::SystemTime::now();
    let code = Command::new("sh")
        .arg(&script)
        .status()
        .map_err(|e| format!("sh {}: {e}", script.display()))?
        .code()
        .unwrap_or(-1);
    let files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|f| f.is_file())
        .collect();
    let mut names: Vec<String> = files
        .iter()
        .filter_map(|f| f.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    // Засчитывается только кадр, появившийся во время этого прогона: вчерашний снимок
    // не доказывает сегодняшнюю готовность.
    let shot = files
        .iter()
        .filter(|f| f.extension().is_some_and(|e| e == "png"))
        .filter(|f| {
            std::fs::metadata(f)
                .and_then(|m| m.modified())
                .is_ok_and(|t| t >= started)
        })
        .max_by_key(|f| {
            std::fs::metadata(f)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
        });
    if code != 0 {
        return Err(format!(
            "сценарий {scenario} завершился кодом {code} — отпечатка нет, права на «готово» тоже"
        ));
    }
    let shot_rel = shot.map(|f| {
        f.strip_prefix(root())
            .unwrap_or(f.as_path())
            .to_string_lossy()
            .into_owned()
    });
    let fingerprint = p(&format!("artifacts/smoke-{scenario}.txt"));
    let date = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let commit = git_in(root(), &["rev-parse", "--short", "HEAD"]).unwrap_or_default();
    let text = format!(
        "# отпечаток smoke-прогона — право листа на «готово» (ТЗ 6.9, Q83)\n\
         сценарий: {scenario}\n\
         скрипт: scripts/smoke/{scenario}.sh\n\
         выход: {code}\n\
         коммит: {commit}\n\
         дата: {date}\n\
         снимок: {}\n\
         артефакты: {}\n",
        shot_rel.clone().unwrap_or_default(),
        names.join(" ")
    );
    std::fs::write(&fingerprint, &text).map_err(|e| format!("{}: {e}", fingerprint.display()))?;
    match smoke_run_problem(&fingerprint) {
        None => println!(
            "smoke {scenario}: прогон зелёный, отпечаток {} — право на «готово» есть",
            fingerprint.display()
        ),
        Some(problem) => println!(
            "smoke {scenario}: прогон зелёный, но {problem} — этот прогон права на «готово» не даёт.\n\
             отпечаток: {} (снимок снимает композитор, ТЗ 10.3; контрактный путь — `starshina`, М2)",
            fingerprint.display()
        ),
    }
    Ok(())
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

/// Порог ТЗ 3.8 (Q84): сборка EFL и `target/` не влезают в меньшее.
const MIN_FREE_GIB: f64 = 60.0;

/// `1.97.1` из `rustc 1.97.1 (…)` или из `channel = "1.97.1"`. Выпуск с суффиксом
/// (`1.100.0-r0`, `1.98.0-beta.2`) разбирается до старших цифр каждого поля.
fn parse_rustc(text: &str) -> Option<(u64, u64, u64)> {
    let digits = |s: &str| {
        s.split(|c: char| !c.is_ascii_digit())
            .next()
            .filter(|v| !v.is_empty())
            .and_then(|v| v.parse::<u64>().ok())
    };
    let version = &text[text.find(|c: char| c.is_ascii_digit())?..];
    let mut parts = version.split('.');
    Some((
        digits(parts.next()?)?,
        digits(parts.next()?)?,
        digits(parts.next()?)?,
    ))
}

/// Строка `df -kP` (кроме шапки) → (источник, ГиБ свободно). `-P` гарантирует
/// колоночный порядок Filesystem, 1024-blocks, Used, Available, Capacity, Mounted on,
/// поэтому Available берём четвёртой колонкой — путь монтирования с пробелом
/// (`/Volumes/Новый том/…`) разбор не сдвигает, а шапка не разбирается как данные.
fn parse_df_row(line: &str) -> Option<(String, f64)> {
    let mut cols = line.split_whitespace();
    let src = cols.next()?.to_string();
    let avail = cols.nth(2)?.parse::<f64>().ok()?;
    Some((src, avail / 1024.0 / 1024.0))
}

/// Монтаж-«тень»: `df` по нему отдаёт место тома Docker Desktop, а не тома, откуда
/// примонтировано дерево. Замер 22.11.2026: `/src` внешнего тома со 520 ГиБ свободных
/// отдаёт 22 ГиБ и источник `/run/host_mark/Volumes`. Хост изнутри контейнера не виден
/// (ADR-0013), поэтому такое место — не свидетель ни в какую сторону.
fn df_is_host_shadow(src: &str) -> bool {
    src.starts_with("/run/host_mark") || src.starts_with("virtiofs") || src.starts_with("9p")
}

/// Свободное место по пути: (источник из первой колонки `df`, ГиБ).
fn df_free(path: &str) -> Result<(String, f64), String> {
    let out = Command::new("df")
        .args(["-kP", path])
        .output()
        .map_err(|e| format!("df: {e}"))?;
    if !out.status.success() {
        return Err(format!("df {path}: не удалось"));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let row = text
        .lines()
        .nth(1)
        .ok_or_else(|| format!("df {path}: нет строки монтирования"))?;
    parse_df_row(row).ok_or_else(|| format!("df {path}: не разобрать свободное место"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Пин rust-toolchain.toml — минимум, а не точная версия (ТЗ 10.1). Строки
    /// `rustc --version` сняты с образов ворот: Alpine edge даёт 1.100.0.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ТЗ 10.1)"
    )]
    fn rustc_pin_sravnivaet_kak_minimum() {
        let pin = parse_rustc("1.97.1").unwrap();
        assert_eq!(
            parse_rustc("rustc 1.100.0 (0ef1bc386 2026-11-09) (Alpine Linux Rust 1.100.0-r0)"),
            Some((1, 100, 0))
        );
        assert!(parse_rustc("rustc 1.100.0 (0ef1bc386 2026-11-09)").unwrap() > pin);
        assert!(parse_rustc("rustc 1.97.1 (38a349) 2026-07-01").unwrap() >= pin);
        assert!(parse_rustc("rustc 1.96.1 (e0988fd) 2026-02-10").unwrap() < pin);
        assert!(parse_rustc("rustc nightly-2026-01-01").is_none());
    }

    /// Разбор `df -kP` (BusyBox из образа ворот) и «тень» тома Docker Desktop. Строки
    /// сняты 22.11.2026 в `svetlitsa-gate:local` с примонтированным внешним томом:
    /// `/src` со 520 ГиБ свободных отдаёт 22 ГиБ чужого тома.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ТЗ 3.8, Q101)"
    )]
    fn df_ten_toma_dockera_ne_mesto_sborki() {
        let (src, gib) =
            parse_df_row("overlay              1055761844   5747864 996310556   1% /").unwrap();
        assert_eq!(src, "overlay");
        assert!(!df_is_host_shadow(&src));
        assert!((gib - 950.2).abs() < 0.1);

        let (src, gib) =
            parse_df_row("/run/host_mark/Volumes   239362496 215926752  23435744  90% /src")
                .unwrap();
        assert_eq!(src, "/run/host_mark/Volumes");
        assert!(df_is_host_shadow(&src));
        assert!((gib - 22.4).abs() < 0.1);

        // Путь монтирования с пробелом: Available остаётся 4-й колонкой.
        let (src, gib) =
            parse_df_row("/dev/disk3s5 241266520 215926752 545331200 29% /Volumes/Новый том")
                .unwrap();
        assert_eq!(src, "/dev/disk3s5");
        assert!((gib - 520.0).abs() < 0.1);

        // Шапка и мусор — не данные.
        assert!(
            parse_df_row("Filesystem  1024-blocks Used Available Capacity Mounted on").is_none()
        );
        assert!(parse_df_row("df: нет такой строки").is_none());
    }

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

    /// Раздел X-механизмов (ADR-0016): найденные маркеры перечисляются с заменой, а пустое
    /// зеркало честно говорит «не найдено» вместо того, чтобы молчать.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn razdel_x_mehanizmov() {
        let row = MapRow {
            repo: "mate-fake".to_string(),
            kreyt: "fake".to_string(),
            maylston: "М0".to_string(),
            level: Level::Modules,
        };
        let found = x_mechanisms_of(&fixture());
        assert_eq!(found.get("XOpenDisplay"), Some(&1));
        let text = render_generated(&row, "deadbee", &modules_of(&fixture()), &found, &[], false);
        assert!(text.contains("## X-механизмы оригинала"), "{text}");
        assert!(
            text.contains("`XOpenDisplay`"),
            "маркер обязан попасть в лист"
        );
        assert!(
            text.contains("wl_display"),
            "рядом обязана быть Wayland-замена"
        );

        let empty = x_mechanisms_of(&p("xtask/tests/fixtures/mate-empty"));
        let text = render_generated(
            &row,
            "deadbee",
            &modules_of(&p("xtask/tests/fixtures/mate-empty")),
            &empty,
            &[],
            false,
        );
        assert!(text.contains("Ни один из маркеров не найден"), "{text}");
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
        let fresh = render_generated(
            &row,
            "deadbee",
            &modules_of(&fixture()),
            &x_mechanisms_of(&fixture()),
            &[],
            false,
        );
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
            &x_mechanisms_of(&fixture()),
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
        let without = render_generated(
            &plain,
            "deadbee",
            &modules,
            &x_mechanisms_of(&fixture()),
            &[],
            false,
        );
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

    /// Негатив (Q53): шаг `deps` красен на X11-пакете и на двух звуковых слоях, зелёный — на
    /// одном слое без X11. Правило судится напрямую, без настоящего дерева.
    #[test]
    fn deps_krasny_na_x11_i_na_dva_zvukovyh_sloya() {
        let mut deps: BTreeSet<String> = ["efl", "zbus", "pipewire"]
            .into_iter()
            .map(String::from)
            .collect();
        assert!(
            dependency_problems(&deps).is_empty(),
            "чистый набор покраснел: {deps:?}"
        );
        deps.insert("x11rb".to_string());
        assert!(
            dependency_problems(&deps)
                .iter()
                .any(|s| s.contains("x11rb") && s.contains("ADR-0016")),
            "x11rb проскочил шаг deps"
        );
        deps.remove("x11rb");
        deps.insert("libpulse-binding".to_string());
        assert!(
            dependency_problems(&deps)
                .iter()
                .any(|s| s.contains("один слой") && s.contains("ADR-0022")),
            "два звуковых слоя проскочили шаг deps"
        );
        // Имя пакета из комментария или из соседней таблицы выдумано быть не должно.
        assert!(
            manifest_dep_names(
                "[package]\nname = \"xtask\"\n# x11rb = \"0.14\"\n[dependencies]\nzbus = \"5\"\n"
            ) == ["zbus"]
                .into_iter()
                .map(String::from)
                .collect::<BTreeSet<String>>(),
            "разбор манифеста читает комментарии или соседние таблицы"
        );
    }

    /// Строка карты «Вне MATE» (ТЗ 2.3): зеркала нет по замыслу, но лист обязан оставаться
    /// непротиворечивым — без полей «Причина:»/«Переносится:» он красен, как и любой нулевой.
    #[test]
    #[allow(
        clippy::unwrap_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn vne_mate_list_bez_zerkala_no_bez_poley_krasen() {
        let row = MapRow {
            repo: "—".to_string(),
            kreyt: "pamyatka".to_string(),
            maylston: "М3".to_string(),
            level: Level::OwnApi,
        };
        assert!(
            !has_mirror(&row),
            "строка «—» обязана распознаваться беззеркальной"
        );
        let generated = render_generated(&row, "—", &BTreeMap::new(), &BTreeMap::new(), &[], false);
        assert!(generated.contains("Зеркало: **нет**"), "{generated}");
        assert!(
            generated.contains("проверять нечего"),
            "X-механизмы обязаны честно говорить, что проверять нечего"
        );
        let bare = splice("# Титл", &generated, None, HANDWRITTEN_STUB).unwrap();
        let (_, tail) = bare.split_once(GEN_END).unwrap();
        let problems = sheet_problems(&row, "", &generated, tail, &proven_is_ok).0;
        assert!(
            problems.iter().any(|s| s.contains("нулевой лист"))
                && problems
                    .iter()
                    .any(|s| s.contains("поверхность ведётся руками")),
            "правила рукописного блока не сработали: {problems:?}"
        );
        // Пустой head не должен порождать ложный «лист устарел».
        assert!(
            !problems.iter().any(|s| s.contains("устарел")),
            "строка без зеркала не может устареть из-за отпечатка: {problems:?}"
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
            &x_mechanisms_of(&p("xtask/tests/fixtures/mate-empty")),
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
            sheet_problems(&row, "deadbee", &generated, tail, &proven_is_ok)
                .0
                .iter()
                .any(|s| s.contains("нулевой лист")),
            "правило нулевого листа не сработало"
        );
        let explained =
            format!("{bare}\nПричина: обёртка сборки, кода нет.\nПереносится: оверрайд — М5.\n");
        let (_, tail) = explained.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", &generated, tail, &proven_is_ok)
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
        let generated = render_generated(
            &row,
            "deadbee",
            &modules_of(&fixture()),
            &x_mechanisms_of(&fixture()),
            &[],
            false,
        );
        let bare = splice("# Титл", &generated, None, HANDWRITTEN_STUB).unwrap();
        let (g, tail) = bare.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail, &proven_is_ok)
                .0
                .iter()
                .any(|s| s.contains("ведётся руками")),
            "лист «руками» без причины обязан краснеть"
        );
        let explained = format!("{bare}\nПричина: заголовков нет.\n");
        let (g, tail) = explained.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail, &proven_is_ok)
                .0
                .is_empty(),
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
        let generated = render_generated(
            &row,
            "deadbee",
            &modules_of(&fixture()),
            &x_mechanisms_of(&fixture()),
            &[],
            false,
        );
        let sheet = splice("# Титл", &generated, None, HANDWRITTEN_STUB).unwrap();
        let closed = format!("{sheet}\nЗакрыт: 2026-11-15, коммит `deadbee`\n");
        let (g, tail) = closed.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail, &proven_is_ok)
                .0
                .iter()
                .any(|s| s.contains("лист закрыт")),
            "храповик закрытия не сработал"
        );
        let note = "artifacts/smoke-m0-hello.txt";
        let full = format!(
            "{closed}| `panel.c` | модуль | готово | {note} |\n\
             | `window.c` | модуль | готово | {note} |\n\
             | `util.c` | модуль | готово | {note} |\n"
        );
        let (g, tail) = full.split_once(GEN_END).unwrap();
        assert!(
            sheet_problems(&row, "deadbee", g, tail, &proven_is_ok)
                .0
                .is_empty(),
            "закрытие при полном покрытии обязано быть зелёным"
        );
        // Мутация из тикета 03: «файла прогона нет → `готово` краснеет».
        assert!(
            sheet_problems(&row, "deadbee", g, tail, &proven_is_missing)
                .0
                .iter()
                .any(|s| s.contains("нет отпечатка прогона") && s.contains("готово")),
            "«готово» без отпечатка прогона обязано краснеть (ТЗ 6.9, Q83)"
        );
        assert!(
            sheet_problems(&row, "stale1", g, tail, &proven_is_ok)
                .0
                .iter()
                .any(|s| s.contains("лист устарел")),
            "уехавший отпечаток обязан краснеть"
        );
    }

    /// Отпечаток считается доказанным — для тестов, которые судят не его.
    fn proven_is_ok(_: &str) -> Option<String> {
        None
    }

    /// Отпечатка нет на диске — так его судит тест мутации.
    fn proven_is_missing(path: &str) -> Option<String> {
        Some(format!("нет отпечатка прогона {path}"))
    }

    /// Право строки на `готово` читается, а не зашито: нет пути в пометке — нет и права;
    /// есть путь, но файла нет — краснеет; `в работе` не судится вовсе (тикет 03, Q83).
    #[test]
    fn право_на_готово_читается_из_отпечатка() {
        let no_note = "| `window.c` | модуль | готово | |\n";
        let with_note = "| `window.c` | модуль | готово | artifacts/smoke-m0-hello.txt |\n";
        let not_ready = "| `window.c` | модуль | в работе | |\n";
        assert_eq!(
            gotovo_without_run_problems("svetlitsa-ui", no_note, &proven_is_ok).len(),
            1,
            "«готово» без пути к прогону в пометке — права нет"
        );
        assert!(gotovo_without_run_problems("svetlitsa-ui", with_note, &proven_is_ok).is_empty());
        assert_eq!(
            gotovo_without_run_problems("svetlitsa-ui", with_note, &proven_is_missing).len(),
            1,
            "отпечатка нет на диске → «готово» краснеет"
        );
        assert!(
            gotovo_without_run_problems("svetlitsa-ui", not_ready, &proven_is_missing).is_empty(),
            "«в работе» судить нечем"
        );
    }

    /// Отпечаток разбирается из файла: код прогона и снимок. Мутация «вернуть литерал»
    /// этот тест краснит — у функции нет ответа без входных данных (Q83).
    #[test]
    fn отпечаток_smoke_разбирается_из_файла() {
        let empty = SmokeRun::default();
        assert_eq!(empty.exit, None, "пустой отпечаток не утверждает про успех");
        assert_eq!(empty.screenshot, None);
        let no_frame = smoke_run_parse("выход: 0\nснимок: \n");
        assert_eq!(no_frame.exit, Some(0));
        assert_eq!(
            no_frame.screenshot, None,
            "пустое значение — «снимка не было»"
        );
        let good = smoke_run_parse("выход: 0\nснимок: artifacts/smoke/frame.png\n");
        assert_eq!(
            good.screenshot.as_deref(),
            Some("artifacts/smoke/frame.png")
        );
        // Красный прогон не зеленеет от соседнего снимка.
        assert_ne!(smoke_run_parse("выход: 1\nснимок: a.png\n"), good);
        // Ни файла, ни права: путь в неexistent артефакте краснеет по-настоящему.
        assert!(smoke_run_problem(&p("artifacts/smoke-такого-нет.txt")).is_some());
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
        })
        .expect_err("одна пропущенная строка карты — это неполная сверка");
        assert!(
            err.contains("marco"),
            "пропуск обязан быть назван именем: {err}"
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
                without_mirror: &[]
            })
            .is_ok(),
            "все строки карты с зеркалом — шаг зелёный"
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
}
