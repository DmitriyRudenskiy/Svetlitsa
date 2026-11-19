# Чек-лист требований среды ворот

Декларация того, какой должна быть среда, по которой потом пишутся `Dockerfile`, `scripts/*` и пункты
`xtask ci --doctor`. Скрипты среды вслепую не пишем — непрогнанный скрипт это «почти зелёное»
(принцип ADR-0006, ТЗ 3.8). Здесь только то, что можно проверить прогоном на реальной среде.

**С 2026-11-20 среда ворот — Docker-образ на Alpine Linux (Q94, [ADR-0015](adr/ADR-0015.md)).**
Имена пакетов ниже — apk; столбец «индекс» означает, что пакет найден в `pkgs.alpinelinux.org`
(edge, x86_64) 2026-11-20, но в собранном образе ещё **не проверялся**: до первого
`docker build` весь список имеет статус «не проверено».

Статус «проверено» относится к среде, в которой запускался `xtask ci --doctor`.

## Обязательное (падает = итерация не начинается)

| Требование | Как проверить | Статус на macOS 2026-11-15 |
|---|---|---|
| rustc = пин из `rust-toolchain.toml` (1.97.1) | `rustc --version` | есть |
| `git`, `rsync` | `command -v` | есть |
| ≥ 60 ГБ свободно под `target/`, зеркала и кэш sccache | `df -kP` | **нет** (15 ГиБ) |
| `cargo-deny` (шаг `deny` обязан исполняться, а не прятать нечитаемый конфиг, Q81) | `cargo deny --version` + `cargo deny check` | есть, 0.21.0 (поставлен 2026-11-18); `advisories ok, bans ok, licenses ok, sources ok` |
| Резерв RAM на сборку: в контейнере — `docker run --memory=12g` (на машине с systemd — slice с `MemoryMax=12G`), чужие тяжёлые процессы остановлены | `docker inspect --format '{{.HostConfig.Memory}}'`, `systemd-cgls` | не применимо на macOS |
| Подпись канона **проверяется** (Q58) | `cargo run -p xtask -- canon` → `теги совпадают` | есть; настроек клона не требуется — `canon/allowed_signers` и `identity` лежат в каноне |
| Тег **подписать** чем | `git config user.signingkey` + `gpg.format=ssh` | настроено в этом клоне; ключ подписи отдельный от ключа пуша и в клон не кладётся |

## После первого зелёного doctor (ADR-0013)

| Требование | Как проверить |
|---|---|
| Golden image со средой снят, id/дата в `docs/journal.md` («Образы») | id образа в журнале |
| Manifest: sha256 тарболов EFL/Elementary, пин тулчейна, состав пакетов, отпечатки зеркал | `sha256sum`, `rustc --version`, `dpkg-query` |
| Doctor при каждом подъёме из образа | `xtask ci --doctor` |
| Машина-источник фикстур — Ubuntu MATE 24.04 LTS (фикстуры миграции, ТЗ 3.4); это **не** среда ворот (Q94) | `cat /etc/os-release`, `dpkg -l \| grep mate-` |

## Нужно для UI-прогонов (подсказка doctor'а, красит не doctor, а smoke)

| Требование | Как проверить | Статус |
|---|---|---|
| EFL 1.29.0 + Elementary в `/opt/svetlitsa-efl`, пин + sha256 | `pkg-config --modversion elementary` | нет |
| `Xvfb` | `command -v Xvfb` | нет |
| `xdotool` (ввод), `xwd`/ImageMagick `import` (снимки) | `command -v` | `import` есть, `xdotool` нет (ADR-0011) |
| Шрифты PT Sans/PT Serif (ТЗ 7.3) | `fc-match` | не проверялось |

## Что требует образ ворот (apk-имена Alpine edge, снято 2026-11-20)

| Назначение | Пакет | Индекс Alpine |
|---|---|---|
| сборка C/EFL: компилятор, meson, ninja, pkg-config | `build-base`, `meson`, `ninja-build`, `pkgconf` | есть (main) |
| зависимости EFL (X11, cairo, fontconfig, fribidi, dbus, gstreamer…) | тянутся как `makedepends` пакета `efl` | есть |
| Xvfb и ввод для smoke (ADR-0011) | `xorg-server`, `xvfb-run`, `xdotool`, `imagemagick` | есть (community) |
| ворота: `cargo deny` / `cargo audit`, rustfmt, clippy, bindgen | `cargo-deny`, `cargo-audit`, `rustfmt`, `rust-clippy`, `rust-bindgen` | есть |
| переводы (`.po`, msgfmt) | `gettext` | есть |

Rust из пакета **не используется**: тулчейн ставит rustup по `rust-toolchain.toml` (ТЗ 5).
`elementary` в репозиториях Alpine нет вообще, `efl` — только 1.28.1 в `testing` (ниже пина
1.29.0): оба берутся из тарбóлов (ADR-0003), из Alpine — только зависимости сборки.

## Доступно извне

| Требование | Зачем |
|---|---|
| GitHub (api + clone) | зеркала `vendor/mate/`, ≈1 ГБ (ТЗ 3.8) |
| `origin` (голый репозиторий) | общая точка пуша; сейчас `ssh://192.168.0.86/home/user/git/svetlitsa.git` |

## Критерий готовности среды (Q40)

Не «`df` показывает 60 ГБ», а **зелёный `xtask ci --doctor` в контейнере образа ворот** (Q94).
До него М0-спайк (контрольная точка, 4 итерации) не начинается; статус — «ожидает среду».
