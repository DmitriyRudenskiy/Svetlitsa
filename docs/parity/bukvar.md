# Парити-лист `bukvar` ← `libmatekbd` (М1)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/libmatekbd` · HEAD `b53e553` · модулей (`.c`): 7

| Каталог | Модули |
|---|---|
| `libmatekbd` | `matekbd-desktop-config.c`, `matekbd-indicator-config.c`, `matekbd-indicator.c`, `matekbd-keyboard-config.c`, `matekbd-keyboard-drawing.c`, `matekbd-status.c`, `matekbd-util.c` |

## Функции (74) — публичная поверхность `libmatekbd`

`matekbd_desktop_config_activate`, `matekbd_desktop_config_init`, `matekbd_desktop_config_load_from_gsettings`, `matekbd_desktop_config_load_group_descriptions`, `matekbd_desktop_config_lock_next_group`, `matekbd_desktop_config_lock_prev_group`, `matekbd_desktop_config_restore_group`, `matekbd_desktop_config_save_to_gsettings`, `matekbd_desktop_config_start_listen`, `matekbd_desktop_config_stop_listen`, `matekbd_desktop_config_term`, `matekbd_indicator_config_activate`, `matekbd_indicator_config_free_image_filenames`, `matekbd_indicator_config_get_images_file`, `matekbd_indicator_config_init`, `matekbd_indicator_config_load_from_gsettings`, `matekbd_indicator_config_load_image_filenames`, `matekbd_indicator_config_refresh_style`, `matekbd_indicator_config_save_to_gsettings`, `matekbd_indicator_config_start_listen`, `matekbd_indicator_config_stop_listen`, `matekbd_indicator_config_term`, `matekbd_indicator_get_group_names`, `matekbd_indicator_get_image_filename`, `matekbd_indicator_get_max_width_height_ratio`, `matekbd_indicator_get_type`, `matekbd_indicator_get_xkl_engine`, `matekbd_indicator_new`, `matekbd_indicator_reinit_ui`, `matekbd_indicator_set_angle`, `matekbd_indicator_set_parent_tooltips`, `matekbd_indicator_set_tooltips_format`, `matekbd_install_glib_log_appender`, `matekbd_keyboard_config_activate`, `matekbd_keyboard_config_add_default_switch_option_if_necessary`, `matekbd_keyboard_config_equals`, `matekbd_keyboard_config_format_full_layout`, `matekbd_keyboard_config_get_descriptions`, `matekbd_keyboard_config_init`, `matekbd_keyboard_config_load_from_gsettings`, `matekbd_keyboard_config_load_from_x_current`, `matekbd_keyboard_config_load_from_x_initial`, `matekbd_keyboard_config_merge_items`, `matekbd_keyboard_config_save_to_gsettings`, `matekbd_keyboard_config_split_items`, `matekbd_keyboard_config_start_listen`, `matekbd_keyboard_config_stop_listen`, `matekbd_keyboard_config_term`, `matekbd_keyboard_config_to_string`, `matekbd_keyboard_drawing_get_compat`, `matekbd_keyboard_drawing_get_geometry`, `matekbd_keyboard_drawing_get_keycodes`, `matekbd_keyboard_drawing_get_symbols`, `matekbd_keyboard_drawing_get_type`, `matekbd_keyboard_drawing_get_types`, `matekbd_keyboard_drawing_new`, `matekbd_keyboard_drawing_new_dialog`, `matekbd_keyboard_drawing_print`, `matekbd_keyboard_drawing_render`, `matekbd_keyboard_drawing_set_groups_levels`, `matekbd_keyboard_drawing_set_keyboard`, `matekbd_keyboard_drawing_set_track_config`, `matekbd_keyboard_drawing_set_track_modifiers`, `matekbd_preview_load_position`, `matekbd_preview_save_position`, `matekbd_status_get_group_names`, `matekbd_status_get_image_filename`, `matekbd_status_get_type`, `matekbd_status_get_xkl_engine`, `matekbd_status_new`, `matekbd_status_reinit_ui`, `matekbd_status_set_angle`, `matekbd_status_set_tooltips_format`, `matekbd_strv_append`
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
