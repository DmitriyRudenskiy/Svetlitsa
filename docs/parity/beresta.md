# Парити-лист `beresta` ← `pluma` (М3)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/pluma` · HEAD `b5e619e` · модулей (`.c`): 81

| Каталог | Модули |
|---|---|
| `plugins/docinfo` | `pluma-docinfo-plugin.c` |
| `plugins/filebrowser` | `pluma-file-bookmarks-store.c`, `pluma-file-browser-messages.c`, `pluma-file-browser-plugin.c`, `pluma-file-browser-store.c`, `pluma-file-browser-utils.c`, `pluma-file-browser-view.c`, `pluma-file-browser-widget.c` |
| `plugins/modelines` | `modeline-parser.c`, `pluma-modeline-plugin.c` |
| `plugins/sort` | `pluma-sort-plugin.c` |
| `plugins/spell` | `pluma-automatic-spell-checker.c`, `pluma-spell-checker-dialog.c`, `pluma-spell-checker-language.c`, `pluma-spell-checker.c`, `pluma-spell-language-dialog.c`, `pluma-spell-plugin.c`, `pluma-spell-utils.c` |
| `plugins/taglist` | `pluma-taglist-plugin-panel.c`, `pluma-taglist-plugin-parser.c`, `pluma-taglist-plugin.c` |
| `plugins/time` | `pluma-time-plugin.c` |
| `plugins/trailsave` | `pluma-trail-save-plugin.c` |
| `pluma` | `bacon-message-connection.c`, `pluma-app-activatable.c`, `pluma-app.c`, `pluma-application.c`, `pluma-close-button.c`, `pluma-commands-documents.c`, `pluma-commands-edit.c`, `pluma-commands-file-print.c`, `pluma-commands-file.c`, `pluma-commands-help.c`, `pluma-commands-search.c`, `pluma-commands-view.c`, `pluma-debug.c`, `pluma-dirs.c`, `pluma-document-input-stream.c`, `pluma-document-loader.c`, `pluma-document-output-stream.c`, `pluma-document-saver.c`, `pluma-document.c`, `pluma-documents-panel.c`, `pluma-encodings-combo-box.c`, `pluma-encodings.c`, `pluma-file-chooser-dialog.c`, `pluma-help.c`, `pluma-history-entry.c`, `pluma-io-error-message-area.c`, `pluma-language-manager.c`, `pluma-message-bus.c`, `pluma-message-type.c`, `pluma-message.c`, `pluma-metadata-manager.c`, `pluma-notebook.c`, `pluma-panel.c`, `pluma-pango.c`, `pluma-plugins-engine.c`, `pluma-print-job.c`, `pluma-print-preview.c`, `pluma-progress-message-area.c`, `pluma-session.c`, `pluma-settings.c`, `pluma-smart-charset-converter.c`, `pluma-status-combo-box.c`, `pluma-statusbar.c`, `pluma-style-scheme-manager.c`, `pluma-tab-label.c`, `pluma-tab.c`, `pluma-utils.c`, `pluma-view-activatable.c`, `pluma-view.c`, `pluma-window-activatable.c`, `pluma-window.c`, `pluma.c`, `plumatextregion.c` |
| `pluma/dialogs` | `pluma-close-confirmation-dialog.c`, `pluma-encodings-dialog.c`, `pluma-preferences-dialog.c`, `pluma-search-dialog.c` |
| `tools/plugin_template` | `pluma-plugin.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|