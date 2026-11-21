# Парити-лист `pristroyki` ← `caja-extensions` (М4)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/caja-extensions` · HEAD `bb163a5` · модулей (`.c`): 26

| Каталог | Модули |
|---|---|
| `audio-video-properties` | `bacon-video-widget-properties.c`, `totem-gst-helpers.c`, `totem-properties-main.c`, `totem-properties-view.c` |
| `gksu` | `libcaja-gksu.c` |
| `image-converter` | `caja-image-converter.c`, `caja-image-resizer.c`, `caja-image-rotator.c`, `image-converter.c` |
| `open-terminal` | `caja-open-terminal.c`, `open-terminal.c` |
| `sendto` | `caja-nste.c`, `caja-sendto-command.c`, `caja-sendto-module.c` |
| `sendto/plugins` | `nst-common.c` |
| `sendto/plugins/caja-burn` | `caja-burn.c` |
| `sendto/plugins/emailclient` | `emailclient.c` |
| `sendto/plugins/gajim` | `gajim.c` |
| `sendto/plugins/pidgin` | `pidgin.c` |
| `sendto/plugins/removable-devices` | `removable-devices.c` |
| `sendto/plugins/upnp` | `upnp.c` |
| `share` | `caja-share.c`, `shares.c` |
| `wallpaper` | `caja-wallpaper-command.c`, `caja-wallpaper-extension.c` |
| `xattr-tags` | `caja-xattr-tags-extension.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
> **Внешний референс (COSMIC):** аналога нет; из `pop-launcher` (5 236 строк) берём механику поиска плагинов и изоляцию падения — см. ТЗ 9.18. Проверено по 136 ebuild `fsvm88/cosmic-overlay`
> и 345 репозиториям org `pop-os` ([`sravnenie.md`](../analysis/cosmic/sravnenie.md), строка 18). Позиции ниже портируются по
> upstream; сверять «как у COSMIC» не с чем.

> **MVP по COSMIC (ТЗ 9.30, 12.16):** расширение = отдельный процесс, каталогов поиска три. Это отправная версия, а не паритет:
> позиции паритета MATE ниже остаются незакрытыми, пока их не примет ADR.

Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
