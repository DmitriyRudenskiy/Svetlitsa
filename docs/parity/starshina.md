# Парити-лист `starshina` ← `marco` (М2)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/marco` · HEAD `8522ed1` · модулей (`.c`): 48

| Каталог | Модули |
|---|---|
| `src/compositor` | `compositor-xrender.c`, `compositor.c` |
| `src/core` | `async-getprop.c`, `bell.c`, `boxes.c`, `constraints.c`, `core.c`, `delete.c`, `display.c`, `edge-resistance.c`, `effects.c`, `errors.c`, `eventqueue.c`, `frame.c`, `group-props.c`, `group.c`, `iconcache.c`, `keybindings.c`, `main.c`, `place.c`, `prefs.c`, `schema-bindings.c`, `screen.c`, `session.c`, `stack.c`, `util.c`, `window-props.c`, `window.c`, `workspace.c`, `xprops.c` |
| `src/tools` | `marco-message.c`, `marco-window-demo.c` |
| `src/ui` | `draw-workspace.c`, `fixedtip.c`, `frames.c`, `gradient.c`, `menu.c`, `metaaccellabel.c`, `preview-widget.c`, `resizepopup.c`, `tabpopup.c`, `theme-parser.c`, `theme-viewer.c`, `theme.c`, `tile-preview.c`, `ui.c` |
| `src/wm-tester` | `focus-window.c`, `main.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
