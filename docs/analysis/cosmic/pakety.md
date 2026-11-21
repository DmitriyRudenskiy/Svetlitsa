# Пакеты для сборки COSMIC DE

Снято с `fsvm88/cosmic-overlay`, HEAD `dc2418e` (2026-11-18), 136 ebuild, зеркало `idea/cosmic-overlay`.

## Оговорка, без неё список читается неверно

**Оверлей ничего не собирает в `.deb`.** Это Portage-оверлей (`EAPI=8`, eclass `cosmic-common` /
`cosmic-de-r2` / `cosmic-live`), он ставит DE через `emerge`. У самого `pop-os/cosmic-epoch`
каталога `dist-deb` в `master` **нет** — он был удалён; сборка upstream — `just` по 30 подмодулям,
артефакты в `${CARGO_TARGET_DIR}/release`, без пакетной упаковки. Поэтому ниже две колонки
пакетных имён: фактическая (Gentoo, откуда снята) и оценочная (Debian/Ubuntu — для среды ворот
Светлицы, Ubuntu MATE 24.04). **Debian-колонка не проверялась**: на этой машине нет `apt`, имена
даны по схеме Debian-нейминга и помечены как предположение там, где имя не очевидно.

Второе: оверлей **отказался от анбандлинга** `iced`/`libcosmic` (README, commit `fec5043`) —
каждый пакет собирается со своим деревом crates. Для нас это тот же класс проблемы, что `vendor/mate/`
и `cargo-deny`, а не аргумент «у них модульная сборка лучше».

## Ядро DE — что реально ставится

`cosmic-base/cosmic-meta` (RDEPEND, ebuild 1.11.0) = 27 обязательных пакетов + 5 опциональных по USE
(`+fonts +gnome-keyring +greeter +monitor store +viewer`) + `~pop-theme-meta-9999`:

`cosmic-applets`, `cosmic-app-library`, `cosmic-bg`, `cosmic-comp`, `cosmic-edit`, `cosmic-files`,
`cosmic-icons`, `cosmic-idle`, `cosmic-initial-setup`, `cosmic-launcher`, `cosmic-notifications`,
`cosmic-osd`, `cosmic-panel`, `cosmic-player`, `cosmic-randr`, `cosmic-screenshot`, `cosmic-session`,
`cosmic-settings`, `cosmic-settings-daemon`, `cosmic-sound-theme`, `cosmic-term`,
`cosmic-workspaces-epoch`, `pop-launcher`, `xdg-desktop-portal-cosmic`;
опционально: `cosmic-greeter`, `cosmic-monitor`, `cosmic-store`, `cosmic-viewer`, шрифты, keyring.

Те же 30 сущностей перечислены как подмодули в `.gitmodules` метарепозитория `cosmic-epoch`
(+ `simple-wrapper`, + `pop-launcher` = репозиторий `pop-os/launcher`, + `cosmic-applibrary` =
путь `cosmic-app-library`). Источник: `cosmic-epoch/.gitmodules`, `cosmic-epoch/justfile`.

## Тулчейн (BDEPEND)

Источник: `eclass/cosmic-common.eclass`, `eclass/cosmic-de-r2.eclass`.

| Gentoo-атом | Debian/Ubuntu (предположение) | Зачем | На этой машине |
|---|---|---|---|
| `${RUST_DEPEND}` ← `RUST_MIN_VER="1.93.0"` | `rustc` ≥ 1.93 | сборка всех 30 пакетов | `dev-lang/rust-bin-1.95.0` ✅ |
| `llvm-core/{llvm,clang}:${LLVM_SLOT}` ← `LLVM_COMPAT=({21..22})` | `llvm-21-dev`, `libclang-21-dev` | bindgen | `llvm-22.1.8`, `clang-22.1.8` ✅ |
| `app-arch/zstd` | `zstd` | распаковка вендорных crates | `zstd-1.5.7-r1` ✅ |
| `>=virtual/pkgconfig-3` | `pkg-config` | pkg-config-шаги | `pkgconfig-3` ✅ |
| `dev-vcs/git` | `git` | live-ebuilds (9999) | `git-2.53.0` ✅ |

