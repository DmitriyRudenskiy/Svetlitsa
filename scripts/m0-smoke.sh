#!/bin/sh
# Прогон контрольной точки М0: сценарий живёт в `xtask smoke hello` (спека add-smoke),
# здесь — только вход в образ ворот. Зеркало вендора приезжает отдельным ro-монтированием:
# `vendor` — симлинк на внешний том, и внутри контейнера он обязан стать настоящим
# каталогом, иначе parity/icons краснеют «зеркала нет».
set -eu
cd "$(dirname "$0")/.."

set -- -w /src -v "$PWD":/src -e CARGO_TARGET_DIR=/tmp/tgt
if [ -L vendor ]; then
	set -- "$@" -v "$(cd vendor && pwd -P):/src/vendor:ro"
fi
exec docker run --rm "$@" svetlitsa-gate:local cargo run -q -p xtask -- smoke hello
