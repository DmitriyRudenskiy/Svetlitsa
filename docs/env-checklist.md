# Чек-лист требований среды ворот

Декларация того, какой должна быть среда, по которой потом пишутся `Dockerfile`, `scripts/*` и пункты
`xtask ci --doctor`. Скрипты среды вслепую не пишем — непрогнанный скрипт это «почти зелёное»
(принцип ADR-0006, ТЗ 3.8). Здесь только то, что можно проверить прогоном на реальной среде.

**С 2026-11-20 среда ворот — Docker-образ на Alpine Linux (Q94, [ADR-0015](adr/ADR-0015.md)).**
Имена пакетов ниже — apk, сверены 2026-11-20 не по сайту, а по `APKINDEX` стабильной ветки
`latest-stable` (**3.24**, `alpine-release 3.24.4-r0`) для aarch64 и x86_64; там же проверено,
чего в стабильной ветке нет вовсе (`efl`, `elementary`). Всё это — сверка имён, а не сборки:
в собранном образе список ещё **не проверялся**, до первого `docker build` статус «не
проверено». Базу образа брать `alpine:3.24` + digest, не `edge`: в edge `efl 1.28.1-r2`
лежит в `testing`, и соблазн «взять пакет вместо тарбóла» означает переезд на rolling.

Статус «проверено» относится к среде, в которой запускался `xtask ci --doctor`.

## Обязательное (падает = итерация не начинается)

| Требование | Как проверить | Статус на macOS 2026-11-15 |
|---|---|---|
| rustc = пин из `rust-toolchain.toml` (1.97.1) | `rustc --version` | есть |
| `git`, `rsync` | `command -v` | есть |
| ≥ 60 ГБ свободно под `target/`, зеркала и кэш sccache | `df -kP` | **нет** (4.1 ГиБ, том заполнен на 98%; именно нехватка места убила ВМ Docker 19.11.2026 — тикет 00) |
| `cargo-deny` (шаг `deny` обязан исполняться, а не прятать нечитаемый конфиг, Q81) | `cargo deny --version` + `cargo deny check` | есть, 0.21.0 (поставлен 2026-11-18); `advisories ok, bans ok, licenses ok, sources ok` |
| Резерв RAM на сборку: в контейнере — `docker run --memory=12g` (на машине с systemd — slice с `MemoryMax=12G`), чужие тяжёлые процессы остановлены | `docker inspect --format '{{.HostConfig.Memory}}'`, `systemd-cgls` | не применимо на macOS |
| Подпись канона **проверяется** (Q58) | `cargo run -p xtask -- canon` → `теги совпадают` | есть; настроек клона не требуется — `canon/allowed_signers` и `identity` лежат в каноне |
| Тег **подписать** чем | `git config user.signingkey` + `gpg.format=ssh` | настроено в этом клоне; ключ подписи отдельный от ключа пуша и в клон не кладётся |

## После первого зелёного doctor (ADR-0013)

| Требование | Как проверить |
|---|---|
| Golden image со средой снят, id/дата в `docs/journal.md` («Образы») | id образа в журнале |
| Manifest: sha256 тарбóла EFL 1.29.0 (Elementary входит в него), пин тулчейна, состав пакетов, отпечатки зеркал | `sha256sum`, `rustc --version` |
| Doctor при каждом подъёме из образа | `xtask ci --doctor` |
| Машина-источник фикстур — Ubuntu MATE 24.04 LTS (фикстуры миграции, ТЗ 3.4); это **не** среда ворот (Q94) | `cat /etc/os-release`, `dpkg -l \| grep mate-` |

## Нужно для UI-прогонов (подсказка doctor'а, красит не doctor, а smoke)

| Требование | Как проверить | Статус |
|---|---|---|
| EFL 1.29.0 с Elementary в `/opt/svetlitsa-efl`, пин + sha256 (один тарбóл, см. ADR-0003) | `pkg-config --modversion elementary` | нет |
| `Xvfb` | `command -v Xvfb` | нет |
| `xdotool` (ввод), `xwd`/ImageMagick `import` (снимки) | `command -v` | `import` есть, `xdotool` нет (ADR-0011) |
| Шрифты PT Sans/PT Serif (ТЗ 7.3) | `fc-match` | не проверялось |

## Что требует образ ворот (apk-имена, APKINDEX Alpine 3.24, снято 2026-11-20)

| Назначение | Пакет | Индекс Alpine |
|---|---|---|
| сборка C/EFL: компилятор, meson, ninja, pkg-config | `build-base`, `meson`, `ninja-build`, `pkgconf` | есть (main) |
| зависимости EFL (X11, fontconfig, fribidi, harfbuzz, gstreamer, lua5.2, mesa…) | список `makedepends` = 35 пакетов из `testing/efl/APKBUILD` (aports) | есть |
| Xvfb и ввод для smoke (ADR-0011) | `xorg-server 21.1.26-r0`, `xvfb-run 1.20.10.3-r2`, `xdotool 4.20260303.1-r0`, `imagemagick 7.1.2.35-r0` | собрано в `svetlitsa-gate:local`; `/usr/bin/import` проверен `import --version` (21.11) |
| ворота: `cargo deny` / `cargo audit`, rustfmt, clippy, bindgen | `cargo-deny 0.18.6-r1` — из apk; `rustfmt`/`clippy` — компоненты rustup из `rust-toolchain.toml`; `cargo-audit`, `rust-bindgen` в образ пока **не** добавлены | `cargo-deny` в образе (21.11); `cargo-audit` и bindgen — в тот день, когда появится потребитель (образ пересобирается за минуты) |
| переводы (`.po`, msgfmt) | `gettext` | есть |

Rust из пакета **не используется** — и это больше не вопрос вкуса: пакетный `rust` в 3.24
это **1.96.1, ниже пина 1.97.1**, так что работает только rustup по `rust-toolchain.toml` (ТЗ 5).
Про EFL и Elementary: в стабильной ветке Alpine их **нет ни в какой редакции**, в `edge/testing`
есть `efl 1.28.1-r2`, а `efl-dev` оттуда же предоставляет `pc:elementary=1.28.1` — то есть
Elementary поставляется внутри EFL, отдельного тарбóла Elementary не существует (последний
отдельный релиз — 1.17.1, 2017 год). Из Alpine берутся только зависимости сборки (ADR-0003).

## Доступно извне

| Требование | Зачем |
|---|---|
| GitHub (api + clone) | зеркала `vendor/mate/`, ≈1 ГБ (ТЗ 3.8) |
| `origin` (голый репозиторий) | общая точка пуша; сейчас `ssh://192.168.0.86/home/user/git/svetlitsa.git` |

## Критерий готовности среды (Q40)

Не «`df` показывает 60 ГБ», а **зелёный `xtask ci --doctor` в контейнере образа ворот** (Q94).
До него М0-спайк (контрольная точка, 4 итерации) не начинается; статус — «ожидает среду».
