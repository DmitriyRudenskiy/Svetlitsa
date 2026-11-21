# Manifest образа ворот `svetlitsa-gate` (ADR-0015, ADR-0013)

Рядом с `Dockerfile`. Строка журнала «Образы» ссылается на этот файл; всё ниже — вывод
команд, а не намерения. Дата замера: **2026-11-21**, хост Apple Silicon (`arm64`),
Docker Desktop 27.4.0, 8 CPU / 7.65 ГиБ выделено ВМ.

## Базовый образ

| Поле | Значение |
|---|---|
| тег | `alpine:3.24` (не `latest`: `latest` сегодня и через месяц — разные образы) |
| digest | `alpine@sha256:f59abe656b692d901e184be3445e084786602108599d2a51d229c0f5a48c71bf` |
| image id | `sha256:abbc135a7d3909a5eb6a56aac4c769cf15be025e3c9e226b8660f2b2b378158b` |
| миниrootfs | `alpine-minirootfs-3.24.4-aarch64.tar.gz`, 8.66 МБ |
| триплета | `aarch64-unknown-linux-musl` — её сообщает сам rustup в образе, musl подтверждён не словами |

## Готовый образ

| Поле | Значение |
|---|---|
| тег | `svetlitsa-gate:local` |
| image id | `sha256:2abc6c94974d1e9e3e2ee21fda51193e7fd79a203c13989fd268e8c7c26b15df` |
| размер | **2.36 ГБ** = 8.66 МБ базы + 618 МБ `apk add` + 1.74 ГБ тулчейна (`/usr/local/rustup` — 1.6 ГБ, из них ~0.5 скачанные тарбóлы: чистить будем, если упрёмся в размер) |
| время `docker build` с нуля | **7 м 15 с** (слой тулчейна — 221 с, почти всё — сеть) |

## Тулчейн

Пин не дублируется в образе: `COPY rust-toolchain.toml` → `rustup toolchain install` читает
канал и компоненты из того же файла (ТЗ 3.1). Проверка встроена в сборку — `RUN ... rustc --version`:

```console
rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo 1.97.1 (c980f4866 2026-06-30)
```

rustup 1.29.3, `--profile minimal`. Пакетного `rust` из Alpine в образе нет: в стабильной 3.24
это 1.96.1, ниже пина (проверено по APKINDEX), и второй `rustc` в PATH сбивает doctor.

## `apk add` (14 имён, версия снята `apk policy` в самом образе)

`build-base 0.5-r4` · `meson 1.11.1-r0` · `ninja-build 1.13.2-r1` · `pkgconf 2.5.1-r0` ·
`git 2.54.1-r0` · `rsync 3.5.2-r1` · `gettext 1.0-r0` · `xz`, `tar`, `curl` ·
`xorg-server 21.1.26-r0` · `xvfb-run 1.20.10.3-r2` · `xdotool 4.20260303.1-r0` ·
`imagemagick 7.1.2.35-r0` · `cargo-deny 0.18.6-r1`

Сомнение из `docs/env-checklist.md` про имя бинарника в ImageMagick 7 закрыто: Alpine 3.24 даёт
**7.1.2-35** и `/usr/bin/import`, который ищет doctor, — мажор 7 на имя не влияет. Проверено
`import --version` внутри образа.

## Tarball'ы

| Файл | sha256 | в образе? |
|---|---|---|
| `efl-1.29.0.tar.xz` (168 895 856 байт) | `248bc76141f2b8b006ccba9e0bb2cb46cbcb1be4181a1cf2728187f778853654` | **нет** — это тикет 02; Elementary отдельным тарбóлом не существует, он внутри этого же архива |

## Как воспроизвести и как проверить

```console
$ docker build -t svetlitsa-gate:local .
$ docker run --rm -v "$PWD":/src svetlitsa-gate:local            # xtask ci --doctor
$ docker run --rm -v "$PWD":/src svetlitsa-gate:local cargo test --offline --workspace
```

Репозиторий в образ не копируется (`.dockerignore` оставляет в контексте один
`rust-toolchain.toml`): он приезжает монтированием, а куда класть `target/` решает замер из
тикета 01, а не вкус.
