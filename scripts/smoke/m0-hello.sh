#!/bin/sh
# ============================================================================
#  М0-smoke: окно `svetlitsa-ui --bin hello` под headless Wayland-композитором.
#  ТЗ 10.2 (контур итерации 10.3), ADR-0016, ADR-0024, Q34.
#
#  Что доказывается: крейт поднимает настоящее окно Elementary 1.29 поверх
#  EFL 1.29 в musl-среде ворот (риск №1 закрыт сборкой), окно доживает до
#  закрытия, колбэк удаления Eo срабатывает. Это то, чем М0-позиция становится
#  «готово» по ТЗ 6.9.
#
#  Чего здесь сознательно нет:
#   * снимков — ТЗ 10.3 отдаёт захват композитора нам же (`starshina`, М2),
#     `xwd`/`import` в воротах нет (ADR-0016), а weston 16 отвечает
#     `Output capture error: unauthorized` на weston_capture_v1;
#   * нажатий — ТЗ 10.2 требует virtual-keyboard-unstable-v1 и крейт
#     `svetlitsa-input`, weston этот глобал не объявляет (см. тикет 03).
#
#  POSIX/BusyBox-совместимо (ТЗ 12.20). Запускать изнутри образа ворот:
#    scripts/build-in-docker.sh -- scripts/smoke/m0-hello.sh
# ============================================================================
set -eu

REPO=${REPO:-/src}
EFL_PREFIX=${EFL_PREFIX:-/opt/svetlitsa-efl}
COMPOSITOR=${COMPOSITOR:-weston}
OUTPUT_MODE=${OUTPUT_MODE:-1920x1080}   # ТЗ 10.3: кадр контура итерации
RUNTIME=${XDG_RUNTIME_DIR:-/tmp/svetlitsa-smoke}
OUT=${OUT_DIR:-$REPO/artifacts/smoke}
WAIT_TIMEOUT=${WAIT_TIMEOUT:-60}        # секунд на сокет композитора (поллинг, не sleep)

say() { echo "[smoke] $*"; }
die() { echo "[smoke] БРАК: $*" >&2; exit 1; }

# --- окружение EFL -----------------------------------------------------------
[ -d "$EFL_PREFIX" ] || die "нет EFL в $EFL_PREFIX — нужен том с собранным EFL (Q92)"
LD_LIBRARY_PATH=$EFL_PREFIX/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
PKG_CONFIG_PATH=$EFL_PREFIX/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}
XDG_DATA_DIRS=$EFL_PREFIX/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}
ELM_SEARCH_PATHS=$EFL_PREFIX/share/elementary/themes
export LD_LIBRARY_PATH PKG_CONFIG_PATH XDG_DATA_DIRS ELM_SEARCH_PATHS

# --- бинарь ------------------------------------------------------------------
cd "$REPO"
say "собираю svetlitsa-ui --features efl --bin hello"
cargo build -q -p svetlitsa-ui --features efl --bin hello
BIN=$REPO/target/debug/hello
[ -x "$BIN" ] || die "нет $BIN"

command -v "$COMPOSITOR" >/dev/null 2>&1 || die "нет композитора $COMPOSITOR в образе (ТЗ 10.2)"

mkdir -p "$RUNTIME" "$OUT"
chmod 700 "$RUNTIME"
export XDG_RUNTIME_DIR=$RUNTIME
WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-wayland-1}
export WAYLAND_DISPLAY

# --- композитор --------------------------------------------------------------
WESTON_INI=$OUT/weston.ini
cat >"$WESTON_INI" <<EOF
[core]
renderer=pixman
shell=desktop-shell.so
[output]
name=headless
mode=$OUTPUT_MODE
EOF

say "поднимаю $COMPOSITOR (headless, $OUTPUT_MODE, софт-рендер pixman)"
# shellcheck disable=SC2086
$COMPOSITOR --backend=headless --config="$WESTON_INI" >"$OUT/weston.log" 2>&1 &
COMPOSITOR_PID=$!
# Композitor мрёт сразу, если не может — ждать сокета бессмысленно, тащим лог.
cleanup() {
	if kill -0 "$COMPOSITOR_PID" 2>/dev/null; then kill "$COMPOSITOR_PID" 2>/dev/null || true; fi
	wait "$COMPOSITOR_PID" 2>/dev/null || true
}
fail_with_log() {
	[ -s "$OUT/weston.log" ] && { say "лог композитора (последние 15 строк):"; tail -15 "$OUT/weston.log"; }
	die "$*"
}

