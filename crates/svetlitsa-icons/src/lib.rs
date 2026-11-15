//! Слой имён иконок (ADR-0014): закрытый список, форма имени, исходы отыскания.
//!
//! Пиксельный поиск по `$XDG_DATA_DIRS`, наследование hicolor, выбор размера и кэш остаются за
//! Efreet / `svetlitsa-xdg` (ТЗ 3.4, 4.4). Здесь только имена — и потому крейт обязан собираться
//! без заголовков EFL: слой имён живёт **до** EFL, а не поверх него.
//!
//! Лога нет: предикат чистый, «строка в журнал» — контракт вызывающего кода, а не библиотеки.

use std::collections::BTreeSet;

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

/// Закрытый список, вшитый в бинарник. Пустой список при неразбираемом листе — это громко на
/// разработке (шаг `icons --check` краснеет до коммита) и безопасно в проде: всякое имя
/// становится «чужим именем», а не паникой (ТЗ 6.4).
pub fn closed_list() -> BTreeSet<String> {
    parse_closed_list(SHEET).unwrap_or_default()
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
}
