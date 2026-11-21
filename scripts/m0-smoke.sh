#!/bin/sh
# Ручной прогон smoke-окна в образе ворот (M0, итерация 4 — зелёная).
# Вызывается снаружи: docker-контейнер с примонтированным /src.
# Обязательное окружение: ELM_AUTO_NORENDER_WITHDRAWN=0 — без оконного
# менеджера (Xvfb) Ecore_X считает окно withdrawn, и Elementary выключает
# отрисовку (кадр чёрный). На настоящем рабочем столе с WM не нужно,
# но держать переменную безвредно.
set -eu

FDIR=$(dirname "$(fc-match -f %{file})")
/opt/svetlitsa-efl/bin/edje_cc -fd "$FDIR" \
  /src/crates/svetlitsa-ui/themes/svetlitsa.edc /tmp/theme.eo

cargo build -q --manifest-path /src/Cargo.toml -p svetlitsa-ui \
  --bin hello --features efl --offline

Xvfb :99 -screen 0 800x480x24 >/tmp/xvfb.log 2>&1 &
sleep 2
export DISPLAY=:99 ELM_AUTO_NORENDER_WITHDRAWN=0

/tmp/tgt/debug/hello --theme /tmp/theme.eo >/tmp/hello.log 2>&1 &
HPID=$!
sleep 4
W=$(xdotool search --onlyvisible --class exe | head -1)
[ -n "$W" ] || { echo "окно не найдено"; cat /tmp/hello.log; exit 1; }

# клик по строке «Файл» (верхняя строка genlist под заголовком edje)
xdotool mousemove 60 130 click 1
sleep 1
import -window "$W" /src/artifacts/m0-smoke.png
xdotool windowclose "$W"
wait $HPID
tail -1 /tmp/hello.log   # «окно закрыто; clicked=true del=true eo_cb_main=true»
