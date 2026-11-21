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
> **MVP по COSMIC (ТЗ 9.30, 12.16):** граница из трёх крейтов: демон / хранилище истории / конфиг. Это отправная версия, а не паритет: позиции
> паритета MATE ниже остаются незакрытыми, пока их не примет ADR.

Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
