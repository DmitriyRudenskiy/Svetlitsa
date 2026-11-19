# Парити-лист `lavka` ← `mate-panel` (М2)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-panel` · HEAD `2ff4ee7` · модулей (`.c`): 103

| Каталог | Модули |
|---|---|
| `applets/clock` | `calendar-client.c`, `calendar-sources.c`, `calendar-window.c`, `clock-face.c`, `clock-location-tile.c`, `clock-location.c`, `clock-map.c`, `clock-sunpos.c`, `clock-utils.c`, `clock.c`, `set-timezone.c`, `system-timezone.c` |
| `applets/fish` | `fish.c` |
| `applets/notification_area` | `main.c`, `na-grid.c`, `na-host.c`, `na-item.c` |
| `applets/notification_area/libstatus-notifier-watcher` | `gf-sn-watcher-v0.c`, `gf-status-notifier-watcher.c` |
| `applets/notification_area/status-notifier` | `sn-dbus-menu-item.c`, `sn-dbus-menu.c`, `sn-flat-button.c`, `sn-host-v0.c`, `sn-item-v0.c`, `sn-item.c` |
| `applets/notification_area/system-tray` | `fixedtip.c`, `na-tray-child.c`, `na-tray-manager.c`, `na-tray.c` |
| `applets/wncklet` | `showdesktop.c`, `wayland-backend.c`, `wayland-window-menu.c`, `wayland-workspace.c`, `window-list.c`, `window-menu.c`, `wncklet.c`, `workspace-switcher.c` |
| `libmate-panel-applet` | `mate-panel-applet-factory.c`, `mate-panel-applet-gsettings.c`, `mate-panel-applet.c`, `panel-plug.c` |
| `mate-panel` | `applet.c`, `button-widget.c`, `drawer.c`, `launcher.c`, `main.c`, `mate-desktop-item-edit.c`, `menu.c`, `panel-a11y.c`, `panel-action-button.c`, `panel-action-protocol.c`, `panel-addto.c`, `panel-applet-frame.c`, `panel-applet-info.c`, `panel-applets-manager.c`, `panel-background.c`, `panel-bindings.c`, `panel-config-global.c`, `panel-context-menu.c`, `panel-ditem-editor.c`, `panel-force-quit.c`, `panel-frame.c`, `panel-layout.c`, `panel-lockdown.c`, `panel-menu-bar.c`, `panel-menu-button.c`, `panel-menu-items.c`, `panel-modules.c`, `panel-multimonitor.c`, `panel-profile.c`, `panel-properties-dialog.c`, `panel-recent.c`, `panel-reset.c`, `panel-run-dialog.c`, `panel-separator.c`, `panel-session.c`, `panel-shell.c`, `panel-stock-icons.c`, `panel-struts.c`, `panel-test-applets.c`, `panel-toplevel.c`, `panel-util.c`, `panel-widget.c`, `panel-xutils.c`, `panel.c`, `wayland-backend.c`, `xstuff.c` |
| `mate-panel/libmate-panel-applet-private` | `panel-applet-container.c`, `panel-applet-frame-dbus.c`, `panel-applet-mini.c`, `panel-applets-manager-dbus.c` |
| `mate-panel/libpanel-util` | `panel-cleanup.c`, `panel-color.c`, `panel-error.c`, `panel-glib.c`, `panel-gtk.c`, `panel-icon-chooser.c`, `panel-keyfile.c`, `panel-launch.c`, `panel-list.c`, `panel-session-manager.c`, `panel-show.c`, `panel-xdg.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
