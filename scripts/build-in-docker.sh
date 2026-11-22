#!/bin/sh
# ============================================================================
#  Прогон чего угодно внутри среды ворот (ТЗ 3.8, ADR-0013, ADR-0023).
#
#  Хостом может быть Linux или macOS (ADR-0023, поправка 1): доказательством
#  считается прогон **внутри контейнера по digest'у**, а не «на хосте собралось».
#  Поэтому скрипт не спрашивает, какая ОС снаружи, — он всегда запускает образ.
#
#  POSIX/BusyBox-совместимый (ТЗ 12.20).
#
#  Использование:
#    scripts/build-in-docker.sh build     # собрать образ ворот
#    scripts/build-in-docker.sh doctor    # xtask ci --doctor внутри контейнера
#    scripts/build-in-docker.sh efl       # собрать EFL+Elementary (риск №1)
#    scripts/build-in-docker.sh ci        # полный прогон ворот
#    scripts/build-in-docker.sh tests     # meson test в EFL (задача 2.2)
#    scripts/build-in-docker.sh shell     # интерактивная оболочка
#    scripts/build-in-docker.sh -- <команда> [аргументы…]
# ============================================================================
set -eu

REPO=$(cd "$(dirname "$0")/.." && pwd)
IMAGE=${GATE_IMAGE:-svetlitsa-gate:local}
# Минимум свободного места на разделе Docker (ТЗ 3.8, Q84). Ниже — ворота всё
# равно краснеют, но лучше узнать об этом до `docker build`, чем через 10 минут.
MIN_FREE_KB=${MIN_FREE_KB:-62914560}   # 60 ГиБ
PLATFORM=${GATE_PLATFORM:-linux/amd64} # Apple Silicon: amd64 через эмуляцию (ADR-0023, поправка 1)

EFLLIB_VOL=${EFLLIB_VOL:-svetlitsa-efl}
DIST_VOL=${DIST_VOL:-svetlitsa-dist}
CARGO_VOL=${CARGO_VOL:-svetlitsa-cargo}

need_docker() {
	command -v docker >/dev/null 2>&1 || { echo "нет docker на хосте — ворота не запускаются (ADR-0013)" >&2; exit 2; }
	docker info >/dev/null 2>&1 || { echo "docker не отвечает (демон запущен?)" >&2; exit 2; }
}

disk_guard() {
	root=$(docker info 2>/dev/null | sed -n 's/^ Docker Root Dir: //p')
	# Docker Desktop (macOS): «Docker Root Dir» — путь внутри ВМ, на хосте его нет,
	# и проверка молча уходила. Кончается там не том с томами, а раздел, на котором
	# лежит образ ВМ (vms/0/data/Docker.raw) — его и мерим (Q102).
	df -k "$root" >/dev/null 2>&1 || root=$HOME/Library/Containers/com.docker.docker
	[ -d "$root" ] || root=$REPO
	free=$(df -k "$root" 2>/dev/null | awk 'NR==2 {print $4}')
	[ -n "$free" ] || return 0
	echo "== свободное место на разделе Docker ($root: $(df -h "$root" | awk 'NR==2 {print $4" из "$2", "$5}'))"
	if [ "$free" -lt "$MIN_FREE_KB" ]; then
		echo "   ВНИМАНИЕ: нужно >= $((MIN_FREE_KB / 1024 / 1024)) ГиБ (ТЗ 3.8, Q84)." >&2
		echo "   Продолжаем на свой риск: C-замыкание и target/ съедят гигабайты." >&2
	fi
}

run() {
	docker run --rm -i \
		--platform "$PLATFORM" \
		-v "$REPO:/src" \
		-v "${EFLLIB_VOL}:/opt/svetlitsa-efl" \
		-v "${DIST_VOL}:/opt/dist" \
		-v "${CARGO_VOL}:/opt/cargo" \
		-w /src \
		"$IMAGE" "$@"
}

case ${1:---help} in
  build)
	need_docker
	disk_guard
	echo "== собираю образ $IMAGE"
	docker build --platform "$PLATFORM" -t "$IMAGE" -f "$REPO/Dockerfile" "$REPO"
	echo "== версия компилятора внутри образа (это свойство образа, не rust-toolchain.toml):"
	run rustc --version
	;;
  doctor)
	need_docker; run cargo run -p xtask -- ci --doctor ;;
  ci)
	need_docker; run cargo run -p xtask -- ci ;;
  efl)
	need_docker; disk_guard; run ./scripts/efl-build.sh ;;
  tests)
	need_docker; run env EFL_RUN_TESTS=1 ./scripts/efl-build.sh ;;
  shell)
	need_docker; run /bin/sh ;;
  --)
	need_docker; shift; run "$@" ;;
  *)
	sed -n '3,20p' "$0" ;;
esac
