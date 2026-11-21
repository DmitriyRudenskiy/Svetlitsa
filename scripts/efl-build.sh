#!/bin/sh
# ============================================================================
#  Сборка EFL 1.29.0 + Elementary под musl (x86_64-alpine-linux-musl).
#
#  Это проверка риска №1 (openspec/changes/musl-only-libc, задача 2.2): до неё
#  никто не собирал EFL 1.29 под musl — в Alpine есть только 1.28.1, он собран
#  с `!check` и `-Dnls=false`, а пакета Elementary нет вовсе.
#
#  Запускать ВНУТРИ контейнера ворот (scripts/build-in-docker.sh efl). Скрипт
#  намеренно POSIX/BusyBox-совместимый (ТЗ 12.20): никаких GNU-измов, ни
#  `[[ ]]`, ни `local`, ни `grep -P`.
#
#  Elementary отдельным тарболом НЕ существует: с 1.20 он живёт в дереве EFL
#  (src/lib/elementary) и собирается тем же meson'ом (проверено: тег v-1.29.0,
#  meson.build, модуль 'elementary'; последний отдельный релиз — 1.17.1).
#  Поэтому «EFL + Elementary» = один прогон, а не два.
# ============================================================================
set -eu

EFL_VERSION=${EFL_VERSION:-1.29.0}
# Тег в git — «v-1.29.0» (дефис!), а в tarball — «1.29.0». Не путать.
EFL_URL=${EFL_URL:-https://download.enlightenment.org/rel/libs/efl/efl-${EFL_VERSION}.tar.xz}
# sha256 тарбола. Пустое значение = сборка запрещена: скачанное «на глаз»
# доказательством не считается (ТЗ 12.19).
EFL_SHA256=${EFL_SHA256:-248bc76141f2b8b006ccba9e0bb2cb46cbcb1be4181a1cf2728187f778853654}

PREFIX=${EFL_PREFIX:-/opt/svetlitsa-efl}
DIST=${EFL_DIST:-/opt/dist}          # кэш тарболов (именуемый том)
JOBS=${JOBS:-$(nproc)}
SRC=${EFL_SRC:-/tmp/efl-build}
LOGDIR=${LOGDIR:-/src/artifacts}
RUN_TESTS=${EFL_RUN_TESTS:-0}        # 1 = прогнать meson test (задача 2.2: !check не включаем)

mkdir -p "$DIST" "$SRC" "$PREFIX" "$LOGDIR"
LOG="$LOGDIR/efl-build-$(date -u +%Y%m%dT%H%M%SZ).log"
TMP="$LOG.part"
exec 3>&1
say() { printf '%s\n' "$*" | tee -a "$LOG" >&3; }

# Команды запускаются НЕ через `| tee`: в BusyBox ash нет pipefail, поэтому в
# конвейере теряется статус возврата, и сборка печатала «ГОТОВО» на упавшем meson
# (проверено 21.11.2026 — ТЗ 3.8 «непрогнанный скрипт это почти зелёное»). Пишем
# вывод в файл, наружу печатаем хвост, падаем по настоящему коду возврата.
run() {
	rc=0
	"$@" >"$TMP" 2>&1 || rc=$?
	cat "$TMP" >>"$LOG"
	tail -n 40 "$TMP" >&3
	if [ "$rc" -ne 0 ]; then
		say "ОШИБКА: команда вернула $rc: $*"
		return "$rc"
	fi
	return 0
}

# ---------------------------------------------------------------- 0. окружение
say "== среда сборки ($(date -u +%FT%TZ))"
say "   cc:      $(cc --version 2>/dev/null | head -1)"
say "   meson:   $(meson --version)"
say "   ninja:   $(ninja --version)"
say "   libc:    $(ls /lib/ld-musl-x86_64.so.1 >/dev/null 2>&1 && echo musl || echo 'НЕ musl')"
say "   jobs:    $JOBS"

# ------------------------------------------------------------------ 1. исходник
TARBALL="$DIST/efl-${EFL_VERSION}.tar.xz"
if [ ! -f "$TARBALL" ]; then
	say "== скачиваю $EFL_URL"
	run curl -fL --retry 3 -o "$TARBALL.part" "$EFL_URL"
	mv "$TARBALL.part" "$TARBALL"
fi
case "$EFL_SHA256" in
  ""|PLACEHOLDER)
	say "ОШИБКА: EFL_SHA256 не задан. Хеш тарбола: $(sha256sum "$TARBALL" | cut -d' ' -f1)"
	exit 2 ;;
esac
say "== проверяю sha256 тарбола"
if ! { printf '%s  %s\n' "$EFL_SHA256" "$TARBALL" | sha256sum -c - >>"$LOG" 2>&1; }; then
	say "ОШИБКА: sha256 тарбола не совпал с EFL_SHA256 (фактический: $(sha256sum "$TARBALL" | cut -d' ' -f1))"
	exit 2
fi

if [ ! -d "$SRC/efl-$EFL_VERSION" ]; then
	say "== раскладываю тарбол"
	tar -xJf "$TARBALL" -C "$SRC"
fi
cd "$SRC/efl-$EFL_VERSION"

