# Парити-лист `traktir` ← `mate-menus` (М1)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-menus` · HEAD `182ff87` · модулей (`.c`): 6

| Каталог | Модули |
|---|---|
| `libmenu` | `desktop-entries.c`, `entry-directories.c`, `matemenu-tree.c`, `menu-layout.c`, `menu-monitor.c`, `menu-util.c` |

## Функции (142) — публичная поверхность `libmenu`

`_entry_directory_list_compare`, `_entry_directory_list_empty_desktop_cache`, `_entry_directory_list_get_all_desktops`, `desktop_entry_copy`, `desktop_entry_get_app_info`, `desktop_entry_get_basename`, `desktop_entry_get_comment`, `desktop_entry_get_generic_name`, `desktop_entry_get_hidden`, `desktop_entry_get_icon`, `desktop_entry_get_name`, `desktop_entry_get_no_display`, `desktop_entry_get_path`, `desktop_entry_get_show_in`, `desktop_entry_get_type`, `desktop_entry_has_categories`, `desktop_entry_has_category`, `desktop_entry_new`, `desktop_entry_ref`, `desktop_entry_reload`, `desktop_entry_set_add_entry`, `desktop_entry_set_foreach`, `desktop_entry_set_get_count`, `desktop_entry_set_intersection`, `desktop_entry_set_lookup`, `desktop_entry_set_new`, `desktop_entry_set_ref`, `desktop_entry_set_subtract`, `desktop_entry_set_swap_contents`, `desktop_entry_set_union`, `desktop_entry_set_unref`, `desktop_entry_unref`, `entry_directory_get_flat_contents`, `entry_directory_list_add_monitors`, `entry_directory_list_append_list`, `entry_directory_list_get_directory`, `entry_directory_list_get_length`, `entry_directory_list_new`, `entry_directory_list_prepend`, `entry_directory_list_ref`, `entry_directory_list_remove_monitors`, `entry_directory_list_unref`, `entry_directory_new`, `entry_directory_ref`, `entry_directory_unref`, `matemenu_tree_alias_get_aliased_directory`, `matemenu_tree_alias_get_aliased_entry`, `matemenu_tree_alias_get_aliased_item_type`, `matemenu_tree_alias_get_directory`, `matemenu_tree_alias_get_parent`, `matemenu_tree_alias_get_tree`, `matemenu_tree_alias_get_type`, `matemenu_tree_directory_get_comment`, `matemenu_tree_directory_get_desktop_file_path`, `matemenu_tree_directory_get_generic_name`, `matemenu_tree_directory_get_icon`, `matemenu_tree_directory_get_is_nodisplay`, `matemenu_tree_directory_get_menu_id`, `matemenu_tree_directory_get_name`, `matemenu_tree_directory_get_parent`, `matemenu_tree_directory_get_tree`, `matemenu_tree_directory_get_type`, `matemenu_tree_directory_iter`, `matemenu_tree_directory_make_path`, `matemenu_tree_entry_get_app_info`, `matemenu_tree_entry_get_desktop_file_id`, `matemenu_tree_entry_get_desktop_file_path`, `matemenu_tree_entry_get_is_excluded`, `matemenu_tree_entry_get_is_nodisplay_recurse`, `matemenu_tree_entry_get_is_unallocated`, `matemenu_tree_entry_get_parent`, `matemenu_tree_entry_get_tree`, `matemenu_tree_entry_get_type`, `matemenu_tree_flags_get_type`, `matemenu_tree_get_canonical_menu_path`, `matemenu_tree_get_directory_from_path`, `matemenu_tree_get_entry_by_id`, `matemenu_tree_get_root_directory`, `matemenu_tree_get_type`, `matemenu_tree_header_get_directory`, `matemenu_tree_header_get_parent`, `matemenu_tree_header_get_tree`, `matemenu_tree_header_get_type`, `matemenu_tree_item_ref`, `matemenu_tree_item_unref`, `matemenu_tree_iter_get_alias`, `matemenu_tree_iter_get_directory`, `matemenu_tree_iter_get_entry`, `matemenu_tree_iter_get_header`, `matemenu_tree_iter_get_separator`, `matemenu_tree_iter_get_type`, `matemenu_tree_iter_next`, `matemenu_tree_iter_ref`, `matemenu_tree_iter_unref`, `matemenu_tree_load_sync`, `matemenu_tree_new`, `matemenu_tree_new_for_path`, `matemenu_tree_separator_get_parent`, `matemenu_tree_separator_get_tree`, `matemenu_tree_separator_get_type`, `menu_debug_print_layout`, `menu_get_directory_monitor`, `menu_get_file_monitor`, `menu_layout_load`, `menu_layout_node_append_child`, `menu_layout_node_default_layout_get_values`, `menu_layout_node_get_children`, `menu_layout_node_get_content`, `menu_layout_node_get_content_as_path`, `menu_layout_node_get_next`, `menu_layout_node_get_parent`, `menu_layout_node_get_root`, `menu_layout_node_get_type`, `menu_layout_node_insert_after`, `menu_layout_node_insert_before`, `menu_layout_node_legacy_dir_get_prefix`, `menu_layout_node_legacy_dir_set_prefix`, `menu_layout_node_menu_get_app_dirs`, `menu_layout_node_menu_get_directory_dirs`, `menu_layout_node_menu_get_name`, `menu_layout_node_menuname_get_values`, `menu_layout_node_merge_file_get_type`, `menu_layout_node_merge_file_set_type`, `menu_layout_node_merge_get_type`, `menu_layout_node_move_get_new`, `menu_layout_node_move_get_old`, `menu_layout_node_new`, `menu_layout_node_prepend_child`, `menu_layout_node_ref`, `menu_layout_node_root_add_entries_monitor`, `menu_layout_node_root_get_basedir`, `menu_layout_node_root_get_name`, `menu_layout_node_root_remove_entries_monitor`, `menu_layout_node_set_content`, `menu_layout_node_steal`, `menu_layout_node_unlink`, `menu_layout_node_unref`, `menu_monitor_add_notify`, `menu_monitor_ref`, `menu_monitor_remove_notify`, `menu_monitor_unref`, `menu_verbose`
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
| `libmenu/desktop-entries.c` | модуль |  | подсказка: статус из легенды |
| `libmenu/entry-directories.c` | модуль |  | подсказка: статус из легенды |
| `libmenu/matemenu-tree.c` | модуль |  | подсказка: статус из легенды |
| `libmenu/menu-layout.c` | модуль |  | подсказка: статус из легенды |
| `libmenu/menu-monitor.c` | модуль |  | подсказка: статус из легенды |
| `libmenu/menu-util.c` | модуль |  | подсказка: статус из легенды |
| `libmenu::_entry_directory_list_compare` | функция |  | подсказка: статус из легенды |
| `libmenu::_entry_directory_list_empty_desktop_cache` | функция |  | подсказка: статус из легенды |
| `libmenu::_entry_directory_list_get_all_desktops` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_copy` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_app_info` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_basename` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_comment` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_generic_name` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_hidden` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_icon` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_name` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_no_display` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_path` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_show_in` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_has_categories` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_has_category` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_new` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_reload` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_add_entry` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_foreach` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_get_count` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_intersection` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_lookup` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_new` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_subtract` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_swap_contents` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_union` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_set_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::desktop_entry_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_get_flat_contents` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_add_monitors` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_append_list` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_get_directory` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_get_length` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_new` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_prepend` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_remove_monitors` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_list_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_new` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::entry_directory_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_aliased_directory` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_aliased_entry` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_aliased_item_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_directory` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_parent` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_tree` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_alias_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_comment` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_desktop_file_path` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_generic_name` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_icon` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_is_nodisplay` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_menu_id` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_name` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_parent` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_tree` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_iter` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_directory_make_path` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_app_info` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_desktop_file_id` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_desktop_file_path` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_is_excluded` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_is_nodisplay_recurse` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_is_unallocated` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_parent` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_tree` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_entry_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_flags_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_get_canonical_menu_path` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_get_directory_from_path` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_get_entry_by_id` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_get_root_directory` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_header_get_directory` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_header_get_parent` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_header_get_tree` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_header_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_item_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_item_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_get_alias` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_get_directory` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_get_entry` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_get_header` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_get_separator` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_next` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_iter_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_load_sync` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_new` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_new_for_path` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_separator_get_parent` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_separator_get_tree` | функция |  | подсказка: статус из легенды |
| `libmenu::matemenu_tree_separator_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_debug_print_layout` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_get_directory_monitor` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_get_file_monitor` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_load` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_append_child` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_default_layout_get_values` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_children` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_content` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_content_as_path` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_next` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_parent` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_root` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_insert_after` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_insert_before` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_legacy_dir_get_prefix` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_legacy_dir_set_prefix` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_menu_get_app_dirs` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_menu_get_directory_dirs` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_menu_get_name` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_menuname_get_values` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_merge_file_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_merge_file_set_type` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_merge_get_type` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_move_get_new` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_move_get_old` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_new` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_prepend_child` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_root_add_entries_monitor` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_root_get_basedir` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_root_get_name` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_root_remove_entries_monitor` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_set_content` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_steal` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_unlink` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_layout_node_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_monitor_add_notify` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_monitor_ref` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_monitor_remove_notify` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_monitor_unref` | функция |  | подсказка: статус из легенды |
| `libmenu::menu_verbose` | функция |  | подсказка: статус из легенды |
