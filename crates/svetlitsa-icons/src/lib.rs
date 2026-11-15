//! Слой имён иконок (ADR-0014): закрытый список, форма имени, исходы отыскания.
//!
//! Пиксельный поиск по `$XDG_DATA_DIRS`, наследование hicolor, выбор размера и кэш остаются за
//! Efreet / `svetlitsa-xdg` (ТЗ 3.4, 4.4). Здесь только имена — и потому крейт обязан собираться
//! без заголовков EFL: слой имён живёт **до** EFL, а не поверх него.
//!
//! Лога нет: предикат чистый, «строка в журнал» — контракт вызывающего кода, а не библиотеки.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Имя-промах, предписанное freedesktop-спекой. Его запрашивает **вызывающий**, закрывая
/// промах; предикат его не подставляет — иначе промах стал бы неотличим от успеха и логировать
/// стало бы нечего. Повторный запрос делается ровно один: нет его в теме — это второй промах.
pub const IMAGE_MISSING: &str = "image-missing";

/// Границы генерируемого блока парити-листа. Пишет их `xtask`, читает этот крейт: определение
/// одно, иначе «лист разошёлся с парсером» стало бы возможным состоянием.
pub const GEN_BEGIN: &str = "<!-- BEGIN GENERATOR (xtask parity) — руками не править -->";
pub const GEN_END: &str = "<!-- END GENERATOR -->";

/// Парити-лист `svetlitsa-ui` — единственная копия закрытого списка (ADR-0014). Третьей копии
/// (генерируемый Rust-файл, перенос руками) нет, поэтому расхождения «крейт говорит одно,
/// лист — другое» не существует по построению, а не сверяются.
const SHEET: &str = include_str!("../../../docs/parity/svetlitsa-ui.md");

/// Почему закрытый список не собрался.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListError {
    /// Генерируемого блока нет — список не опубликован.
    NoBlock,
    /// Блок открыт, но не закрыт: правка человека поверх генератора.
    Unclosed,
    /// Имя повторилось. Схлопать его в множество — значит показать зелёным список, которого
    /// никто не собирал: повтор обязан быть красным, а не молчаливым.
    Duplicate { name: String },
    /// Имя не проходит форму (калёное имя в списке). Пункт блока без обратных кавычек — тоже
    /// имя: кавычки это формат генератора, а не часть имени.
    IllFormed { name: String },
}

/// Форма имени иконки: `^[a-z0-9]+(-[a-z0-9]+)*$` — kebab, строгий нижний регистр, без пустых
/// сегментов, без подчёркиваний, нижней границы длины нет. Требовать дефис — значит отказать
/// `home`, а оно легально.
///
/// Нижняя граница длины живёт в другом правиле и в другом месте — в эвристике `xtask`,
/// выуживающей токены из текста C-кода.
pub fn well_formed(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

/// Разобрать закрытый список из markdown-листа. Проза блока (строка счётчика имён) не является
/// пунктом `- `имя`` и пропускается намеренно: генератор пишет её сам.
pub fn parse_closed_list(sheet: &str) -> Result<BTreeSet<String>, ListError> {
    let Some(rest) = sheet.split_once(GEN_BEGIN).map(|(_, rest)| rest) else {
        return Err(ListError::NoBlock);
    };
    let Some(block) = rest.split_once(GEN_END).map(|(block, _)| block) else {
        return Err(ListError::Unclosed);
    };
    let mut names = BTreeSet::new();
    for line in block.lines() {
        let Some(raw) = line.trim().strip_prefix("- ") else {
            continue;
        };
        let name = raw
            .strip_prefix('`')
            .and_then(|s| s.strip_suffix('`'))
            .unwrap_or(raw);
        if !well_formed(name) {
            return Err(ListError::IllFormed {
                name: name.to_string(),
            });
        }
        if !names.insert(name.to_string()) {
            return Err(ListError::Duplicate {
                name: name.to_string(),
            });
        }
    }
    Ok(names)
}

/// Закрытый список, вшитый в бинарник. Разбирается один раз на процесс (лента обращений к
/// иконкам идёт по каждому виджету), поэтому список — `&'static`.
///
/// Пустой список при неразбираемом листе — это громко на разработке (шаг `icons --check`
/// краснеет до коммита) и безопасно в проде: всякое имя становится «чужим именем»,
/// а не паникой (ТЗ 6.4).
pub fn closed_list() -> &'static BTreeSet<String> {
    static LIST: OnceLock<BTreeSet<String>> = OnceLock::new();
    LIST.get_or_init(|| parse_closed_list(SHEET).unwrap_or_default())
}

