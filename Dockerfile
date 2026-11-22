# syntax=docker/dockerfile:1.6
# ============================================================================
#  Среда ворот Svetlitsa — ТЗ 3.8, ADR-0013, ADR-0023 (решение 6).
#
#  ПИН. Базовый образ фиксируется **digest'ом**, а не тегом: после отказа от
#  rustup версия компилятора — это свойство образа, а не `rust-toolchain.toml`
#  (он объявлен минимальной версией). Менять digest — только вместе с записью
#  в docs/journal.md и с перепр прогоном ворот.
#      alpine:edge, linux/amd64
#      sha256:266f29255458134745f2bf588cb23ed1ed1768b96ff2580a05d70a8aba59e145
#      снят 2026-11-22 (`docker manifest inspect --verbose alpine:edge`).
#  Пинним манифест **одной архитектуры**: проект только x86_64 (ADR-0023),
#  поэтому манифест-лист был бы лишней степенью свободы.
#
#  Хост. Одинаково Linux и macOS (ADR-0023, поправка 1): внутри контейнера
#  среда идентична, но часть проверок на macOS не выполняется и не засчитывается —
#  перечень в ADR-0023, «Поправка 1».
#
#  EFL/Elementary сюда НЕ пекутся на каждой итерации: их собирает
#  scripts/efl-build.sh в именуемый том (-v svetlitsa-efl:/opt/svetlitsa-efl),
#  иначе каждый прогон платил бы минутами компиляции C-замыкания. В golden
#  image (ADR-0013) тот же скрипт исполняется один раз при сборке образа.
# ============================================================================
FROM alpine:edge@sha256:266f29255458134745f2bf588cb23ed1ed1768b96ff2580a05d70a8aba59e145

# edge в образе даёт main+community; testing нужен для weston/cage/clang16/
# ninja-build (в main их нет) — см. таблицу доступности в docs/journal.md.
RUN printf '%s\n' \
      https://dl-cdn.alpinelinux.org/alpine/edge/main \
      https://dl-cdn.alpinelinux.org/alpine/edge/community \
      https://dl-cdn.alpinelinux.org/alpine/edge/testing \
      > /etc/apk/repositories \
 && apk --no-cache upgrade

# --- тулчейн --------------------------------------------------------------
# rust/cargo — пакеты Alpine: тройка `x86_64-alpine-linux-musl`, динамическая
# линковка по умолчанию (ADR-0023). clang16 кладём рядом с gcc: воротам хватает
# gcc (именно им собирается efl в aports), clang — второй прогон и `bindgen`.
# `openssh` — из-за `ssh-keygen`: им `xtask canon check` проверяет штампы canon-r4…r11
# (Q58); без него канон красится «cannot run ssh-keygen» (journal, 21.11.2026).
RUN apk --no-cache add \
      rust cargo rustfmt rust-clippy cargo-deny sccache \
      build-base gcc make pkgconf meson ninja-build clang16 \
      musl-dev linux-headers \
      git curl xz tar jq python3 diffutils grep openssh \
      scdoc gettext gettext-dev

# --- C-замыкание EFL 1.29 --------------------------------------------------
# Список — makedepends пакета efl 1.28.1 из Alpine testing (единственный
# работающий прецедент сборки EFL под musl), за вычетом X11 (libx11/libxcb/
# xorgproto/…) и PulseAudio: у нас только Wayland (ADR-0016) и только
# PipeWire (ADR-0022). Добавлено: pipewire-dev, at-spi2-core-dev (elementary),
# jansson-dev, libdrm-dev.
#
# lua5.2-dev и check-dev — не из aports, а из прогона: meson EFL 1.29 перебирает
# pkg-config имена `lua, lua51, lua-5.1, lua5.1, lua52, lua-5.2, lua5.2` и падает
# на `ERROR: Problem encountered: Lua not found`, если 5.2 нет; lua5.4-dev в edge
# есть, но им EFL 1.29 не смотрит. check-dev нужен потому, что в скрипте стоит
# `-Dbuild-tests=true` (иначе `Dependency "check" not found`). Проверено
# `meson setup` на том же наборе флагов: exit=0, Elementary сконфигурирован
# (22.11.2026, тикет 02 из `.scratch/gate-alpine`).
RUN apk --no-cache add \
      dbus-dev eudev-dev libdrm-dev libinput-dev at-spi2-core-dev \
      fontconfig-dev freetype-dev fribidi-dev harfbuzz-dev \
      giflib-dev libjpeg-turbo-dev libpng-dev libwebp-dev librsvg-dev openjpeg-dev \
      glib-dev gstreamer-dev gst-plugins-base-dev libsndfile-dev \
      pipewire-dev openssl-dev jansson-dev lua5.2-dev check-dev \
      wayland-dev wayland-protocols libxkbcommon-dev mesa-dev mesa-demos

# --- headless-сеанс для смоков (ТЗ 10.2) ----------------------------------
RUN apk --no-cache add weston cage wayland-utils seatd libseat-dev

# --- шрифты ----------------------------------------------------------------
# PT Sans / PT Serif в Alpine нет (проверено по APKINDEX edge), поэтому в
# golden image они бандлом из репозитория (roadmap, раздел 5). До этой задачи
# ставим DejaVu, чтобы «серая тема» и скриншоты вообще имели чем рисоваться.
RUN apk --no-cache add font-dejavu

# XDG_RUNTIME_DIR для headless-композитора: /run/user/0 в контейнере никто не
# создаёт (systemd-logind у нас нет и быть не может, ADR-0023), поэтому заводим
# каталог с правами 700 прямо в образе.
RUN mkdir -m 700 -p /tmp/xdg-runtime

ENV PKG_CONFIG_PATH=/opt/svetlitsa-efl/lib/pkgconfig \
    LD_LIBRARY_PATH=/opt/svetlitsa-efl/lib \
    PATH=/opt/svetlitsa-efl/bin:$PATH \
    CARGO_HOME=/opt/cargo \
    XDG_RUNTIME_DIR=/tmp/xdg-runtime

WORKDIR /src
CMD ["./scripts/build-in-docker.sh", "--help"]
