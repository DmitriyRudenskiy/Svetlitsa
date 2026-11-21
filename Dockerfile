# Среда ворот Светлицы (Q94, ADR-0015): ворота — образ, а не чужая машина.
# Пишется и собирается в тот же день (ТЗ 3.8, ADR-0006); содержимое зафиксировано
# рядом в IMAGE-MANIFEST.md, таблица «Образы» — в docs/journal.md.
#
# docker build -t svetlitsa-gate:local .
# docker run --rm -v "$PWD":/src svetlitsa-gate:local
FROM alpine:3.24

# Список — шаг 1 тикета 01 + зависимости сборки EFL из `testing/efl/APKBUILD`.
# Пакетного rust здесь нет намеренно: в стабильном 3.24 это 1.96.1, ниже пина 1.97.1,
# а второй rustc в PATH сбивает doctor (проверено по APKINDEX, раунд 13, тикет 02).
RUN apk add --no-cache \
      build-base meson ninja-build pkgconf \
      git rsync \
      gettext xz tar curl \
      xorg-server xvfb-run xdotool imagemagick \
      cargo-deny

# Пин тулчейна в образе не дублируется: rustup читает `rust-toolchain.toml`
# (канал + компоненты rustfmt/clippy) и ставит ровно то, что требует ТЗ 3.1.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
COPY rust-toolchain.toml /opt/toolchain/rust-toolchain.toml
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal \
    && cd /opt/toolchain && rustup toolchain install && rustc --version && cargo --version

# Репозиторий в образ не копируется: он приезжает монтированием (см. про замеры
# bind-mount против volume в ADR-0015 п. 4). Без монтирования doctor проверит пустоту.
WORKDIR /src
CMD ["cargo", "run", "--quiet", "-p", "xtask", "--", "ci", "--doctor"]