/// Путь к файлу, отданный темой. Непустота — **свойство типа**: конструктор приватен,
/// единственное место, где значение рождается — [`find`], и он проверяет. За пределами крейта
/// пустой отклик подделать нельзя, поэтому вызывающему не нужно проверять путь каждый раз.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconPath(PathBuf);

impl IconPath {
    fn from_theme(path: PathBuf) -> Option<Self> {
        (!path.as_os_str().is_empty()).then_some(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Исход отыскания имени — четыре названных значения глоссария. Не путать с **негативами
/// закрытого списка** (лишнее, отсутствующее, повтор, калёное имя) — то про ворота, это про API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<'a> {
    /// **Отклик** — имя из списка, файл в теме. Несёт непустой путь (свойство [`IconPath`]).
    Hit(IconPath),
    /// **Промах** — имя наше и в форме, но тема его не отдаёт. Несёт **искомое** имя:
    /// подмена именем-промахом выдала бы отсутствие за успех и оставила бы лог без предмета.
    Miss { name: &'a str },
    /// **Чужое имя** — формы правильной, но в закрытом списке её нет: приложение не имеет права
    /// такую просить, и в тему мы с ней не идём.
    Foreign { name: &'a str },
    /// **Калёное имя** — не проходит форму. Список о таком не спрашивает, тема тоже.
    Malformed { name: &'a str },
}

/// Имя × доступная тема → один из четырёх исходов. Функция чистая: ни I/O, ни лога, ни кэша;
/// тему подаёт вызывающий (у Efreet поиск и так выглядит как функция, поэтому темой здесь
/// называется отображение имени на путь, а не коллекция).
///
/// Порядок проверок — форма, список, тема (ADR-0014): калёное имя нельзя спрашивать у списка,
/// чужое нельзя спрашивать у темы. Обратный порядок выдал бы чужое имя как «промах» и тем самым
/// легализовал бы имя вне закрытого списка.
pub fn find<'a>(name: &'a str, theme: impl Fn(&str) -> Option<PathBuf>) -> Outcome<'a> {
    if !well_formed(name) {
        return Outcome::Malformed { name };
    }
    if !closed_list().contains(name) {
        return Outcome::Foreign { name };
    }
    match theme(name).and_then(IconPath::from_theme) {
        Some(path) => Outcome::Hit(path),
        // Пустой путь тема отдать не вправе: это отсутствие файла, а не находка.
        None => Outcome::Miss { name },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet_with(items: &str) -> String {
        format!("# Лист\n{GEN_BEGIN}\n\nИконок: N\n\n{items}\n{GEN_END}\n")
    }

    /// Предикат формы, а не правило ворот: он остаётся зелёным и при выключенном правиле, и
    /// потому проверкой правила не считается (ТЗ 6.3).
    #[test]
    fn форма_имени_кебаб_без_дефиса_легальна() {
        for good in [
            "home",
            "edit-cut",
            "user-desktop-wallpaper",
            "audio-x-generic",
            "a1",
        ] {
            assert!(well_formed(good), "{good} легально");
        }
        for bad in [
            "",
            "Edit-Copy",
            "edit_copy",
            "edit--copy",
            "-edit",
            "edit-",
            "edit copy",
            "edit.copy",
            "edit/0",
            "ёлок",
        ] {
            assert!(!well_formed(bad), "{bad} не проходит форму");
        }
    }

    /// Повтор имени — красный исход разбора, а не схлопывание в множество: молчаливый дубль
    /// показал бы зелёным список, которого никто не собирал.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn повтор_имени_красит_разбор_списка() {
        let err = parse_closed_list(&sheet_with("- `edit-cut`\n- `edit-copy`\n- `edit-cut`\n"))
            .expect_err("имя повторилось");
        assert_eq!(
            err,
            ListError::Duplicate {
                name: "edit-cut".to_string()
            }
        );
    }

    /// Калёное имя в списке — красный разбор: список не имеет права легализовать имя, которое
    /// приложение потом не примет.
    #[test]
    #[allow(
        clippy::expect_used,
        reason = "в тесте паника и есть результат (ADR-0010)"
    )]
    fn калёное_имя_в_списке_красит_разбор() {
        let err = parse_closed_list(&sheet_with("- `edit--copy`\n")).expect_err("имя не проходит");
        assert_eq!(
            err,
            ListError::IllFormed {
                name: "edit--copy".to_string()
            }
        );
        // Проза блока — не имя, а пропускать её молча нельзя только когда она похожа на имя:
        // русское слово пунктом не является, а `edit--copy` является и красит разбор.
        assert_eq!(
            parse_closed_list(&sheet_with("- палитра «Гжель»\n")).expect_err("не имя"),
            ListError::IllFormed {
                name: "палитра «Гжель»".to_string()
            }
        );
    }

    /// Разбираемый лист обязан быть непустым: пустой список — это «всякое имя чужое», и если бы
    /// границы блока изменились, крейт молча отказался бы от всего списка.
    #[test]
    fn вшитый_лист_разбирается_и_не_пуст() {
        let names = closed_list();
        assert!(
            !names.is_empty(),
            "вшитый лист не разобрался — закрытый список пуст, всякое имя станет чужим"
        );
        assert!(
            names.contains("image-missing"),
            "имя-промах обязано быть в списке"
        );
    }

    /// Два исхода сломанного листа различимы: нет блока и блок без конца. Библиотека не падает
    /// ни там ни там (ТЗ 6.4) — `closed_list` остаётся пустым множеством.
    #[test]
    fn лист_без_границ_не_роняет_библиотеку() {
        assert_eq!(
            parse_closed_list("# Лист\nручной текст\n"),
            Err(ListError::NoBlock)
        );
        assert_eq!(
            parse_closed_list(&format!("{GEN_BEGIN}\n- `edit-cut`\n")),
            Err(ListError::Unclosed)
        );
    }

    #[test]
    fn имя_промаха_доступно_вызывающему_и_лежит_в_списке() {
        assert!(
            closed_list().contains(IMAGE_MISSING),
            "имя-промах обязано входить в закрытый список, иначе вызывающему нечем закрывать промах"
        );
        assert!(well_formed(IMAGE_MISSING));
    }

    use std::cell::Cell;
    use std::path::PathBuf;

    /// Фикстурная тема: что есть на «диске». `None` — файла нет; счётчик обращений доказывает,
    /// что чужое и калёное имя до темы вообще не доходят.
    fn fixture_theme(asks: &Cell<u32>) -> impl Fn(&str) -> Option<PathBuf> {
        move |name| {
            asks.set(asks.get() + 1);
            match name {
                "document-open" => Some(PathBuf::from("/theme/document-open.png")),
                "" => Some(PathBuf::new()),
                _ => None,
            }
        }
    }

    #[test]
    fn отклик_несёт_путь_из_темы_а_не_что_то_другое() {
        let asks = Cell::new(0);
        let outcome = find("document-open", fixture_theme(&asks));
        match outcome {
            Outcome::Hit(path) => {
                assert_eq!(path.as_path().to_str(), Some("/theme/document-open.png"))
            }
            other => panic!("ожидался отклик, получено {other:?}"),
        }
        assert_eq!(asks.get(), 1, "тему спросили ровно один раз");
    }

    /// Подмены нет: промах обязан нести искомое имя. Верни он `image-missing`, лог стал бы
    /// писать «не найдено image-missing» вместо настоящего предмета поиска.
    #[test]
    fn промах_несёт_искомое_имя_а_не_имя_промах() {
        let asks = Cell::new(0);
        let outcome = find("document-properties", fixture_theme(&asks));
        assert_eq!(
            outcome,
            Outcome::Miss {
                name: "document-properties"
            }
        );
        if let Outcome::Miss { name } = outcome {
            assert_ne!(name, IMAGE_MISSING, "промах не подменяется именем-промахом");
        }
    }

    /// Пустой путь тема отдать не вправе: это отсутствие файла, а не находка. Проверка
    /// спрятана в приватном конструкторе `IconPath`, поэтому `Hit` с пустым путём не строится.
    #[test]
    fn пустой_путь_это_промах_а_не_отклик() {
        let theme = |_: &str| Some(PathBuf::new());
        assert_eq!(
            find("document-open", theme),
            Outcome::Miss {
                name: "document-open"
            }
        );
    }

    /// Имя вне списка не доходит до темы: «чужое имя» — отказ приложению, а не промах темы.
    #[test]
    fn чужое_имя_не_спрашивают_у_темы() {
        let asks = Cell::new(0);
        assert!(
            !closed_list().contains("firefox"),
            "фикстура потеряла смысл"
        );
        assert_eq!(
            find("firefox", fixture_theme(&asks)),
            Outcome::Foreign { name: "firefox" }
        );
        assert_eq!(asks.get(), 0, "с чужим именем в тему не идут");
    }

    #[test]
    fn калёное_имя_не_спрашивают_ни_у_списка_ни_у_темы() {
        let asks = Cell::new(0);
        assert_eq!(
            find("edit--copy", fixture_theme(&asks)),
            Outcome::Malformed { name: "edit--copy" }
        );
        assert_eq!(asks.get(), 0, "с калёным именем в тему не идут");
    }
}