# Поллинг, а не sleep (ТЗ 10.2): ждём сокет, потом глобал xdg_wm_base.
i=0
while [ ! -S "$RUNTIME/$WAYLAND_DISPLAY" ]; do
	kill -0 "$COMPOSITOR_PID" 2>/dev/null || fail_with_log "$COMPOSITOR умер при старте"
	i=$((i + 1))
	[ "$i" -ge "$WAIT_TIMEOUT" ] && fail_with_log "нет сокета $RUNTIME/$WAYLAND_DISPLAY за ${WAIT_TIMEOUT}с"
	sleep 1
done
say "сокет $RUNTIME/$WAYLAND_DISPLAY появился за ${i}с"

if command -v wayland-info >/dev/null 2>&1; then
	wayland-info 2>/dev/null | grep -q "interface: 'xdg_wm_base'" \
		|| fail_with_log "композитор не объявил xdg_wm_base"
	say "композитор объявил xdg_wm_base"
fi

# --- тема --------------------------------------------------------------------
# Электронная тема (themes/*.edc) — задача 2.x; если .edj собран, ставим его
# на окно через `--theme` (это путь `elm_layout_file_set`, а не ELM_THEME —
# последний подменяет тему виджетов, а не наш макет).
EDC=$REPO/crates/svetlitsa-ui/themes/svetlitsa.edc
EDJ=$OUT/svetlitsa.edj
THEME_ARG=""
if [ -f "$EDC" ] && command -v edje_cc >/dev/null 2>&1; then
	FONT=$(find /usr/share/fonts -name 'DejaVuSans.ttf' 2>/dev/null | head -1)
	FD=""
	[ -n "$FONT" ] && FD=$(dirname "$FONT") || say "шрифт DejaVu не найден — собираю тему без -fd"
	if [ -n "$FD" ]; then
		edje_cc -fd "$FD" "$EDC" "$EDJ" && THEME_ARG=$EDJ
	else
		edje_cc "$EDC" "$EDJ" && THEME_ARG=$EDJ
	fi
	[ -n "$THEME_ARG" ] && say "макет окна: $THEME_ARG (наш EDC, группа svetlitsa/main)" \
		|| say "тема не собралась — окно без edje-макета"
else
	say "темы (.edc) ещё нет — окно без edje-макета"
fi

# --- прогон окна -------------------------------------------------------------
say "гоняю окно: $BIN${THEME_ARG:+ --theme $THEME_ARG}"
set +e
if [ -n "$THEME_ARG" ]; then
	timeout 60 "$BIN" --theme "$THEME_ARG" >"$OUT/hello.log" 2>&1
else
	timeout 60 "$BIN" >"$OUT/hello.log" 2>&1
fi
RC=$?
set -e
cleanup

grep -v "^ERR<" "$OUT/hello.log" >"$OUT/hello.clean.log" 2>/dev/null || cp "$OUT/hello.log" "$OUT/hello.clean.log"
cat "$OUT/hello.clean.log"

# У утилиты Elementary «выход по событию удаления окна». Ошибка EFL после
# efl_shutdown() в логе — известная болтовня 1.29, решением не считается,
# решает код возврата (см. комментарий в src/bin/hello.rs).
[ "$RC" -eq 0 ] || fail_with_log "окно завершилось с кодом $RC (лог: $OUT/hello.log)"
grep -q "del=true eo_cb_main=true" "$OUT/hello.clean.log" \
	|| fail_with_log "окно закрылось не через Eo-колбэк удаления (лог: $OUT/hello.log)"

say "SMOKE OK: окно Window под $COMPOSITOR прожило сеанс, Eo-колбэк удаления сработал"
say "артефакты: $OUT/{hello.log,weston.log}"
exit 0
