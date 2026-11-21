# Реестр разбора COSMIC DE

Контракт Светлицы — MATE (карта ТЗ 2, `docs/parity/`). COSMIC — **не контракт**, а источник
готовых Rust-решений. Поэтому это не парити-лист: здесь нет позиций «готово/в работе», есть вердикт
о переносимости. Порядок разбора и правила — `openspec/changes/cosmic-de-analysis/design.md`.

Вердикт (закрытый список): `переносим` — берём идею или контракт поведения; `добавляем` — берём
как готовую зависимость; `не переносим` — конфликтует со стеком или ADR; `мимо` — к Светлице не
относится. Уровень: `контракт` / `крейт` / `содержимое` / `—`.

Лицензионный гейт = список `allow` в `deny.toml` (MIT, Apache-2.0, BSD-2/3, ISC, Zlib, **MPL-2.0**,
LGPL-2.0/2.1, GPL-2/3), а не «только разрешительные»: проект уже принимает MPL-2.0 и GPL. Для
вердикта `добавляем` читается `license` из `Cargo.toml` самого крейта, а не из README репозитория.

Зеркала: `idea/<репозиторий>` (`--depth 1 --single-branch --no-tags`), в git не входят. `HEAD` —
голова зеркала на момент вердикта. Размер — `du -sh` зеркала с `.git`.

## Слой 1 — мелкие GUI-free крейты

| Репозиторий | HEAD | Размер | Лицензия | Что это | Уровень | Вердикт | Наш крейт | Пометка |
|---|---|---|---|---|---|---|---|---|
| `pop-os/freedesktop-desktop-entry` | `19765f3` | 452 КиБ | MPL-2.0 | Парсер Desktop Entry Spec: `generic_entry`, `exec.rs` (раскладка `%f/%U`), поиск приложения по appid, `iter` по `$XDG_DATA_DIRS` | крейт | добавляем | `traktir`, `svetlitsa-xdg`, `bufetchik` | 2224 строки Rust, зависимостей 6 (`memchr`, `thiserror`, `xdg`, `bstr`, `unicase`, `gettext-rs`), ни одной GUI. MPL-2.0 уже в `deny.toml`. Покрывает ядро `traktir` (mate-menus) — писать свой парсер `.desktop` смысла нет; наш порт обязан давать то же поведение на тех же файлах, а не тот же код |
| `pop-os/os-release` | `5d42b30` | 236 КиБ | MIT | Парсер `/etc/os-release` (`KEY=VALUE` с кавычками), ленивая статическая `OS_RELEASE` | контракт | переносим | `svetlitsa-platform` | 173 строки, зависимость одна (`lazy_static`). Брать как зависимость незачем — свой парсер на 20 строк дешевле нового узла supply chain; берём набор полей (`ID`, `ID_LIKE`, `VERSION_ID`, `VARIANT`) как контракт для определения гостя ворот |
| `pop-os/xkb-data` | `85bfc76` | 252 КиБ | MPL-2.0 | Читает `/usr/share/X11/xkb` (`evdev.xml`): список раскладок, варианты, `ConfigItem` | крейт | добавляем | `bukvar`, `skhodka` | 226 строк, зависимости `serde`/`serde-xml-rs`. Прямое попадание в `bukvar` (порт libmatekbd, ТЗ 9.3): перечисление раскладок — ровно та поверхность, которую MATE берёт из libxkbcommon-данных, и писать свой XML-парсер под тот же файл смысла нет |
| `pop-os/recently-used-xbel` | `fe32c16` | 308 КиБ | MPL-2.0 | Парсер и очистка `~/.local/share/recently-used.xbel` (XBEL: закладки, mime, приложения, timestamps) | крейт | добавляем | `ambar`, `svetlitsa-vfs` | 494 строки, `quick-xml`/`chrono`/`url`/`mime_guess`. «Недавние документы» в меню амбара (caja) читают этот же файл; формат XBEL в MATE больше никем не парсится, так что это единственный готовый вариант на Rust |
| `pop-os/proc-mounts` | `2eca05a` | 280 КиБ | MIT | Перечисление активных монтирований и свопов из `/proc/mounts`, `/proc/swaps` | контракт | переносим | `ambar`, `svetlitsa-vfs` | 624 строки, одна зависимость (`partition-identity`). Последний коммит 2022-03 (пин Rust 1.59) — как зависимость брать нельзя, заброшен. Идея «отдать список монтирований с типом и опциями» полезна для дерева амбара, но MATE-контракт идёт через GIO/GDaemon + udis2, а не через `/proc` |
| `pop-os/sysfs-class` | `ab63e7f` | 328 КиБ | MIT | Объектный вид `/sys/class`: подсветка, температурки, power_supply, `numtoa` вместо форматников | контракт | переносим | `barometr`, `kerosinka` | 973 строки в 22 файлах. Последний коммит 2021-02. Для `barometr` (mate-sensors-applet) контракт «какие узлы hwmon/power_supply читать» ценен, но MATE опирается на libsensors, поэтому копия кода создала бы второй, расходящийся источник показаний |
| `pop-os/deb-control` | `f92a25a` | 264 КиБ | MIT OR Apache-2.0 | Zero-copy парсер и futures-кодек файлов `debian/control` | — | мимо | — | 307 строк, нужен pop-os для установщика и репозитории. К 28 компонентам рабочего стола отношения не имеет: Светлица пакеты не парсит |

