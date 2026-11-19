# Парити-лист `samodelki` ← `mate-applets` (М4)
<!-- BEGIN GENERATOR (xtask parity) — руками не править -->

Зеркало: `mate-desktop/mate-applets` · HEAD `e4b07e6` · модулей (`.c`): 57

| Каталог | Модули |
|---|---|
| `accessx-status/src` | `applet.c` |
| `battstat` | `acpi-freebsd.c`, `acpi-linux.c`, `battstat-preferences.c`, `battstat-upower.c`, `battstat_applet.c`, `power-management.c` |
| `battstat/apmlib` | `apmlib.c` |
| `charpick` | `charpick.c`, `properties.c` |
| `command/src` | `command.c`, `ma-command.c` |
| `cpufreq/src` | `cpufreq-applet.c`, `cpufreq-monitor-cpuinfo.c`, `cpufreq-monitor-factory.c`, `cpufreq-monitor-libcpufreq.c`, `cpufreq-monitor-sysfs.c`, `cpufreq-monitor.c`, `cpufreq-popup.c`, `cpufreq-prefs.c`, `cpufreq-selector.c`, `cpufreq-utils.c` |
| `cpufreq/src/cpufreq-selector` | `cpufreq-selector-factory.c`, `cpufreq-selector-libcpufreq.c`, `cpufreq-selector-service.c`, `cpufreq-selector-sysfs.c`, `cpufreq-selector.c`, `main.c` |
| `drivemount/src` | `drive-button.c`, `drive-list.c`, `drivemount.c` |
| `geyes/src` | `geyes.c`, `themes.c` |
| `invest-applet/invest` | `invest-applet-chart.c`, `invest-applet.c` |
| `mateweather/src` | `main.c`, `mateweather-about.c`, `mateweather-applet.c`, `mateweather-dialog.c`, `mateweather-pref.c` |
| `multiload/src` | `autoscaler.c`, `linux-proc.c`, `load-graph.c`, `main.c`, `netspeed.c`, `properties.c` |
| `netspeed/src` | `backend.c`, `netspeed-preferences.c`, `netspeed.c` |
| `stickynotes` | `stickynotes.c`, `stickynotes_applet.c`, `stickynotes_applet_callbacks.c`, `stickynotes_callbacks.c`, `util.c` |
| `timerapplet/src` | `timerapplet.c` |
| `trashapplet/src` | `trash-empty.c`, `trashapplet.c` |
<!-- END GENERATOR -->

## Вердикты (ручной блок — генератор его не трогает)
Статусы позиции: `нет` / `в работе` / `отложено (причина)`; `готово` — только после
первого зелёного smoke-прогона листа (ТЗ 6.9, Q83), до него этот статус красен.
`отложено` без причины в скобках — не статус, а обещание. Судьба листа («заменено
конструкцией», «переносится полностью») — поле `Судьба:`, а не статус позиции;
`MVP` — обязательно к закрытию компонента.

| Позиция | Тип | Статус | Пометка |
|---|---|---|---|
