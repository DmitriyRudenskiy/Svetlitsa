# 01 · Собрать Docker-образ ворот на Alpine и получить в нём зелёный `--doctor`

Статус: открыт — упирается в демон хоста · Раунд 13 · ТЗ 3.8, Q94, ADR-0015

## Зачем

Четыре раунда М0 стоит в статусе «ожидает среду»: план «QEMU/KVM-VM в облаке» не исполнен
ни разу, smoke не запускался ни разу. Q94 меняет форму среды: образ в репозитории.

## Шаг 0 (сегодня не пройден)

Демон не отвечает. Проверено 2026-11-20:

```console
$ docker version          # висит >25 с, убит; сокет ~/.docker/run/docker.sock молчит
$ pgrep -fl com.docker    # com.docker.backend запущен (pid 6829, 6835)
```

Задний план жив, сокета как ответа нет — значит образ собрать нельзя, и `Dockerfile`
вслепую не пишется (ТЗ 3.8, ADR-0006). Первое действие человека: открыть Docker Desktop
(или поставить colima/podman) и добиться `docker version` с выводом.

## Что делает тикет (после шага 0, в один день с первой сборкой)

1. `Dockerfile` на `alpine:latest` (digest пинится в manifest): `build-base`, `meson`,
   `ninja-build`, `pkgconf`, `xorg-server`, `xvfb-run`, `xdotool`, `imagemagick`,
   `gettext`, `git`, `rsync`, `cargo-deny`, `rustfmt`, `rust-clippy`, `rust-bindgen`.
2. Rust — rustup по `rust-toolchain.toml` (пакетный `rust` из Alpine не используется, ТЗ 5).
3. `target/` и `vendor/mate/` — именованные volumes; режим mount выбирается замером, а не
   вкусовщиной: bind-mount macOS-дерева на Rust-сборке замедляет компиляцию в разы.
   Основное ядро образа — musl (Q95): `alpine:latest` уже musl, отдельного `musl-gcc` для
   чистого Rust-ковдра не требуется, он понадобится ровно в тот день, когда появится C.
4. `docker run … cargo test -p xtask` и `docker run … xtask ci --doctor` — вывод в журнал.

## Метрики, которые тикет обязан назвать числом

| Что мерить | Зачем |
|---|---|
| время `cargo test -p xtask` в `linux/arm64` | базовая цена контейнера на этом хосте |
| то же в `--platform linux/amd64` | цена эмуляции; целевая триплета продукту ещё не названа (ADR-0015 п. 2) |
| bind-mount против volume | решение про mount, а не вера в статьи |
| размер образа | «Alpine вместо 25 ГБ гостя» должно быть числом |

## Что тикет НЕ решает

- EFL/Elementary в образе — тикет 02.
- smoke-раннер — тикет 03, он без 02 не имеет смысла.
- glibc: второго образа не заводим (Q95), проверкой упаковки в glibc-среде занимаемся в M5 —
  тикет 04, он же единственное место, где glibc когда-либо поминается вслух.
- musl vs glibc — отдельное решение (ADR-0015 п. 3), не «потом».

## Проверка

`docker build -t svetlitsa-gate:local .` · `docker run --rm svetlitsa-gate:local xtask ci --doctor`
