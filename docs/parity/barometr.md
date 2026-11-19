# Парити-лист `barometr` ← `mate-sensors-applet` (М4)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-sensors-applet` · HEAD `65e34e2` · модулей (`.c`): 26

| Каталог | Модули |
|---|---|
| `lib` | `sensors-applet-plugin.c` |
| `plugins/acpi` | `acpi-plugin.c` |
| `plugins/dummy` | `dummy-plugin.c` |
| `plugins/hddtemp` | `hddtemp-plugin.c` |
| `plugins/i2c-proc` | `i2c-proc-plugin.c` |
| `plugins/i2c-sys` | `i2c-sys-plugin.c` |
| `plugins/i8k` | `i8k-plugin.c` |
| `plugins/ibm-acpi` | `ibm-acpi-plugin.c` |
| `plugins/libsensors` | `libsensors-plugin.c` |
| `plugins/mbmon` | `mbmon-plugin.c` |
| `plugins/netbsd` | `netbsd-plugin.c` |
| `plugins/nvidia` | `nvidia-plugin.c` |
| `plugins/omnibook` | `omnibook-plugin.c` |
| `plugins/pmu-sys` | `pmu-sys-plugin.c` |
| `plugins/smu-sys` | `smu-sys-plugin.c` |
| `plugins/sonypi` | `sonypi-plugin.c` |
| `plugins/udisks2` | `udisks2-plugin.c` |
| `sensors-applet` | `about-dialog.c`, `active-sensor-libnotify.c`, `active-sensor.c`, `main.c`, `prefs-dialog.c`, `sensor-config-dialog.c`, `sensors-applet-plugins.c`, `sensors-applet-settings.c`, `sensors-applet.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|