**Жёсткий гейт:** `RUST_MIN_VER=1.93` и LLVM 21–22. Наш пин — `rust-toolchain.toml` 1.97.1, то есть
требование COSMIC ниже нашего и конфликтом не является.

## Общие рантайм-зависимости (RDEPEND всех 30 пакетов)

Источник: `eclass/cosmic-common.eclass` (блок `RDEPEND`). В eclass прямо сказано, почему они
вынесены в eclass, а не в метопакет: их тянет `libcosmic` → `iced` → GTK/dbus/systemd, и Portage
иначе ставил бы их параллельно с первым же пакетом COSMIC.

| Gentoo-атом (мин. версия из eclass) | Debian/Ubuntu (предположение) | На этой машине |
|---|---|---|
| `>=dev-libs/glib-2.78.3` | `libglib2.0-dev` | `glib-2.84.4-r5` ✅ |
| `>=dev-cpp/glibmm-2.66.7:2` | `libglibmm-2.68-dev` | `glibmm-2.84.0` ✅ |
| `>=gui-libs/gtk-4.12.5` | `libgtk-4-dev` | `gtk-4.18.6-r1` ✅ |
| `>=x11-libs/cairo-1.18` | `libcairo2-dev` | `cairo-1.18.4-r1` ✅ |
| `>=x11-libs/pango-1.52.1` | `libpango1.0-dev` | `pango-1.57.0` ✅ |
| `>=x11-libs/gdk-pixbuf-2.42.10-r1` | `libgdk-pixbuf-2.0-dev` | `gdk-pixbuf-2.42.12` ✅ |
| `>=media-libs/graphene-1.10.8` | `libgraphene-1.0-dev` | `graphene-1.10.8-r1` ✅ |
| `>=x11-libs/libxkbcommon-1.6.0` | `libxkbcommon-dev` | `libxkbcommon-1.13.1` ✅ |
| `>=dev-libs/wayland-1.22` | `libwayland-dev` | **нет** ❌ |
| `>=dev-libs/libinput-1.25.0` | `libinput-dev` | `libinput-1.31.1` ✅ |
| `>=media-libs/gstreamer-1.22.11` | `libgstreamer1.0-dev` | `gstreamer-1.24.13` ✅ |
| `>=media-video/pipewire-1.0.5` | `libpipewire-0.3-dev` | `pipewire-1.6.4` ✅ |
| `>=media-libs/libpulse-17.0` | `libpulse-dev` | `libpulse-17.0` ✅ |
| `>=media-libs/libglvnd-1.7.0` | `libglvnd-dev` | `libglvnd-1.7.0` ✅ |
| `>=sys-auth/polkit-123-r1` | `libpolkit-agent-1-dev`, `libpolkit-gobject-1-dev` | `polkit-126-r3` ✅ |
| `>=virtual/libudev-251-r2` | `libudev-dev` | `libudev-257` ✅ |
| `systemd? ( sys-apps/systemd:= )` \| `elogind? ( >=sys-auth/elogind-246.10-r3 )` | `libsystemd-dev` / `libelogind-dev` | `elogind-255.24` ✅ (из `systemd` — только `systemd-utils`) |
| `\|\| ( >=sys-apps/dbus-1.15.8 >=sys-apps/dbus-broker-36 )` | `libdbus-1-dev` | `dbus-1.16.2` ✅ |
| `>=media-fonts/open-sans-1-r1` | `fonts-open-sans` | **нет** ❌ |

Флаги eclass: `debug`, `debug-line-tables-only`, `max-opt` (`debug` и `max-opt` взаимоисключающи),
`elogind`, `systemd`, при этом `^^ ( elogind systemd )` — ровно одно из двух.

## Точечные зависимости (кто что тянет сверх eclass)

Источник — ebuild'ы; перечислены только пакеты, у которых есть свои DEPEND/RDEPEND.

