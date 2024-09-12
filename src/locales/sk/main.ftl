### Localization resource for Furtherance (Slovak)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: MartinIIOT <https://github.com/MartinIIOT>, 2022
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Uložiť zálohu Furtherance
sqlite-database = SQLite databáza
sqlite-files = SQLite súbory
backup-successful = Záloha databázy úspešná
save-csv-title = Uložiť CSV Furtherance
open-csv-title = Otvoriť CSV Furtherance
new-database-title = Nová databáza Furtherance
open-database-title = Otvoriť databázu Furtherance

## General UI
shortcuts = Skratky
timer = Časovač
history = História
report = Správa
settings = Predvoľby
today = Dnes
yesterday = Včera
cancel = Zrušiť
save = Uložiť
delete = Odstrániť
edit = Upraviť
ok = OK
stop = Stop
continue = Ďalej
discard = Zahodiť

## Timer
task-input-placeholder = Názov úlohy #Tags
started-at = Začiatok o {$time}
recorded-today = Zaznamenané dnes: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} úloha
    *[other] {$count} úloh
}
total-time-dynamic = Spolu: {$time}
total-earnings = ${$amount}
repeat = Opakovať

## Shortcuts
new-shortcut = Nová skratka
create-shortcut = Vytvoriť skratku

## Reports
charts = Grafy
list = Zoznam
breakdown-by-selection = Rozpis podľa výberu
total-time = Celkový čas
earned = Zarobené
past-week = Minulý týždeň
past-thirty-days = Posledných 30 dní
past-six-months = Posledných 180 dní
all-time = Celkový čas
date-range = Rozsah dátumov
title = Názov
tags = Značky
rate = Sadzba
none = Žiadne
no-tags = žiadne značky

## Settings
general = Všeobecné
advanced = Pokročilé
pomodoro = Pomodoro
data = Dáta

### General Settings
interface = Rozhranie
default-view = Predvolené zobrazenie
show-delete-confirmation = Potvrdenie vymazania
task-history = História úloh
show-project = Zobraziť projekt
show-tags = Zobraziť značky
show-earnings = Zobraziť zárobky
show-seconds = Zobraziť sekundy
show-daily-time-total = Ukážte denné množstvá

### Advanced Settings
idle = Nečinnosť
idle-detection = Detekcia nečinnosti
minutes-until-idle = Minúty do nečinnosti
dynamic-total = Dynamický súčet
dynamic-total-description = Dnešný celkový čas sa zvyšuje s časovačom
days-to-show = Počet dní na zobrazenie

### Pomodoro Settings
pomodoro-timer = Pomodoro časovač
countdown-timer = Odpočítavanie
timer-length = Dĺžka časovača
break-length = Dĺžka prestávky
snooze-length = Dĺžka odloženia
extended-break = Predĺžená prestávka
extended-breaks = Predĺžené prestávky
extended-break-interval = Interval predĺženej prestávky
extended-break-length = Dĺžka predĺženej prestávky

### Report Settings
toggle-charts = Prepnúť grafy
total-time-box = Pole celkového času
total-earnings-box = Pole celkových zárobkov
time-recorded = Zaznamenaný čas
earnings = Zárobky
average-time-per-task = Priemerný čas na úlohu
average-earnings-per-task = Priemerné zárobky na úlohu
breakdown-by-selection-section = Sekcia rozpisu podľa výberu
time-recorded-for-selection = Zaznamenaný čas pre výber
earnings-for-selection = Zárobky pre výber

### Data Settings
local-database = Lokálna databáza
database-location = Umiestnenie databázy
create-new = Vytvoriť novú
open-existing = Otvoriť existujúcu
export-csv = Exportovať CSV
import-csv = Importovať CSV
backup = Záloha
backup-database = Zálohovať databázu

## Inspector
task-name = Názov úlohy
project = Projekt
hashtag-tags = #značky
start-colon = Štart:
stop-colon = Stop:
per-hour = /hodina
color = Farba
edit-shortcut = Upraviť skratku
start-to-stop = {$start} do {$stop}
nothing-selected = Nič nie je vybrané.

## Charts
average-earnings-per-task-title = Priemerné zárobky na úlohu
average-time-per-task-title = Priemerný čas na úlohu
time-recorded-title = Zaznamenaný čas
time-recorded-for-selection-title = Zaznamenaný čas pre výber
earnings-for-selection-title = Zárobky pre výber

## Alerts
delete-all = Vymazať všetko
delete-all-question = Vymazať všetko?
delete-all-description = Týmto sa odstránia všetky výskyty tejto úlohy v tento deň.
delete-shortcut-question = Odstrániť skratku?
delete-shortcut-description = Ste si istý, že chcete odstrániť túto skratku?
delete-task-question = Odstrániť úlohu?
delete-task-description = Ste si istý, že chcete natrvalo odstrániť túto úlohu?
idle-alert-title = Boli ste nečinní {$duration}
idle-alert-description = Chcete tento čas zahodiť alebo pokračovať v hodinách?
break-over-title = Prestávka skončila!
break-over-description = Čas vrátiť sa k práci.
pomodoro-over-title = Čas vypršal!
pomodoro-over-description = Ste pripravený na prestávku?
snooze-button = O {$duration} viac {$duration ->
    [one] minútu
    *[other] minút
}
long-break = Dlhá prestávka
break = Prestávka
shortcut-exists = Skratka už existuje
shortcut-exists-description = Skratka pre túto úlohu už existuje.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} h
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = Je čas na prestávku.
idle-notification-title = Boli ste nečinní.
idle-notification-body = Otvorte Furtherance pre pokračovanie alebo zahodenie času nečinnosti.

## Errors
invalid-database = Neplatná databáza.
error-upgrading-database = Chyba pri aktualizácii starej databázy.
error-accessing-database = Chyba pri prístupe k novej databáze.
database-loaded = Databáza načítaná.
database-created = Databáza vytvorená.
csv-file-saved = CSV súbor uložený.
error-writing-csv = Chyba pri zápise dát do CSV.
csv-imported = CSV úspešne importované
invalid-csv-file = Neplatný CSV súbor
error-retrieving-tasks = Nepodarilo sa načítať úlohy z databázy
error-creating-file = Nepodarilo sa vytvoriť súbor
error-reading-headers = Nepodarilo sa prečítať hlavičky
wrong-column-order = Nesprávne poradie stĺpcov.
missing-column = Chýbajúci stĺpec
invalid-csv = Neplatné CSV
backup-database-failed = Zálohovanie databázy zlyhalo
name-cannot-contain = Názov úlohy nemôže obsahovať #, @ alebo $.
project-cannot-contain = Projekt nemôže obsahovať #, @ alebo $.
tags-cannot-contain = Značky nemôžu obsahovať @ alebo $.
tags-must-start = Značky musia začínať #.
no-symbol-in-rate = Nezahŕňajte $ v sadzbe.
rate-invalid = Sadzba musí byť platná dolárová suma.
