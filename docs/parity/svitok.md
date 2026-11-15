# Парити-лист `svitok` ← `atril` (М4)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/atril` · HEAD `a049ebf` · модулей (`.c`): 140

| Каталог | Модули |
|---|---|
| `backend/comics` | `comics-document.c`, `ev-archive.c` |
| `backend/djvu` | `djvu-document.c`, `djvu-links.c`, `djvu-text-page.c` |
| `backend/dvi` | `cairo-device.c`, `dvi-document.c`, `fonts.c` |
| `backend/dvi/mdvi-lib` | `afmparse.c`, `bitmap.c`, `color.c`, `common.c`, `dviread.c`, `files.c`, `font.c`, `fontmap.c`, `fontsrch.c`, `gf.c`, `hash.c`, `list.c`, `pagesel.c`, `paper.c`, `pk.c`, `setup.c`, `sp-epsf.c`, `special.c`, `t1.c`, `tfm.c`, `tfmfile.c`, `tt.c`, `util.c`, `vf.c` |
| `backend/epub` | `epub-document.c` |
| `backend/pixbuf` | `pixbuf-document.c` |
| `backend/ps` | `ev-spectre.c` |
| `backend/tiff` | `tiff-document.c`, `tiff2ps.c` |
| `backend/xps` | `xps-document.c` |
| `cut-n-paste/toolbar-editor` | `egg-editable-toolbar.c`, `egg-toolbar-editor.c`, `egg-toolbars-model.c` |
| `cut-n-paste/zoom-control` | `ephy-zoom-action.c`, `ephy-zoom-control.c`, `ephy-zoom.c` |
| `libdocument` | `ev-annotation.c`, `ev-async-renderer.c`, `ev-attachment.c`, `ev-backends-manager.c`, `ev-debug.c`, `ev-document-annotations.c`, `ev-document-attachments.c`, `ev-document-factory.c`, `ev-document-find.c`, `ev-document-fonts.c`, `ev-document-forms.c`, `ev-document-images.c`, `ev-document-layers.c`, `ev-document-links.c`, `ev-document-misc.c`, `ev-document-print.c`, `ev-document-security.c`, `ev-document-text.c`, `ev-document-thumbnails.c`, `ev-document-transition.c`, `ev-document.c`, `ev-file-exporter.c`, `ev-file-helpers.c`, `ev-form-field.c`, `ev-image.c`, `ev-init.c`, `ev-layer.c`, `ev-link-action.c`, `ev-link-dest.c`, `ev-link.c`, `ev-mapping-list.c`, `ev-module.c`, `ev-page.c`, `ev-render-context.c`, `ev-selection.c`, `ev-transition-effect.c` |
| `libmisc` | `ev-page-action-widget.c`, `ev-page-action.c` |
| `libview` | `ev-annotation-window.c`, `ev-document-model.c`, `ev-form-field-accessible.c`, `ev-image-accessible.c`, `ev-job-scheduler.c`, `ev-jobs.c`, `ev-link-accessible.c`, `ev-page-accessible.c`, `ev-page-cache.c`, `ev-pixbuf-cache.c`, `ev-print-operation.c`, `ev-stock-icons.c`, `ev-timeline.c`, `ev-transition-animation.c`, `ev-view-accessible.c`, `ev-view-cursor.c`, `ev-view-presentation.c`, `ev-view.c`, `ev-web-view.c` |
| `previewer` | `ev-previewer-window.c`, `ev-previewer.c` |
| `properties` | `ev-properties-main.c`, `ev-properties-view.c` |
| `shell` | `eggfindbar.c`, `ev-annotation-properties-dialog.c`, `ev-application.c`, `ev-bookmark-action.c`, `ev-bookmarks.c`, `ev-daemon.c`, `ev-file-monitor.c`, `ev-find-sidebar.c`, `ev-history.c`, `ev-keyring.c`, `ev-loading-message.c`, `ev-media-player-keys.c`, `ev-message-area.c`, `ev-metadata.c`, `ev-navigation-action-widget.c`, `ev-navigation-action.c`, `ev-open-recent-action.c`, `ev-password-view.c`, `ev-progress-message-area.c`, `ev-properties-dialog.c`, `ev-properties-fonts.c`, `ev-properties-license.c`, `ev-sidebar-annotations.c`, `ev-sidebar-attachments.c`, `ev-sidebar-bookmarks.c`, `ev-sidebar-layers.c`, `ev-sidebar-links.c`, `ev-sidebar-page.c`, `ev-sidebar-thumbnails.c`, `ev-sidebar.c`, `ev-utils.c`, `ev-window-title.c`, `ev-window.c`, `main.c` |
| `thumbnailer` | `atril-thumbnailer.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
