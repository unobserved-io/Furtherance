### Localization resource for Furtherance (Dutch)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Heimen Stoffels <vistausss@fastmail.com>, 2022
### and Philip Goto <philip.goto@gmail.com>, 2024
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Furtherance-reservekopie opslaan
sqlite-database = SQLite-database
sqlite-files = SQLite-bestanden
backup-successful = Database-reservekopie gemaakt
save-csv-title = Furtherance-csv opslaan
open-csv-title = Furtherance-csv openen
new-database-title = Nieuwe Furtherance-database
open-database-title = Furtherance-database openen

## General UI
shortcuts = Sneltoetsen
timer = Tijdklok
history = Geschiedenis
report = Verslag
settings = Voorkeuren
today = Vandaag
yesterday = Gisteren
cancel = Annuleren
save = Opslaan
delete = Wissen
edit = Bewerken
ok = Oké
stop = Stoppen
continue = Doorgaan
discard = Wissen

## Timer
task-input-placeholder = Taaknaam @Project #labels $tarief
started-at = Gestart om {$time}
recorded-today = Vandaag genoteerd: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} taak
    *[other] {$count} taken
}
total-time-dynamic = Totaal: {$time}
total-earnings = ${$amount}
repeat = Herhalen

## Shortcuts
new-shortcut = Nieuwe sneltoets
create-shortcut = Sneltoets aanmaken

## Reports
charts = Grafieken
list = Lijst
breakdown-by-selection = Uitsplitsing per selectie
total-time = Totale tijd
earned = Verdiend
past-week = Afgelopen week
past-thirty-days = Afgelopen 30 dagen
past-six-months = Afgelopen 180 dagen
all-time = Alle tijd
date-range = Datumbereik
title = Titel
tags = Labels
rate = Tarief
none = Geen
no-tags = geen labels

## Settings
general = Algemeen
advanced = Geavanceerd
pomodoro = Pomodoro
data = Gegevens

### General Settings
interface = Interface
default-view = Standaardweergave
show-delete-confirmation = Om bevestiging vragen alvorens te wissen
task-history = Taakgeschiedenis
show-project = Project tonen
show-tags = Labels tonen
show-earnings = Inkomsten tonen
show-seconds = Seconden tonen
show-daily-time-total = Dagelijks overzicht tonen

### Advanced Settings
idle = Inactief
idle-detection = Inactiviteitsdetectie
minutes-until-idle = Inactiviteit (in minuten)
dynamic-total = Dynamisch totaal
dynamic-total-description = Het totaal van vandaag tikt op met de tijdklok
days-to-show = Te tonen dagen

### Pomodoro Settings
pomodoro-timer = Pomodoro-tijdklok
notification-alarm-sound = Meldingsalarmgeluid
countdown-timer = Aftelklok
timer-length = Tijdklokduur
break-length = Pauzeduur
snooze-length = Snoozeduur
extended-break = Verlengde pauze
extended-breaks = Verlengde pauzes
extended-break-interval = Interval verlengde pauze
extended-break-length = Duur verlengde pauze

### Report Settings
toggle-charts = Grafieken in-/uitschakelen
total-time-box = Totale-tijdvak
total-earnings-box = Totale-inkomstenvak
time-recorded = Genoteerde tijd
earnings = Inkomsten
average-time-per-task = Gemiddelde tijd per taak
average-earnings-per-task = Gemiddelde inkomsten per taak
breakdown-by-selection-section = Uitsplitsing per selectiesectie
time-recorded-for-selection = Genoteerde tijd voor selectie
earnings-for-selection = Inkomsten voor selectie

### Data Settings
local-database = Lokale database
database-location = Databaselocatie
create-new = Nieuwe aanmaken
open-existing = Bestaande openen
export-csv = Exporteren naar csv-bestand
import-csv = Csv importeren
backup = Reservekopie maken
backup-database = Database-reservekopie maken

## Inspector
task-name = Taaknaam
project = Project
hashtag-tags = #labels
start-colon = Begin:
stop-colon = Einde:
per-hour = /uur
color = Kleur
edit-shortcut = Sneltoets bewerken
start-to-stop = {$start} tot {$stop}
nothing-selected = Niets geselecteerd.

## Charts
average-earnings-per-task-title = Gemiddelde inkomsten per taak
average-time-per-task-title = Gemiddelde tijd per taak
time-recorded-title = Genoteerde tijd
time-recorded-for-selection-title = Genoteerde tijd voor selectie
earnings-for-selection-title = Inkomsten voor selectie

## Alerts
delete-all = Alles wissen
delete-all-question = Weet u zeker dat u alles wilt wissen?
delete-all-description = Hierdoor worden alle taken in deze groep definitief gewist.
delete-shortcut-question = Sneltoets wissen?
delete-shortcut-description = Weet u zeker dat u deze sneltoets wilt wissen?
delete-task-question = Weet u zeker dat u deze taak wilt wissen?
delete-task-description = Weet u zeker dat u deze taak definitief wilt wissen?
idle-alert-title = U bent inactief: {$duration}
idle-alert-description = Wilt u deze tijd wissen of de klok verder laten tikken?
break-over-title = De pauze is voorbij!
break-over-description = Tijd om weer aan het werk te gaan.
pomodoro-over-title = De tijd is om!
pomodoro-over-description = Bent u klaar om een pauze te nemen?
snooze-button = {$duration} meer {$duration ->
    [one] minuut
    *[other] minuten
}
long-break = Lange pauze
break = Pauze
shortcut-exists = Sneltoets bestaat al
shortcut-exists-description = Er bestaat al een sneltoets voor die taak.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} u
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = Het is tijd om een pauze te nemen.
idle-notification-title = U bent inactief.
idle-notification-body = Open Furtherance om door te gaan of de inactieve tijd te wissen.

## Errors
invalid-database = Ongeldige database.
error-upgrading-database = Fout bij het upgraden van de oude database.
error-accessing-database = Fout bij het openen van de nieuwe database.
database-loaded = Database geladen.
database-created = Database aangemaakt.
csv-file-saved = Csv-bestand opgeslagen.
error-writing-csv = Fout bij het schrijven van gegevens naar csv.
csv-imported = Csv succesvol geïmporteerd
invalid-csv-file = Ongeldig csv-bestand
error-retrieving-tasks = Kan taken niet ophalen uit de database
error-creating-file = Kan het bestand niet aanmaken
error-reading-headers = Kan de headers niet lezen
wrong-column-order = Onjuiste kolomvolgorde.
missing-column = Ontbrekende kolom
invalid-csv = Ongeldig csv-bestand
backup-database-failed = Maken van database-reservekopie mislukt
name-cannot-contain = Taaknaam mag geen #, @ of $ bevatten.
project-cannot-contain = Project mag geen #, @ of $ bevatten.
tags-cannot-contain = Labels mogen geen @ of $ bevatten.
tags-must-start = Labels moeten beginnen met #.
no-symbol-in-rate = Gebruik geen $ in het tarief.
rate-invalid = Tarief moet een geldig bedrag in dollars zijn.