| Пакет оверлея | Тянет | На этой машине |
|---|---|---|
| `cosmic-base/cosmic-viewer` | `media-libs/libheif ≥1.17`, `virtual/pkgconfig` | `libheif-1.20.2` ✅ |
| `cosmic-base/cosmic-wallpapers` | `media-gfx/imagemagick` | есть (`import`) ✅ |
| `cosmic-base/cosmic-meta` | `gnome-base/gnome-keyring ≥46.2`, `media-fonts/{open-sans,noto}` | `gnome-keyring-48.0-r1` ✅, шрифтов **нет** ❌ |
| `cosmic-base/pop-appstream-data` | `dev-libs/appstream ≥1.0.6`-по-ebuild (`0.16.4`), `dev-build/make` | `appstream-1.0.6` ✅ |
| `cosmic-base/pop-icon-theme` | `dev-build/meson`, `x11-themes/adwaita-icon-theme`, `x11-themes/hicolor-icon-theme` | `meson-1.11.1`, `adwaita-icon-theme-48.1`, `hicolor-icon-theme-0.17` ✅ |
| `cosmic-base/pop-fonts` | `dev-build/make` | `make-4.4.1-r102` ✅ |
| `cosmic-extra/cutecosmic` | `dev-qt/qtbase`, `dev-qt/qtdeclarative`, `dev-build/cmake`, `dev-build/corrosion` | Qt 6.11.1 ✅, **`corrosion` нет** ❌ |
| `x11-themes/adw-gtk3` | `gui-libs/gtk-4 ≥4.16`, `dev-util/dart-sass`, `dev-util/dart-sass-bin` | GTK ✅, **dart-sass нет** ❌ |
| `dev-util/dart-sass` | `dev-lang/dart ≥3`, `dev-ruby/sass`, `dev-util/buf`, `net-misc/curl`, `app-arch/xz-utils`, `git` | **dart/sass/buf нет** ❌ (`go-1.26.3` есть) |
| `dev-util/buf` | `dev-lang/go ≥1.20` | `go-1.26.3` ✅ |
| `cosmic-utils/minimon` | `dev-util/pkgconf`, `dev-vcs/git`, `virtual/pkgconfig` | `pkgconf-2.5.1` ✅ |
| `virtual/notification-daemon` | альтернатива из 13 реализаций, в том числе `x11-misc/mate-notification-daemon` | `mate-notification-daemon-1.28.1-r2` ✅ |

`cosmic-utils/*` (13 сторонних апплетов) сверх eclass не тянут ничего, кроме pkgconfig/pkgconf.

## Итог по этой машине (Gentoo, `/var/db/pkg`)

Для сборки DE не хватает четырёх атомов:

```
dev-libs/wayland            # общий RDEPEND, нужен всем 30 пакетам
media-fonts/open-sans       # общий RDEPEND + USE=fonts
media-fonts/noto            # USE=fonts
dev-build/corrosion         # только cosmic-extra/cutecosmic
dev-util/dart-sass          # только x11-themes/adw-gtk3 и pop-theme-meta
```

Оверлей на машине **не зарегистрирован** (`/var/db/repos/` содержит только `gentoo`,
`/etc/portage/repos.conf/` о cosmic молчит), то есть `emerge cosmic-meta` сейчас покраснеет не по
зависимостям, а по отсутствию репозитория. Подключение (из README оверлея, не исполнялось):
`eselect repository add cosmic-overlay git https://github.com/fsvm88/cosmic-overlay.git`, затем
маскировка `cosmic-base/*` в `package.accept_keywords`.

## Что это значит для Светлицы

1. **Для среды ворот** (`docs/env-checklist.md`) из этого списка полезны две строки: тулчейн-гейт
   rust/LLVM и перечень GTK/пакетных имён для Ubuntu-гостя. Остальное — про чужую сборку.
2. **Для ADR-0002 (лицензии)**: сам оверлей распространяется под MIT (`eclass`), но ebuild
   `cosmic-meta` объявляет микс `CC-BY-SA-4.0 GPL-3 GPL-3+ MPL-2.0` и прямо пишет «no guarantee» —
   то есть даже автор оверлея не ручается за лицензии подсобираемых crates. Это аргумент за то,
   чтобы вердикт `добавляем` выдавать только после чтения `LICENSE` конкретного крейта (D5 в design).
3. **Стек не совпадает**: 19 общих зависимостей — это GTK4/iced-путь, Светлица — EFL/Elementary.
   Ни одна строка этого списка не становится нашим требованием автоматически.
