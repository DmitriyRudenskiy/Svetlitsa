# Парити-лист `klyuchnik` ← `mate-session-manager` (М2)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-session-manager` · HEAD `419f2ce` · модулей (`.c`): 34

| Каталог | Модули |
|---|---|
| `capplet` | `gsm-app-dialog.c`, `gsm-properties-dialog.c`, `gsp-app-manager.c`, `gsp-app.c`, `gsp-keyfile.c`, `main.c`, `msm-desktop-app-dialog.c` |
| `mate-session` | `gs-idle-monitor.c`, `gsm-app.c`, `gsm-autostart-app.c`, `gsm-client.c`, `gsm-consolekit.c`, `gsm-dbus-client.c`, `gsm-inhibit-dialog.c`, `gsm-inhibitor.c`, `gsm-logout-dialog.c`, `gsm-manager.c`, `gsm-presence.c`, `gsm-session-save.c`, `gsm-store.c`, `gsm-systemd.c`, `gsm-util.c`, `gsm-xsmp-client.c`, `gsm-xsmp-server.c`, `main.c`, `mdm-log.c`, `mdm-signal-handler.c`, `mdm.c`, `msm-gnome.c` |
| `tools` | `mate-session-check-accelerated-gl-helper.c`, `mate-session-check-accelerated-gles-helper.c`, `mate-session-check-accelerated.c`, `mate-session-inhibit.c`, `mate-session-save.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|