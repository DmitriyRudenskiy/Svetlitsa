## 1. Подготовка

- [x] 1.1 Добавить `idea/` в `.gitignore`, создать `docs/analysis/cosmic/`, закоммитить каркас
- [x] 1.2 Проверить свободное место (`df -h`) и зафиксировать в задаче 7.4 решение о полном/урезанном охвате (гейт D7: < 8 ГиБ → только слои 1–3)
- [x] 1.3 Склонировать `fsvm88/cosmic-overlay` в `idea/cosmic-overlay` (`--depth 1 --single-branch --no-tags`)

## 2. Список пакетов для сборки DE

- [x] 2.1 Вычитать `eclass/cosmic-common.eclass` и `eclass/cosmic-de-r2.eclass`: общие BDEPEND/RDEPEND, `RUST_MIN_VER`, `LLVM_COMPAT`, IUSE
- [x] 2.2 Пройти 136 ebuild и собрать точечные зависимости (кто тянет qt6, libheif, appstream, dart-sass, go/buf, gtk-4)
- [x] 2.3 Выписать 30 подмодулей `pop-os/cosmic-epoch` из `.gitmodules` и `justfile` — это ядро DE
- [x] 2.4 Написать `docs/analysis/cosmic/pakety.md`: слои «тулчейн / GUI / мультимедиа / система / точечные», две колонки (Gentoo-атом ↔ Debian/Ubuntu), источник в каждой секции
- [x] 2.5 Закоммитить `pakety.md`

## 3. Очередь, слой 1 — мелкие GUI-free крейты

- [x] 3.1 `pop-os/freedesktop-desktop-entry` → вердикт в реестр
- [x] 3.2 `pop-os/os-release`, `pop-os/xkb-data`, `pop-os/recently-used-xbel` (пачка ≤3) → вердикты
- [x] 3.3 `pop-os/proc-mounts`, `pop-os/sysfs-class`, `pop-os/deb-control` (пачка ≤3) → вердикты
- [x] 3.4 Коммит после каждой строки/пачки; реестр создаётся на первом же шаге слоя

## 4. Очередь, слой 2 — D-Bus/zbus-обёртки

- [x] 4.1 `pop-os/upower-dbus`, `pop-os/logind-dbus`, `pop-os/timedate-zbus` → вердикты
- [ ] 4.2 `pop-os/gvfs-zbus`, `pop-os/mpris2-zbus`, `pop-os/packagekit-zbus` → вердикты
- [ ] 4.3 `pop-os/fwupd-dbus`, `pop-os/dbus-udisks2`, `pop-os/dbus-settings-bindings` → вердикты
- [ ] 4.4 Коммит после каждой пачки

## 5. Очередь, слой 3 — конфигурация, темы, текст, протоколы, загрузчик

- [x] 5.1 `pop-os/libcosmic` (только `cosmic-config/`, `cosmic-theme/`, `cosmic-launch/`) → вердикты по трём крейтам
- [x] 5.2 `pop-os/async-fetcher` → вердикт
- [ ] 5.3 `pop-os/cosmic-text` → вердикт
- [ ] 5.4 `pop-os/cosmic-protocols` → вердикт
- [ ] 5.5 `pop-os/cosmic-icons` и `pop-os/icon-theme` → вердикт с опорой на ADR-0012/0014
- [ ] 5.6 `pop-os/launcher` (pop-launcher) → вердикт
- [ ] 5.7 Коммит после каждого шага

## 6. Очередь, слой 4–6 — приложения, панель, сессия

- [ ] 6.1 `cosmic-files`, `cosmic-edit`, `cosmic-term` → вердикты (ambar, beresta, morzyanka)
- [ ] 6.2 `cosmic-viewer`, `cosmic-store`, `cosmic-player` → вердикты (vernissage; для store/player аналога в карте ТЗ 2 нет — так и писать)
- [ ] 6.3 `cosmic-panel`, `cosmic-applets`, `cosmic-workspaces-epoch` → вердикты (lavka, samodelki, lezanka)
- [ ] 6.4 `cosmic-app-library`, `cosmic-launcher` → вердикты (bufetchik, traktir)
- [ ] 6.5 `cosmic-session`, `cosmic-settings-daemon`, `cosmic-idle` → вердикты (klyuchnik, domovoy, stavni)
- [ ] 6.6 `cosmic-settings`, `cosmic-notifications`, `cosmic-osd`, `cosmic-screenshot` → вердикты (skhodka, vestnik, gramofon-OSD, shkatulka)
- [ ] 6.7 `cosmic-greeter`, `greetd`, `xdg-desktop-portal-cosmic`, `cosmic-randr`, `cosmic-bg`, `cosmic-monitor` → вердикты
- [ ] 6.8 Коммит после каждой строки/пачки
- [ ] 6.9 Проверить наличие/отсутствие космических эквивалентов у `svitok` (atril → cosmic-reader), `schoty` (mate-calc), `uzelok` (engrampa), `vedro` (libmateweather), `bukvar` (libmatekbd), `barometr` (mate-sensors-applet); чего нет — писать `—`

## 7. Очередь, слой 7 — композитор и GUI-фундамент

- [ ] 7.1 `pop-os/cosmic-comp` → вердикт с явной ссылкой на ADR-0007 (композитор пост-1.0)
- [ ] 7.2 `pop-os/libcosmic` (корневой крейт) и `pop-os/iced` → вердикт с явной ссылкой на ADR-0003
- [ ] 7.3 `pop-os/cosmic-epoch` (метарепозиторий: `.gitmodules`, `justfile`, `templates/`) → вердикт

## 8. Сравнение и закрытие

- [ ] 8.1 Собрать `docs/analysis/cosmic/sravnenie.md`: 28 строк по карте ТЗ 2, пустые эквиваленты — `—`
- [ ] 8.2 В `sravnenie.md` — отдельный абзац-вердикт о переносимости стека (ADR-0003, ADR-0007)
- [ ] 8.3 Сверить реестр с требованиями спеки: закрытый список вердиктов, шесть полей строки, `HEAD` у каждой строки
- [ ] 8.4 Запись в `docs/journal.md`: сколько репозиториев пройдено, сколько `переносим`/`добавляем`, что осталось открытым
- [ ] 8.5 Проверить `git log` на отсутствие правок `crates/`, `Cargo.toml`, `Cargo.lock`, `xtask/` и отсутствие `idea/` в индексе
- [ ] 8.6 `openspec validate --change cosmic-de-analysis`
