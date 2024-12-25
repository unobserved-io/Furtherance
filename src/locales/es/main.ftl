### Localization resource for Furtherance (Spanish)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Óscar Fernández Díaz <oscfdezdz@tuta.io>, 2022
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Guardar copia de seguridad de Furtherance
sqlite-database = Base de datos SQLite
sqlite-files = Archivos SQLite
backup-successful = Copia de seguridad de la base de datos exitosa
save-csv-title = Guardar CSV de Furtherance
open-csv-title = Abrir CSV de Furtherance
new-database-title = Nueva base de datos de Furtherance
open-database-title = Abrir base de datos de Furtherance

## General UI
shortcuts = Atajos
timer = Temporizador
history = Historial
report = Informe
settings = Configuración
today = Hoy
yesterday = Ayer
cancel = Cancelar
save = Guardar
delete = Borrar
edit = Editar
ok = Aceptar
stop = Detener
continue = Continuar
discard = Descartar

## Timer
task-input-placeholder = Nombre de la tarea @Proyecto #etiquetas $tarifa
started-at = Iniciado a las {$time}
recorded-today = Registrado hoy: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} tarea
    *[other] {$count} tareas
}
total-time-dynamic = Total: {$time}
total-earnings = ${$amount}
repeat = Repetir

## Shortcuts
new-shortcut = Nuevo atajo
create-shortcut = Crear atajo

## Reports
charts = Gráficos
list = Lista
breakdown-by-selection = Desglose por selección
total-time = Tiempo total
earned = Ganado
past-week = Última semana
past-thirty-days = Últimos 30 días
past-six-months = Últimos 6 meses
all-time = Todo el tiempo
date-range = Rango de fechas
title = Título
tags = Etiquetas
rate = Tarifa
none = Ninguno
no-tags = sin etiquetas

## Settings
general = General
advanced = Avanzado
pomodoro = Pomodoro
data = Datos

### General Settings
interface = Interfaz
default-view = Vista predeterminada
show-delete-confirmation = Mostrar confirmación de borrado
task-history = Historial de tareas
show-project = Mostrar proyecto
show-tags = Mostrar etiquetas
show-earnings = Mostrar ganancias
show-seconds = Mostrar segundos
show-daily-time-total = Mostrar total de tiempo diario

### Advanced Settings
idle = Inactivo
idle-detection = Detección de inactividad
minutes-until-idle = Minutos hasta inactividad
dynamic-total = Total dinámico
dynamic-total-description = El tiempo total de hoy aumenta con el temporizador
days-to-show = Días a mostrar

### Pomodoro Settings
pomodoro-timer = Temporizador Pomodoro
notification_alarm_sound = Sonido de alarma de notificación
countdown-timer = Temporizador de cuenta regresiva
timer-length = Duración del temporizador
break-length = Duración del descanso
snooze-length = Duración de la pausa
extended-break = Descanso extendido
extended-breaks = Descansos extendidos
extended-break-interval = Intervalo de descanso extendido
extended-break-length = Duración del descanso extendido

### Report Settings
toggle-charts = Alternar gráficos
total-time-box = Casilla de tiempo total
total-earnings-box = Casilla de ganancias totales
time-recorded = Tiempo registrado
earnings = Ganancias
average-time-per-task = Tiempo promedio por tarea
average-earnings-per-task = Ganancias promedio por tarea
breakdown-by-selection-section = Sección de desglose por selección
time-recorded-for-selection = Tiempo registrado para la selección
earnings-for-selection = Ganancias para la selección

### Data Settings
local-database = Base de datos local
database-location = Ubicación de la base de datos
create-new = Crear nuevo
open-existing = Abrir existente
export-csv = Exportar CSV
import-csv = Importar CSV
backup = Copia de seguridad
backup-database = Hacer copia de seguridad de la base de datos

## Inspector
task-name = Nombre de la tarea
project = Proyecto
hashtag-tags = #etiquetas
start-colon = Inicio:
stop-colon = Fin:
per-hour = /hr
color = Color
edit-shortcut = Editar atajo
start-to-stop = {$start} a {$stop}
nothing-selected = Nada seleccionado.

## Charts
average-earnings-per-task-title = Ganancias promedio por tarea
average-time-per-task-title = Tiempo promedio por tarea
time-recorded-title = Tiempo registrado
time-recorded-for-selection-title = Tiempo registrado para la selección
earnings-for-selection-title = Ganancias para la selección

## Alerts
delete-all = Borrar todo
delete-all-question = ¿Borrar todo?
delete-all-description = ¿Está seguro de que desea borrar permanentemente todas las tareas de este grupo?
delete-shortcut-question = ¿Borrar atajo?
delete-shortcut-description = ¿Está seguro de que desea borrar este atajo?
delete-task-question = ¿Borrar tarea?
delete-task-description = ¿Está seguro de que desea borrar permanentemente esta tarea?
idle-alert-title = Ha estado inactivo durante {$duration}
idle-alert-description = ¿Desea descartar ese tiempo o continuar el reloj?
break-over-title = ¡Se acabó el descanso!
break-over-description = Es hora de volver al trabajo.
pomodoro-over-title = ¡Se acabó el tiempo!
pomodoro-over-description = ¿Está listo para tomar un descanso?
snooze-button = {$duration} {$duration ->
    [one] minuto
    *[other] minutos
} más
long-break = Descanso largo
break = Descanso
shortcut-exists = El atajo ya existe
shortcut-exists-description = Ya existe un atajo para esa tarea.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} h
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = Es hora de tomar un descanso.
idle-notification-title = Ha estado inactivo.
idle-notification-body = Abra Furtherance para continuar o descartar el tiempo inactivo.

## Errors
invalid-database = Base de datos no válida.
error-upgrading-database = Error al actualizar la base de datos heredada.
error-accessing-database = Error al acceder a la nueva base de datos.
database-loaded = Base de datos cargada.
database-created = Base de datos creada.
csv-file-saved = Archivo CSV guardado.
error-writing-csv = Error al escribir datos en CSV.
csv-imported = CSV importado con éxito
invalid-csv-file = Archivo CSV no válido
error-retrieving-tasks = Error al recuperar tareas de la base de datos
error-creating-file = Error al crear el archivo
error-reading-headers = Error al leer los encabezados
wrong-column-order = Orden de columnas incorrecto.
missing-column = Columna faltante
invalid-csv = CSV no válido
backup-database-failed = Error al hacer copia de seguridad de la base de datos
name-cannot-contain = El nombre de la tarea no puede contener #, @, o $.
project-cannot-contain = El proyecto no puede contener #, @, o $.
tags-cannot-contain = Las etiquetas no pueden contener @ o $.
tags-must-start = Las etiquetas deben comenzar con #.
no-symbol-in-rate = No incluya $ en la tarifa.
rate-invalid = La tarifa debe ser una cantidad válida en dólares.
