# Чек-лист требований среды ворот

Декларация того, какой должна быть среда, по которой потом пишутся `scripts/build-in-docker.sh` и
пункты `xtask ci --doctor`. Среда — **только Wayland** (ADR-0016) и **только контейнер на Alpine
Linux: musl, x86_64, openrc/BusyBox** (ADR-0023). X11-инструменты (`Xvfb`, `xdotool`, `xwd`, `import`)
из чек-листа убраны; glibc-хост, на котором всё «соберётся руками», доказательством не является.
Скрипты среды вслепую не пишем — непрогнанный скрипт это «почти зелёное» (принцип ADR-0006, ТЗ 3.8).
Здесь только то, что можно проверить прогоном на реальной среде.

Статус «проверено» относится к среде, в которой запускался `xtask ci --doctor`. Контейнер на Alpine
ещё не собран, поэтому все строки про musl — «не проверялось», а не «есть».

## Обязательное (падает = итерация не начинается)

| Требование | Как проверить | Статус на 21.11.2026 |
|---|---|---|
| Docker на хосте ворот доступен и работает | `docker info` | **есть** на `192.168.0.80` (docker 28.0.4); на `192.168.0.86` не проверялось — хост по Q82 не назван |
| Базовый образ Alpine зафиксирован digest'ом, а не тегом `latest` | `docker inspect --format '{{index .RepoDigests 0}}' <образ>` | **нет** — `Dockerfile` не написан (задача 2.1) |
| rustc **не ниже** пина `rust-toolchain.toml` (1.97.1), тройка `x86_64-alpine-linux-musl` | `docker run … rustc --version -Vl` | не проверялось; в Alpine edge — `rust` 1.100.0 |
| `cargo`, `rustfmt`, `rust-clippy`, `cargo-deny`, `sccache` — пакетами Alpine | `apk info -e cargo rustfmt rust-clippy cargo-deny sccache` | не проверялось; версии в индексе edge: 1.100.0 / 1.100.0 / 1.100.0 / 0.20.2 / 0.17.0 |
| `git`, `curl`, `jq`, `meson`, `ninja-build`, `pkgconf`, `musl-dev`, `clang16` | `command -v` внутри контейнера | не проверялось (все есть в индексе edge) |
| ≥ 60 ГБ свободно под `target/`, зеркала и кэш sccache | `df -kP` на хосте | **нет**: на `192.168.0.80` `/var/lib/docker` занят на 91% (19 ГиБ свободно) |
| Ограничение памяти сборки — лимитом контейнера, не systemd | `docker run --memory=12g …` | заменяет строку про `systemd-slice`/`MemoryMax` (ADR-0023) |
| `cargo-deny` (шаг `deny` обязан исполняться, а не прятать нечитаемый конфиг, Q81) | `cargo deny --version` + `cargo deny check` | была версия 0.21.0 на macOS-среде (2026-11-18); в контейнере не проверялось |
| Подпись канона **проверяется** (Q58) | `cargo run -p xtask -- canon` → `теги совпадают` | есть (проверено 2026-11-15 в клоне); настроек клона не требуется — `canon/allowed_signers` и `identity` лежат в каноне |
| Тег **подписать** чем | `git config user.signingkey` + `gpg.format=ssh` | настроено в этом клоне; ключ подписи отдельный от ключа пуша и в клон не кладётся |

## После первого зелёного doctor (ADR-0013 в редакции ADR-0023)

| Требование | Как проверить |
|---|---|
| Образ среды снят, digest/дата в `docs/journal.md` («Образы») | digest образа в журнале |
| Manifest: digest базового Alpine, sha256 тарбóлов EFL/Elementary, версии Alpine-пакетов тулчейна, путь к `/opt/svetlitsa-efl`, отпечатки зеркал | `sha256sum`, `rustc --version`, `apk info` |
| Doctor при каждом подъёме из образа | `xtask ci --doctor` |
| EFL 1.29.0 + Elementary собраны **под musl** нашим скриптом, с тестами и NLS | `scripts/efl-build.sh`, `pkg-config --modversion elementary` |
| Фикстура «откуда мигрируем» — гость с Ubuntu MATE 24.04 LTS (ТЗ 3.4) | `cat /etc/os-release`, `dpkg -l \| grep mate-` |

Последняя строка — единственное место, где чужая glibc-система нужна намеренно: это образец
мигрирующего пользователя, а не среда сборки.

## Нужно для UI-прогонов (подсказка doctor'а, красит не doctor, а smoke)

| Требование | Как проверить | Статус |
|---|---|---|
| EFL 1.29.0 + Elementary в `/opt/svetlitsa-efl` (musl), пин + sha256 | `pkg-config --modversion elementary` | нет (сборки под musl ещё нет — риск №1, задача 2.2) |
| Wayland-композитор для прогонов (`cage` 0.3.1 или `weston --headless` 16.0.0) и `WAYLAND_DISPLAY` | `command -v cage weston; echo $WAYLAND_DISPLAY` | нет (в контейнере ещё не запускалось) |
| `wayland-protocols` + `wayland-scanner` (генерация `xdg-shell`, `wlr-layer-shell`, `wlr-data-control`) | `pkg-config --modversion wayland-protocols; command -v wayland-scanner` | нет (в индексе edge: `wayland-protocols` 1.49) |
| Снимок экрана — наш путь захвата (протокол сеанса), а не `xwd`/`import` (ADR-0016) | smoke-сценарий `xtask smoke` | нет (smoke без композитора не запускается) |
| Шрифты PT Sans/PT Serif (ТЗ 7.3) — лежат в репозитории, в Alpine их нет | `fc-match` | не проверялось |
| Скрипты репозитория запускаются на BusyBox ash | `busybox sh -n scripts/*.sh` | нет (задача 4.5) |

## Доступно извне

| Требование | Зачем |
|---|---|
| GitHub (api + clone) | зеркала `vendor/mate/`, ≈ 1 ГБ (ТЗ 3.8) |
| `origin` | общая точка пуша: `https://github.com/DmitriyRudenskiy/Svetlitsa` (проверено 21.11.2026; голый репозиторий на `192.168.0.86`, названный в первой редакции чек-листа, в клонах не настроен) |
| `dl-cdn.alpinelinux.org` | индекс и пакеты Alpine (проверено 21.11.2026: APKINDEX edge main/community/testing загружены) |

## Критерий готовности среды (Q40)

Не «`df` показывает 60 ГБ», а **зелёный `xtask ci --doctor` внутри контейнера на Alpine**, с
напечатанными digest'ом образа и `rustc --version`. До него М0-спайк (контрольная точка, 4 итерации)
не начинается; статус — «ожидает среду».