## Слой 2 — D-Bus/zbus-обёртки

Общее по слою, и оно же — причина, по которой ни один из этих крейтов не получает `добавляем`:
актуальный `zbus` на crates.io — **6.5.0** (rust 1.89), а космические обёртки сидят на **3.7/3.8**
(2023 год) или на git-версии. Это три мажора; переносится не код, а контракт — какие методы и
сигналы зовутся и в какие структуры раскладываются ответы. Наш `svetlitsa-dbus` по ТЗ 4.3 и
ADR-0005 — zbus без tokio-цикла в UI-потоке, то есть чужой async-код нам и не нужен.
| Репозиторий | HEAD | Размер | Лицензия | Что это | Уровень | Вердикт | Наш крейт | Пометка |
|---|---|---|---|---|---|---|---|---|

| `pop-os/upower-dbus` | `30e0173` | 260K | MIT | UPower info via zbus | контракт | переносим | `kerosinka` | 169 строк, zbus 3.7, последний релиз 2023-01. Ценна поверхность `org.freedesktop.UPower` (устройства, уровень, статус, сигнал `Changed`) — ровно то, что ТЗ 9.21 требует от керосинки |
| `pop-os/logind-dbus` | `f7178dd` | 232K | MIT | Communicate with the LoginManager DBus API | контракт | переносим | `kerosinka`, `klyuchnik` | 342 строки в одном файле, zbus **из git** (gitlab freedesktop) — прямо против `deny.toml` (`unknown-git = deny`). Как зависимость невозможен; берём список методов `org.freedesktop.login1.Manager` (Sleep/Hibernate/Inhibit) |
| `pop-os/timedate-zbus` | `bf4a841` | 248K | MPL-2.0 | a dbus client (using zbus) for timedate | контракт | переносим | `skhodka` | 105 строк, zbus 3.8. Модуль «дата-время» сходки (ТЗ 9.20); поверхность крошечная — фиксировать имена и сигнатуры, а не тащить обёртку |
| `pop-os/gvfs-zbus` | `b7e3bfb` | 312K | MIT OR Apache-2.0 | DBus interfaces for GVfs; 726 строк Rust в 12 файлах | контракт | переносим | `svetlitsa-vfs` | zbus 3, обёртки GVfs-портов. Наш `svetlitsa-vfs` по ТЗ 3 делает то же через GIO-FFI (сеть, корзина, монтирование) — два источника одной правды вести нельзя, поэтому берём перечень путей/методов, а не обёртку |
| `pop-os/mpris2-zbus` | `a41a9b9` | 344K | MPL-2.0 | zbus-based bindings for MPRIS2 (Media Player Remote Interfacing Specification) on Linux; 1835 строк Rust в 17 файлах | контракт | переносим | `gramofon` | 1635 строк, zbus 3.7, последний коммит 2023-01. MPRIS-2 — то, чем граммофон (ТЗ 9.15) управляет плеерами и получает медиа-клавиши; объёмный контракт (Player/TrackList/Playlists + свойства), поэтому перечень методов стоит перенести, а код — нет |
| `pop-os/packagekit-zbus` | `ad88d9b` | 272K | MIT OR Apache-2.0 | DBus interfaces for PackageKit; 524 строк Rust в 4 файлах | — | мимо | — | 425 строк, zbus 5, свежий (2026-03) — единственный из слоя, кто не отстал на три мажора. Но в карте ТЗ 2 нет компонента управления пакетами: у MATE это отдельный mate-software-update-indicator, в объём Светлицы не входит |
| `pop-os/dbus-udisks2` | `15c4e70` | 296K | MIT | UDisks2 DBus API; 1277 строк Rust в 9 файлах | контракт | переносим | `ambar`, `svetlitsa-vfs` | 1176 строк, MIT, последний коммит 2020-11 — поколение zbus 1.x, как зависимость мёртво. Контракт `org.freedesktop.UDisks2` (Drive/Block/Filesystem/Mount) нужен амбару для раздела «Устройства» и для `pristroyki`-ярколыков дисков |
| `pop-os/dbusmenu-zbus` | `c7153b2` | 276K | MIT OR Apache-2.0 | DBus interfaces for dbusmenu; 120 строк Rust в 3 файлах | контракт | переносим | `lavka`, `pristroyki` | 109 строк, MIT OR Apache-2.0, zbus 4 (2024-11). Протокол DBusMenu (`com.canonical.dbusmenu`) — то, чем лавка показывает чужие меню приложений, а пристройки — контекстные меню амбара от внешних расширений. Кода на 109 строк не хватает на полный протокол (GetLayout/AboutToShow/ItemUpdated), поэтому переносим как контракт, а не как крейт |
| `pop-os/dbus-settings-bindings` | `6ccc9fb` | 1,2M | MPL-2.0 | —; 9359 строк Rust в 105 файлах | крейт | добавляем | `domovoy`, `kerosinka`, `gramofon`, `skhodka` | **Главная находка слоя 2.** MPL-2.0, 8028 строк, 15 отдельных крейтов по одному на интерфейс (`upower`, `login1`, `timedate1`, `mpris2`, `hostname1`, `locale1`, `a11y`, `bluez`, `geoclue2`, `accounts`, `switcheroo-control`, …), zbus **6.0** — то есть актуальное нам поколение, свежий коммит 2026-11. Это официальная замена устаревшим upower-dbus/logind-dbus/timedate-zbus/mpris2-zbus выше: их вердикты «переносим» держатся только потому, что их не обновили, а не потому что брать нечего. Брать поштучно, крейтами; `cosmic-settings-daemon` из них — не наш контракт (у нас `org.svetlitsa.Settings` по ТЗ 9.20) |

