# Парити-лист `vestnik` ← `mate-notification-daemon` (М2)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-notification-daemon` · HEAD `3ce6b85` · модулей (`.c`): 14

| Каталог | Модули |
|---|---|
| `src/capplet` | `mate-notification-applet-dbus.c`, `mate-notification-applet-history.c`, `mate-notification-applet.c`, `mate-notification-properties.c` |
| `src/daemon` | `daemon.c`, `engines.c`, `mnd-daemon.c`, `sound.c`, `stack.c`, `wayland.c` |
| `src/themes/coco` | `coco-theme.c` |
| `src/themes/nodoka` | `nodoka-theme.c` |
| `src/themes/slider` | `theme.c` |
| `src/themes/standard` | `theme.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|