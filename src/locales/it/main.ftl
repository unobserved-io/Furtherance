### Localization resource for Furtherance (Italian)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Musiclover <Musiclover382@protonmail.com>, 2022
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Salva backup di Furtherance
sqlite-database = Database SQLite
sqlite-files = File SQLite
backup-successful = Backup del database completato con successo
save-csv-title = Salva CSV di Furtherance
open-csv-title = Apri CSV di Furtherance
new-database-title = Nuovo database di Furtherance
open-database-title = Apri database di Furtherance

## General UI
shortcuts = Scorciatoie
timer = Timer
history = Cronologia
report = Resoconto
settings = Preferenze
today = Oggi
yesterday = Ieri
cancel = Annulla
save = Salva
delete = Elimina
edit = Modifica
ok = OK
stop = Fine
continue = Continua
discard = Scarta

## Timer
task-input-placeholder = Nome attività #Tag
started-at = Iniziato alle {$time}
recorded-today = Registrato oggi: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} attività
    *[other] {$count} attività
}
total-time-dynamic = Totale: {$time}
total-earnings = ${$amount}
repeat = Ripeti

## Shortcuts
new-shortcut = Nuova scorciatoia
create-shortcut = Crea scorciatoia

## Reports
charts = Grafici
list = Lista
breakdown-by-selection = Suddivisione per selezione
total-time = Tempo totale
earned = Guadagnato
past-week = Ultima settimana
past-thirty-days = Ultimi 30 giorni
past-six-months = Ultimi 180 giorni
all-time = Tutto il tempo
date-range = Personalizzato
title = Titolo
tags = Tag
rate = Tariffa
none = Nessuno
no-tags = Nessun tag

## Settings
general = Generali
advanced = Avanzate
pomodoro = Pomodoro
data = Dati

### General Settings
interface = Interfaccia
default-view = Vista predefinita
show-delete-confirmation = Conferma eliminazioni
task-history = Cronologia attività
show-project = Mostra progetto
show-tags = Mostra tag
show-earnings = Mostra guadagni
show-seconds = Mostra secondi
show-daily-time-total = Mostra somme giornaliere

### Advanced Settings
idle = Inattività
idle-detection = Rilevamento di inattività
minutes-until-idle = Minuti per l'inattività
dynamic-total = Totale dinamico
dynamic-total-description = Il tempo totale di oggi aumenta con il timer
days-to-show = Giorni da mostrare

### Pomodoro Settings
pomodoro-timer = Timer Pomodoro
notification-alarm-sound = Suono di allarme di notifica
countdown-timer = Timer conto alla rovescia
timer-length = Durata timer
break-length = Durata pausa
snooze-length = Durata posticipo
extended-break = Pausa estesa
extended-breaks = Pause estese
extended-break-interval = Intervallo pausa estesa
extended-break-length = Durata pausa estesa

### Report Settings
toggle-charts = Attiva/disattiva grafici
total-time-box = Casella tempo totale
total-earnings-box = Casella guadagni totali
time-recorded = Tempo registrato
earnings = Guadagni
average-time-per-task = Tempo medio per attività
average-earnings-per-task = Guadagno medio per attività
breakdown-by-selection-section = Sezione suddivisione per selezione
time-recorded-for-selection = Tempo registrato per selezione
earnings-for-selection = Guadagni per selezione

### Data Settings
local-database = Database locale
database-location = Posizione database
create-new = Crea nuovo
open-existing = Apri esistente
export-csv = Esporta come CSV
import-csv = Importa CSV
backup = Esegui backup
backup-database = Backup del database

## Inspector
task-name = Nome attività
project = Progetto
hashtag-tags = #Tag
start-colon = Inizio:
stop-colon = Fine:
per-hour = /ora
color = Colore
edit-shortcut = Modifica scorciatoia
start-to-stop = {$start} a {$stop}
nothing-selected = Nessuna selezione.

## Charts
average-earnings-per-task-title = Guadagno medio per attività
average-time-per-task-title = Tempo medio per attività
time-recorded-title = Tempo registrato
time-recorded-for-selection-title = Tempo registrato per selezione
earnings-for-selection-title = Guadagni per selezione

## Alerts
delete-all = Elimina tutto
delete-all-question = Eliminare tutto?
delete-all-description = Verranno eliminate tutte le attività con questo nome in questo giorno.
delete-shortcut-question = Eliminare la scorciatoia?
delete-shortcut-description = Sei sicuro di voler eliminare questa scorciatoia?
delete-task-question = Eliminare l'attività?
delete-task-description = Sei sicuro di voler eliminare questa attività?
idle-alert-title = Sei stato inattivo per {$duration}
idle-alert-description = Vuoi scartare tale lasso di tempo o continuare regolarmente?
break-over-title = La pausa è finita!
break-over-description = È ora di tornare al lavoro.
pomodoro-over-title = Tempo scaduto!
pomodoro-over-description = Sei pronto per una pausa?
snooze-button = {$duration} in più {$duration ->
    [one] minuto
    *[other] minuti
}
long-break = Pausa lunga
break = Pausa
shortcut-exists = La scorciatoia esiste già
shortcut-exists-description = Esiste già una scorciatoia per questa attività.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} o
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = È ora di fare una pausa.
idle-notification-title = Sei stato inattivo.
idle-notification-body = Apri Furtherance per continuare o scartare il tempo di inattività.

## Errors
invalid-database = Database non valido.
error-upgrading-database = Errore durante l'aggiornamento del database legacy.
error-accessing-database = Errore durante l'accesso al nuovo database.
database-loaded = Database caricato.
database-created = Database creato.
csv-file-saved = File CSV salvato.
error-writing-csv = Errore durante la scrittura dei dati in CSV.
csv-imported = CSV importato con successo
invalid-csv-file = File CSV non valido
error-retrieving-tasks = Impossibile recuperare le attività dal database
error-creating-file = Impossibile creare il file
error-reading-headers = Impossibile leggere le intestazioni
wrong-column-order = Ordine delle colonne errato.
missing-column = Colonna mancante
invalid-csv = CSV non valido
backup-database-failed = Backup del database fallito
name-cannot-contain = Il nome dell'attività non può contenere #, @ o $.
project-cannot-contain = Il progetto non può contenere #, @ o $.
tags-cannot-contain = I tag non possono contenere @ o $.
tags-must-start = I tag devono iniziare con #.
no-symbol-in-rate = Non includere $ nella tariffa.
rate-invalid = La tariffa deve essere un importo in dollari valido.
