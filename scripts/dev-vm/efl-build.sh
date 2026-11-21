#!/bin/sh
# Сборка EFL 1.29.0 в /opt/svetlitsa-efl внутри образа ворот (ADR-0003, ADR-0015, Q95).
# Elementary — часть этого же тарбóла, отдельного его шага нет.
#
# Скрипт написан в тот же день, когда его прогнали (ТЗ 3.8, ADR-0006). Запуск:
#   docker run --rm -v "$PWD":/src -w /src \
#     -v svetlitsa-efl:/opt/svetlitsa-efl -v svetlitsa-efl-build:/var/tmp/efl-build \
#     svetlitsa-gate:local scripts/dev-vm/efl-build.sh
#
# Две вещи здесь не вкусовщина:
#   * дерево сборки живёт вне /src — bind-mount macOS-дерева на сборке замедляет компиляцию
#     (замер тикета 01: 5.1 с против 3.7 с на маленьком дереве, и это маленькое дерево);
#   * -j 4 вместо девяти ninja: в ВМ 7.65 ГиБ памяти, а линковка EFL — не компиляция одного
#     файла. Кто упрётся в OOM, поднимает JOBS и смотрит, что побьёт раньше — память или время.
set -eu

EFL_VERSION=1.29.0
EFL_SHA256=248bc76141f2b8b006ccba9e0bb2cb46cbcb1be4181a1cf2728187f778853654
EFL_URL="https://download.enlightenment.org/rel/libs/efl/efl-${EFL_VERSION}.tar.xz"
PREFIX="${PREFIX:-/opt/svetlitsa-efl}"
BUILD="${BUILD:-/var/tmp/efl-build}"
JOBS="${JOBS:-4}"

# Зависимости — makedepends из aports `testing/efl/APKBUILD` (35 имён, meson/ninja уже в образе).
# Если 1.29.0 потребует большего, вывод apk add идёт в журнал как есть, а не «починим потом».
DEPS="
  dbus-dev eudev-dev fontconfig-dev freetype-dev fribidi-dev giflib-dev
  gst-plugins-base-dev gstreamer-dev harfbuzz-dev libheif-dev libinput-dev
  libjpeg-turbo-dev libpng-dev librsvg-dev libsndfile-dev libwebp-dev
  libx11-dev libxcomposite-dev libxcursor-dev libxdamage-dev libxinerama-dev
  libxkbcommon-dev libxrandr-dev libxscrnsaver-dev libxtst-dev lua5.2-dev
  mesa-dev openjpeg-dev openssl-dev pulseaudio-dev wayland-dev
  wayland-protocols xorgproto
"

echo "=== apk add ($(echo $DEPS | wc -w) пакетов, версия 3.24)"
apk add --no-cache $DEPS

mkdir -p "$BUILD"
cd "$BUILD"
TARBALL="efl-${EFL_VERSION}.tar.xz"
if [ ! -f "$TARBALL" ]; then
    echo "=== скачивание $EFL_URL"
    curl -fL --retry 3 -sS -o "$TARBALL" "$EFL_URL"
fi
echo "=== sha256 (пин из IMAGE-MANIFEST.md)"
ls -l "$TARBALL"
echo "$EFL_SHA256  $TARBALL" | sha256sum -c -
tar xf "$TARBALL"

# Флаги — из build() того же APKBUILD, с тремя отклонениями, названными вслух:
#   -Db_lto=false — LTO в aports включена, но в ВМ на 7.65 ГиБ это риск OOM на линковке,
#     а ворота не измеряют быстродействие релиза;
#   -Dpipewire=false — в 1.29 этот аудио-бэкенд включён по умолчанию и требует
#     `libpipewire-0.3 >= 1.4.0`, которого в makedepends aports нет (в 1.28.1 его вообще не
#     спрашивают). Без флага meson падает: `Dependency "libpipewire-0.3" not found`. Пакет
#     в 3.24 есть (`pipewire-dev 1.6.8-r4`, community), но мы его не ставим: ворота звук не
#     тестируют, а pulseaudio из aports-списка остаётся единственным аудио-бэкендом. Для
#     M5-пакета (тикет 04) это решение надо пересматривать — там PipeWire и есть звук.
#     Заодно совпадает с планом проекта по звуку: v1 — PulseAudio-совместимый слой,
#     PipeWire напрямую — v2 (TASK.md, решения по звуку);
#   -Dprefix — наш префикс, потому что EFL ставится не в /usr, а в /opt/svetlitsa-efl (ADR-0003).
echo "=== meson setup"
rm -rf build # неудачный configure оставляет занятую директорию — дешевле пересоздать
CFLAGS="${CFLAGS:-} -D_LARGEFILE64_SOURCE -D__USE_MISC" meson setup \
    -Db_lto=false \
    -Ddebug=false \
    -Doptimization=s \
    -Dnls=false \
    -Dopengl=es-egl \
    -Dxinput22=true \
    -Dbuffer=false \
    -Dfb=false \
    -Ddrm=true \
    -Dtslib=false \
    -Dharfbuzz=true \
    -Dwl=true \
    -Dnetwork-backend=connman \
    -Dpipewire=false \
    -Devas-loaders-disabler=pdf,ps,heif,raw,tiff,json,avif,jxl \
    -Dphysics=false \
    -Dsystemd=false \
    -Davahi=false \
    -Dbindings= \
    -Dbuild-examples=false \
    -Dbuild-tests=false \
    -Decore-imf-loaders-disabler=ibus,scim \
    -Dprefix="$PREFIX" \
    "efl-${EFL_VERSION}" build

echo "=== сборка (-j $JOBS)"
ninja -C build -j "$JOBS"

echo "=== установка в $PREFIX"
meson install -C build

echo "=== то, ради чего всё это"
PKG_CONFIG_PATH="$PREFIX"/lib/pkgconfig pkg-config --modversion elementary edje eet efreet ecore-evas

echo "=== недостающая величина: размер дерева сборки (для пересчёта порога doctor'а)"
du -sh "$BUILD/efl-${EFL_VERSION}" "$BUILD/build" "$PREFIX" 2>/dev/null
df -kP / | tail -1
