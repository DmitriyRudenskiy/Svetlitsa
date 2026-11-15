# Светлица

Порт рабочего стола MATE на Rust + EFL с русской системой имён компонентов. Контракт — [`TASK.md`](TASK.md).

## С чего начинать

1. [`TASK.md`](TASK.md) — объём, стек, майлстоуны, критерии готовности.
2. [`GLOSSARY.md`](GLOSSARY.md) — слова, по которым в ТЗ уже было расхождение.
3. [`docs/adr/`](docs/adr/) — 13 решений, объясняющих «почему», и их следствия.
4. [`docs/journal.md`](docs/journal.md) — протокол разбора ТЗ и счётчик итераций.
5. `docs/parity/<крейт>.md` — объём конкретного компонента: 28 листов, фактуру генерирует `xtask parity`, вердикты пишет человек.

## Ворота

```sh
cargo run -p xtask -- ci             # doctor → fmt → clippy → test → deny → lints → licenses → parity
cargo run -p xtask -- ci --doctor    # только окружение
cargo run -p xtask -- parity [крейт] # перегенерировать фактуру парити-листов из зеркал
cargo run -p xtask -- icons          # закрытый список иконок М2 (ADR-0012)
```

**Зелёным считается только прогнанное в среде ворот** — QEMU/KVM VM с Linux, EFL 1.29 и Xvfb
([ТЗ 3.8](TASK.md), [ADR-0003](docs/adr/ADR-0003.md)); сегодня она не поднята, и это статус М0. На macOS `xtask ci --doctor` обязан быть
красным: там нет ни EFL, ни X11, ни места. Удалённого CI нет, workflow-файлов в репозитории тоже
([ADR-0006](docs/adr/ADR-0006.md)).

## Что где лежит

| Путь | Содержимое |
|---|---|
| `crates/<имя>/` | 39 крейтов: 7 инфраструктурных (`svetlitsa-*`) + 28 компонентов + 3 ядра (`*-core`) + `svetlitsa-demo` (тестовый клиент окон, ТЗ 10.2) |
| `xtask/` | ворота: doctor, лицензии, парити-листы, закрытые списки иконок |
| `docs/adr/` | ADR-0001…0013 |
| `docs/licenses.csv` | лицензии по крейтам, наследуются по upstream ([ADR-0002](docs/adr/ADR-0002.md)) |
| `docs/parity/` | парити-листы (28): генерируемая фактура + рукописные вердикты; функции — только для публичного API библиотек |
| `docs/env-checklist.md` | требования к среде ворот; по нему пишутся doctor-пункты и будущие `scripts/dev-vm/*` |
| `scripts/fetch-vendor.sh` | shallow-зеркала C-оригиналов в `vendor/mate/` (в `.gitignore`, 995 МБ) |
| `themes/`, `po/`, `tests/fixtures/mate-dconf/` | тема «Лубок», каталоги переводов, фикстуры дампов MATE |

Карта «русское имя → C-оригинал → майлстоун» — [ТЗ 2](TASK.md); здесь она дублируется в
`crates/*/Cargo.toml` (`description`) и в `docs/licenses.csv`.

## Состояние

Канон пересобран 2026-11-15 из разбора ТЗ (раунды 1–3, 33 решения). Первый код есть: слой
имённых `svetlitsa-icons` и ворота `xtask ci` (раунд 10, `xtask canon` → голова `canon-r10`);
М0 упирается в среду ворот, а не в код.

## Проверка подписи канона

Подписи канон-тегов — SSH, доверенный принципал лежит в репозитории (`canon/allowed_signers`),
чтобы проверка не требовала чужой ключевой инфраструктуры. Ворота `xtask canon` передают этот
файл из проверяемого дерева сами, поэтому зелёные в любом клоне. Голый `git verify-tag`
файл не ищет — ему его надо назвать, иначе он красен по причине конфига, а не подписи:

```bash
git -c gpg.ssh.allowedSignersFile=canon/allowed_signers verify-tag canon-r10
# Good "git" signature for canon@svetlitsa with ED25519 key SHA256:Ow3HCiH90PGDxDLz0IvBYx/Kgx7NLmJNyOBle8HrAEk
```

Содержит ли запись реестра правду, проверяется против тега: `git rev-parse canon-r10^{commit}`
обязан показать `sha` из `canon.toml` (граница Q58 зафиксирована в журнале раунда 10).