## Слой 3 — конфигурация, темы, текст, протоколы

| Репозиторий | HEAD | Размер | Лицензия | Что это | Уровень | Вердикт | Наш крейт | Пометка |
|---|---|---|---|---|---|---|---|---|
| `pop-os/libcosmic` → `cosmic-config` + `cosmic-config-derive` | `be7937d` | 6,9M (весь libcosmic) | MPL-2.0 | Конфигурационный движок: схема конфига через proc-macro-derive, формат RON, запись атомарно, слежение через `notify`, подписка на изменения | контракт | переносим | `svetlitsa-config` | 996 + 224 строки. Зависимости от iced/tokio/smol/zbus **опциональны** (фича `subscription`), то есть ядро GUI-free — но `atomicwrites` при этом не опциональна и берётся **из git** (`github.com/jackpot51/rust-atomicwrites`), что против `deny.toml` (`unknown-git = deny`). Формат тоже чужой: у Светлицы `lavka.toml` и TOML-конвей (ТЗ 3), у них RON. Берём конструкцию «derive-описал поле → получили дефолт, миграцию, watch», а не крейт |
| `pop-os/libcosmic` → `cosmic-theme` | `be7937d` | 6,9M | MPL-2.0 | Модель темы: акцентный цвет → полная палитра (фон/текст/акцент/опасность), экспорт, чтение `configparser`-конфигов тем | контракт | переносим | `svetlitsa-ui`, `izba` | 4414 строки, но `iced_core` — **неопциональная** зависимость: крейт нельзя взять, не затащив iced, а это прямой конфликт с ADR-0003 (EFL, не iced/wgpu). Ценно ровно одно — алгоритм раскладки палитры из одного акцента (зависимости `palette`/`hex_color`, обе под разрешительными лицензиями): у Светлицы тема — Elementary-тема и список иконок по ADR-0012, акцент туда ложится через перегенерацию, и считать её сейчас нечем |
| `pop-os/libcosmic` → `cosmic-launch` | `be7937d` | 6,9M | MPL-2.0 | Запуск `.desktop`: раскладка exec-строки, терминал, `StartupNotify`, смена GPU через switcheroo | контракт | переносим | `svetlitsa-xdg`, `lavka` | 794 строки. Дефолтные фичи `tokio` + `systemd`, а `switcheroo-control-zbus` — git-зависимость, снова против `deny.toml`. Раскладку exec уже покрывает `freedesktop-desktop-entry` (строка 1 реестра), поэтому отсюда берём только перечень: `Terminal=true` → запуск в терминале, `StartupNotify` → курсор ожидания, `prefers-non-blocklisting` |
