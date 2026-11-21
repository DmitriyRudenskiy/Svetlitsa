# Парити-лист `uzelok` ← `engrampa` (М3)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/engrampa` · HEAD `dbad544` · модулей (`.c`): 56

| Каталог | Модули |
|---|---|
| `caja` | `caja-engrampa.c`, `engrampa-module.c` |
| `src` | `actions.c`, `dlg-add-files.c`, `dlg-add-folder.c`, `dlg-ask-password.c`, `dlg-batch-add.c`, `dlg-delete.c`, `dlg-extract.c`, `dlg-new.c`, `dlg-open-with.c`, `dlg-package-installer.c`, `dlg-password.c`, `dlg-prop.c`, `dlg-update.c`, `eggfileformatchooser.c`, `eggtreemultidnd.c`, `file-data.c`, `file-utils.c`, `fr-archive.c`, `fr-command-7z.c`, `fr-command-ace.c`, `fr-command-alz.c`, `fr-command-ar.c`, `fr-command-arj.c`, `fr-command-cfile.c`, `fr-command-cpio.c`, `fr-command-dpkg.c`, `fr-command-iso.c`, `fr-command-jar.c`, `fr-command-lha.c`, `fr-command-lrzip.c`, `fr-command-nomarch.c`, `fr-command-rar.c`, `fr-command-rpm.c`, `fr-command-tar.c`, `fr-command-unarchiver.c`, `fr-command-unstuff.c`, `fr-command-zip.c`, `fr-command-zoo.c`, `fr-command.c`, `fr-error.c`, `fr-init.c`, `fr-list-model.c`, `fr-proc-error.c`, `fr-process.c`, `fr-window.c`, `gio-utils.c`, `glib-utils.c`, `gtk-utils.c`, `java-utils.c`, `main.c`, `open-file.c`, `preferences.c`, `rar-utils.c`, `server.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
> **Внешний референс (COSMIC):** отсутствует — отдельного архиватора нет, архив открывает сам `cosmic-files` (`src/archive.rs`). Проверено по 136 ebuild `fsvm88/cosmic-overlay`
> и 345 репозиториям org `pop-os` ([`sravnenie.md`](../analysis/cosmic/sravnenie.md), строка 14). Позиции ниже портируются по
> upstream; сверять «как у COSMIC» не с чем.

Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
