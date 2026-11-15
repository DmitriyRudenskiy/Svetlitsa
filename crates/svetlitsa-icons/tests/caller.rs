//! Контракт вызывающего (тикет 04, ADR-0014): журнал и имя-промах живут здесь, а не в крейте.
//! Отдельный тестовый крейт видит только публичный API — ровно как любое приложение М2.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use svetlitsa_icons::{IMAGE_MISSING, Outcome, find};

/// Тема, которую подаёт вызывающий. Счётчик обращений — часть доказательства: он показывает,
/// что повторный запрос имени-промаха делается ровно один и цикла нет.
struct Theme {
    files: BTreeMap<String, PathBuf>,
    asks: Cell<u32>,
}

impl Theme {
    fn new(files: &[&str]) -> Self {
        Self {
            files: files
                .iter()
                .map(|name| {
                    (
                        name.to_string(),
                        PathBuf::from(format!("/theme/{name}.png")),
                    )
                })
                .collect(),
            asks: Cell::new(0),
        }
    }

    fn lookup(&self) -> impl Fn(&str) -> Option<PathBuf> + '_ {
        move |name| {
            self.asks.set(self.asks.get() + 1);
            self.files.get(name).cloned()
        }
    }
}

/// Так выглядит вызывающий код: он же пишет строку в журнал, он же ровно один раз просит
/// имя-промах. У библиотеки ни того, ни другого нет — предикат обязан остаться чистым.
fn request<'a>(name: &'a str, theme: &Theme, log: &mut Vec<String>) -> Outcome<'a> {
    let outcome = find(name, theme.lookup());
    let name = match outcome {
        Outcome::Miss { name } => name,
        other => return other,
    };
    log.push(format!("иконка «{name}» не найдена в теме"));
    let retry = find(IMAGE_MISSING, theme.lookup());
    if matches!(retry, Outcome::Miss { .. }) {
        log.push(format!(
            "имя-промах «{IMAGE_MISSING}» тоже не найдено — третьего запроса не будет"
        ));
    }
    retry
}

#[test]
fn промах_пишется_в_журнал_с_исходным_именем() {
    let theme = Theme::new(&["document-open"]);
    let mut log = Vec::new();
    let outcome = request("document-properties", &theme, &mut log);
    assert_eq!(
        log.first().map(String::as_str),
        Some("иконка «document-properties» не найдена в теме"),
        "в журнале должно быть исходное имя, а не имя-промах"
    );
    // Второй промах — уже про имя-промах: его вызывающий попросил сам, подмены нет.
    assert_eq!(
        outcome,
        Outcome::Miss {
            name: IMAGE_MISSING
        }
    );
    assert_eq!(
        log.len(),
        2,
        "строка о промахе и строка об отсутствии имени-промаха"
    );
    assert_eq!(
        theme.asks.get(),
        2,
        "своё имя и ровно один запрос имени-промаха"
    );
}

#[test]
fn промах_закрывается_именем_промахом_без_второй_попытки() {
    let theme = Theme::new(&["document-open", IMAGE_MISSING]);
    let mut log = Vec::new();
    let outcome = request("document-properties", &theme, &mut log);
    assert!(
        matches!(outcome, Outcome::Hit(_)),
        "тема отдаёт имя-промах — отклик, а не промах: {outcome:?}"
    );
    assert_eq!(
        log.len(),
        1,
        "одна строка о промахе, второй промах не наступил"
    );
    assert_eq!(
        theme.asks.get(),
        2,
        "ровно два обращения к теме: своё имя и имя-промах"
    );
}

/// Прогон четырёх исходов на фикстурной теме (вывод — в журнале раунда). Пустого пути в
/// отклике здесь не встречается ни разу — и не может встретиться: `IconPath` извне не строится.
#[test]
fn четыре_исхода_на_фикстурной_теме() {
    let theme = Theme::new(&["document-open"]);
    for (request, expected) in [
        ("document-open", "отклик"),
        ("document-properties", "промах"),
        ("firefox", "чужое имя"),
        ("edit--copy", "калёное имя"),
    ] {
        let outcome = find(request, theme.lookup());
        let kind = match &outcome {
            Outcome::Hit(_) => "отклик",
            Outcome::Miss { .. } => "промах",
            Outcome::Foreign { .. } => "чужое имя",
            Outcome::Malformed { .. } => "калёное имя",
        };
        if let Outcome::Hit(path) = &outcome {
            assert!(
                !path.as_path().as_os_str().is_empty(),
                "отклик не может нести пустой путь"
            );
        }
        println!("{request} → {kind} ({outcome:?})");
        assert_eq!(kind, expected, "{request} дало не тот исход");
    }
}
