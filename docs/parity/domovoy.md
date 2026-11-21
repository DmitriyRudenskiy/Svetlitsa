# Парити-лист `domovoy` ← `mate-settings-daemon` (М2)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-settings-daemon` · HEAD `927e9dc` · модулей (`.c`): 74

| Каталог | Модули |
|---|---|
| `mate-settings-daemon` | `main.c`, `mate-settings-bus.c`, `mate-settings-manager.c`, `mate-settings-module.c`, `mate-settings-plugin-info.c`, `mate-settings-plugin.c`, `mate-settings-profile.c` |
| `plugins/a11y-keyboard` | `msd-a11y-keyboard-atspi.c`, `msd-a11y-keyboard-manager.c`, `msd-a11y-keyboard-plugin.c`, `msd-a11y-preferences-dialog.c` |
| `plugins/a11y-settings` | `msd-a11y-settings-manager.c`, `msd-a11y-settings-plugin.c` |
| `plugins/background` | `msd-background-manager-wayland.c`, `msd-background-manager.c`, `msd-background-plugin.c` |
| `plugins/clipboard` | `list.c`, `msd-clipboard-manager-wayland.c`, `msd-clipboard-manager.c`, `msd-clipboard-plugin.c`, `xutils.c` |
| `plugins/common` | `eggaccelerators.c`, `msd-input-helper.c`, `msd-keygrab.c`, `msd-osd-window.c` |
| `plugins/datetime` | `msd-datetime-mechanism-main.c`, `msd-datetime-mechanism.c`, `system-timezone.c` |
| `plugins/dummy` | `msd-dummy-manager.c`, `msd-dummy-plugin.c` |
| `plugins/housekeeping` | `msd-disk-space.c`, `msd-housekeeping-manager.c`, `msd-housekeeping-plugin.c`, `msd-ldsm-dialog.c`, `msd-ldsm-trash-empty.c` |
| `plugins/keybindings` | `dconf-util.c`, `msd-keybindings-manager.c`, `msd-keybindings-plugin.c` |
| `plugins/keyboard` | `delayed-dialog.c`, `msd-keyboard-manager.c`, `msd-keyboard-plugin.c`, `msd-keyboard-xkb.c` |
| `plugins/media-keys` | `msd-media-keys-manager.c`, `msd-media-keys-plugin.c`, `msd-media-keys-window.c` |
| `plugins/mouse` | `msd-locate-pointer.c`, `msd-mouse-manager.c`, `msd-mouse-plugin.c`, `msd-timeline.c` |
| `plugins/mpris` | `bus-watch-namespace.c`, `msd-mpris-manager.c`, `msd-mpris-plugin.c` |
| `plugins/rfkill` | `msd-rfkill-manager.c`, `msd-rfkill-plugin.c`, `rfkill-glib.c` |
| `plugins/smartcard` | `msd-smartcard-manager.c`, `msd-smartcard-plugin.c`, `msd-smartcard.c` |
| `plugins/sound` | `msd-sound-manager.c`, `msd-sound-plugin.c` |
| `plugins/typing-break` | `msd-typing-break-manager.c`, `msd-typing-break-plugin.c` |
| `plugins/wlrandr` | `msd-wlrandr-manager.c`, `msd-wlrandr-plugin.c` |
| `plugins/xrandr` | `msd-xrandr-manager.c`, `msd-xrandr-plugin.c` |
| `plugins/xrdb` | `msd-xrdb-manager.c`, `msd-xrdb-plugin.c` |
| `plugins/xsettings` | `fontconfig-monitor.c`, `msd-xsettings-manager.c`, `msd-xsettings-plugin.c`, `wm-common.c`, `xsettings-common.c`, `xsettings-manager.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
> **MVP по COSMIC (ТЗ 9.30, 12.16):** интерфейс домового — varlink вместо D-Bus-фасада. Это отправная версия, а не паритет: позиции
> паритета MATE ниже остаются незакрытыми, пока их не примет ADR.

Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