# ------------------------------------------------------------------- 2. флаги
# Каждый флаг — это решение, а не настройка по умолчанию.
#   x11/xpresent/xinput2*/fb/sdl/cocoa=false — только Wayland (ADR-0016);
#     ecore_x в дереве остаётся (это код оригинала), но не линкуется.
#   systemd=false — в образе openrc, systemd нет и не будет (ADR-0023).
#   pulseaudio=false, pipewire=true — один звуковой слой (ADR-0022); xtask deps
#     красен на два слоя, и мы не должны тащить второй через EFL.
#   nls=true — вопреки Alpine (-Dnls=false): переводы нам нужны, каталоги .mo
#     бандлим. Локальные категории (месяцы, сортировка) musl не даёт — они на
#     Rust (ADR-0023), это не отменяет перевода строк EFL.
#   build-tests=true — вопреки Alpine (-Dbuild-tests=false + !check): тесты
#     компилируются, прогон отдельно (RUN_TESTS), чтобы провал теста не прятал
#     ошибку компиляции.
#   opengl=es-egl — как в aports: headless-композиторы дают GLES.
#   elua/bindings/mono — привязки не нужны, они только раздувают сборку.
#   loaders-disabler — форматы, которых нет в образе (pdf/ps/heif/raw/tiff/
#     json/avif/jxl); остаются png/jpeg/gif/webp/svg/bmp/ico/eet.
say "== настраиваю meson"
rm -rf build
# Исходник и каталог сборки передаются ОДИН раз: `meson setup build … . build`
# роняет meson лишним positional-аргументом (проверено 21.11.2026).
run meson setup build \
	--prefix="$PREFIX" \
	--libdir=lib \
	--buildtype=debugoptimized \
	--wrap-mode=nodownload \
	--auto-features=disabled \
	-Dnls=true \
	-Daudio=true -Dpipewire=true -Dpulseaudio=false -Dgstreamer=true \
	-Dwl=true -Ddrm=true \
	-Dx11=false -Dxpresent=false -Dxinput2=false -Dxinput22=false \
	-Dfb=false -Dsdl=false -Dcocoa=false -Dbuffer=false \
	-Dwl-deprecated=false -Ddrm-deprecated=false \
	-Dsystemd=false -Davahi=false -Dnetwork-backend=none \
	-Dharfbuzz=true -Dfribidi=true -Dfontconfig=true -Dpixman=true \
	-Dhyphen=false -Dedje-sound-and-video=true \
	-Dinput=true -Deeze=true -Dtslib=false -Dv4l2=false -Dvnc-server=false \
	-Dphysics=false -Dopengl=es-egl \
	-Db_lto=false -Ddocs=false -Dnative-arch-optimization=false \
	-Dbindings= -Delua=false -Dmono-beta=false \
	-Dbuild-examples=false -Dbuild-tests=true \
	-Decore-imf-loaders-disabler=ibus,scim \
	-Devas-loaders-disabler=pdf,ps,heif,raw,tiff,json,avif,jxl \
	.

# ---------------------------------------------------------------- 3. сборка
say "== компиляция (-j$JOBS)"
run meson compile -C build

say "== установка в $PREFIX"
run env DESTDIR= meson install --no-rebuild -C build

# ------------------------------------------------------- 4. доказательства
say "== что получилось"
say "   pkg-config efl:  $(PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig pkg-config --modversion efl)"
say "   elementary.h:    $(ls "$PREFIX"/include/elementary-* -d 2>/dev/null || echo НЕТ)"
say "   библиотек .so:   $(find "$PREFIX"/lib -name '*.so.*' | wc -l | tr -d ' ')"
say "   загрузчик:       $(readelf -l "$PREFIX"/lib/libelementary.so 2>/dev/null | grep -o 'ld-musl[^ ]*' | head -1)"
say "   dlopen-модули:   $(grep -c dlopen src/lib/eina/eina_module.c 2>/dev/null || echo '?') вызовов в eina_module.c"
say "   X11 в линковке:  $(find "$PREFIX"/lib -name '*.so*' -exec ldd {} \; 2>/dev/null | grep -ciE 'libX11|libxcb' || true)"
say "   лог:             $LOG"

if [ "$RUN_TESTS" = 1 ]; then
	say "== meson test (!check не включаем — задача 2.2)"
	run meson test -C build --print-errorlogs
fi

# Доказательства обязательны: «скрипт завершился» ≠ «библиотеки есть». Пока их не
# проверить, любой провал выше выглядел как успех (см. ловушку с `| tee` выше).
say "== проверяю доказательства"
EFL_OK=1
[ "$(PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig pkg-config --modversion efl 2>/dev/null)" = "$EFL_VERSION" ] \
	|| { say "ОШИБКА: pkg-config не отдаёт efl $EFL_VERSION"; EFL_OK=0; }
[ -f "$PREFIX/lib/libelementary.so" ] \
	|| { say "ОШИБКА: нет $PREFIX/lib/libelementary.so"; EFL_OK=0; }
[ "$EFL_OK" = 1 ] || { say "СБОРКА НЕ ПРИНЯТА, лог: $LOG"; exit 1; }

say "== ГОТОВО: EFL $EFL_VERSION + Elementary под musl"
