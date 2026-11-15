# Парити-лист `ambar-core` ← `caja/libcaja-extension` (М4)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/caja/libcaja-extension` · HEAD `4de76eb` · модулей (`.c`): 13

| Каталог | Модули |
|---|---|
| `.` | `caja-column-provider.c`, `caja-column.c`, `caja-configurable.c`, `caja-extension-types.c`, `caja-file-info.c`, `caja-info-provider.c`, `caja-location-widget-provider.c`, `caja-menu-item.c`, `caja-menu-provider.c`, `caja-menu.c`, `caja-property-page-provider.c`, `caja-property-page.c`, `caja-widget-view-provider.c` |

## Функции (70) — публичная поверхность `libcaja-extension`

`caja_column_get_type`, `caja_column_new`, `caja_column_provider_get_columns`, `caja_column_provider_get_type`, `caja_configurable_get_type`, `caja_configurable_run_config`, `caja_file_info_add_emblem`, `caja_file_info_add_string_attribute`, `caja_file_info_can_write`, `caja_file_info_create`, `caja_file_info_create_for_uri`, `caja_file_info_get_activation_uri`, `caja_file_info_get_file_type`, `caja_file_info_get_location`, `caja_file_info_get_mime_type`, `caja_file_info_get_mount`, `caja_file_info_get_name`, `caja_file_info_get_parent_info`, `caja_file_info_get_parent_location`, `caja_file_info_get_parent_uri`, `caja_file_info_get_string_attribute`, `caja_file_info_get_type`, `caja_file_info_get_uri`, `caja_file_info_get_uri_scheme`, `caja_file_info_invalidate_extension_info`, `caja_file_info_is_directory`, `caja_file_info_is_gone`, `caja_file_info_is_mime_type`, `caja_file_info_list_copy`, `caja_file_info_list_free`, `caja_file_info_lookup`, `caja_file_info_lookup_for_uri`, `caja_info_provider_cancel_update`, `caja_info_provider_get_type`, `caja_info_provider_update_complete_invoke`, `caja_info_provider_update_file_info`, `caja_location_widget_provider_get_type`, `caja_location_widget_provider_get_widget`, `caja_menu_append_item`, `caja_menu_get_items`, `caja_menu_get_type`, `caja_menu_item_activate`, `caja_menu_item_get_type`, `caja_menu_item_list_free`, `caja_menu_item_new`, `caja_menu_item_set_submenu`, `caja_menu_new`, `caja_menu_provider_emit_items_updated_signal`, `caja_menu_provider_get_background_items`, `caja_menu_provider_get_file_items`, `caja_menu_provider_get_toolbar_items`, `caja_menu_provider_get_type`, `caja_module_initialize`, `caja_module_list_pyfiles`, `caja_module_list_types`, `caja_module_shutdown`, `caja_operation_result_get_type`, `caja_property_page_get_type`, `caja_property_page_new`, `caja_property_page_provider_get_pages`, `caja_property_page_provider_get_type`, `caja_widget_view_provider_add_file`, `caja_widget_view_provider_clear`, `caja_widget_view_provider_get_first_visible_file`, `caja_widget_view_provider_get_item_count`, `caja_widget_view_provider_get_type`, `caja_widget_view_provider_get_widget`, `caja_widget_view_provider_set_location`, `caja_widget_view_provider_set_window`, `caja_widget_view_provider_supports_uri`
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
