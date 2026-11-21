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
> **Внешний референс (COSMIC):** частично: `sysfs-class` (1 120 строк) покинут и не развивается с 2021 года, метрики процесса — `sysinfo`; читаем `/sys/class/hwmon` напрямую (ТЗ 9.24). Проверено по 136 ebuild `fsvm88/cosmic-overlay`
> и 345 репозиториям org `pop-os` ([`sravnenie.md`](../analysis/cosmic/sravnenie.md), строка 24). Позиции ниже портируются по
> upstream; сверять «как у COSMIC» не с чем.

Статусы: `нет` / `в работе` / `готово` / `отложено (причина)`. `MVP` — обязательно к закрытию
компонента; «готово» для MVP-позиции означает зелёный smoke-сценарий (ТЗ 6.9), а не мнение
исполнителя.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
