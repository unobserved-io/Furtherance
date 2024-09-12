### Localization resource for Furtherance (German)
### Last updated: 2024-09-11

## File dialogs
save-backup-title = Sicherungsdatenbank speichern
sqlite-database = SQLite-Datenbank
sqlite-files = SQLite-Dateien
backup-successful = Datenbank-Sicherung erfolgreich
save-csv-title = Furtherance CSV speichern
open-csv-title = Furtherance CSV öffnen
new-database-title = Neue Furtherance-Datenbank
open-database-title = Furtherance-Datenbank öffnen

## General UI
shortcuts = Verknüpfungen
timer = Timer
history = Chronik
report = Bericht
settings = Einstellungen
today = Heute
yesterday = Gestern
cancel = Abbrechen
save = Speichern
delete = Löschen
edit = Bearbeiten
ok = OK
stop = Stopp
continue = Fortsetzen
discard = Verwerfen

## Timer
task-input-placeholder = Aufgabenname @Projekt #Tags $Tarif
started-at = Gestartet um {$time}
recorded-today = Heute aufgezeichnet: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} Aufgabe
    *[other] {$count} Aufgaben
}
total-time-dynamic = Gesamt: {$time}
total-earnings = ${$amount}
repeat = Wiederholen

## Shortcuts
new-shortcut = Neue Verknüpfung
create-shortcut = Verknüpfung erstellen

## Reports
charts = Diagramme
list = Liste
breakdown-by-selection = Aufschlüsselung nach Auswahl
total-time = Gesamtzeit
earned = Verdient
past-week = Letzte Woche
past-thirty-days = Letzte 30 Tage
past-six-months = Letzte 180 Tage
all-time = Gesamte Zeit
date-range = Datumsbereich
title = Titel
tags = Tags
rate = Tarif
none = Keine
no-tags = Keine Tags

## Settings
general = Allgemein
advanced = Erweitert
pomodoro = Pomodoro
data = Daten

### General Settings
interface = Oberfläche
default-view = Standardansicht
show-delete-confirmation = Löschen bestätigen
task-history = Aufgabenchronik
show-project = Projekt anzeigen
show-tags = Tags anzeigen
show-earnings = Verdienst anzeigen
show-seconds = Sekunden anzeigen
show-daily-time-total = Tägliche Gesamtzeit anzeigen

### Advanced Settings
idle = Inaktivität
idle-detection = Inaktivitätserkennung
minutes-until-idle = Minuten bis zur Inaktivität
dynamic-total = Dynamische Gesamtzeit
dynamic-total-description = Heutige Gesamtzeit beinhaltet den laufenden Timer
days-to-show = Anzuzeigende Tage

### Pomodoro Settings
pomodoro-timer = Pomodoro-Timer
countdown-timer = Countdown-Timer
timer-length = Timer-Länge
break-length = Pausenlänge
snooze-length = Schlummerlänge
extended-break = Verlängerte Pause
extended-breaks = Verlängerte Pausen
extended-break-interval = Intervall für verlängerte Pausen
extended-break-length = Länge der verlängerten Pause

### Report Settings
toggle-charts = Diagramme umschalten
total-time-box = Gesamtzeitbox
total-earnings-box = Gesamtverdienst-Box
time-recorded = Aufgezeichnete Zeit
earnings = Verdienst
average-time-per-task = Durchschnittliche Zeit pro Aufgabe
average-earnings-per-task = Durchschnittlicher Verdienst pro Aufgabe
breakdown-by-selection-section = Aufschlüsselung nach Auswahl-Bereich
time-recorded-for-selection = Aufgezeichnete Zeit für Auswahl
earnings-for-selection = Verdienst für Auswahl

### Data Settings
local-database = Lokale Datenbank
database-location = Speicherort der Datenbank
create-new = Neu erstellen
open-existing = Bestehende öffnen
export-csv = Als CSV exportieren
import-csv = CSV importieren
backup = Sichern
backup-database = Datenbank sichern

## Inspector
task-name = Aufgabenname
project = Projekt
hashtag-tags = #Tags
start-colon = Start:
stop-colon = Stopp:
per-hour = /Std
color = Farbe
edit-shortcut = Verknüpfung bearbeiten
start-to-stop = {$start} bis {$stop}
nothing-selected = Nichts ausgewählt.

## Charts
average-earnings-per-task-title = Durchschnittlicher Verdienst pro Aufgabe
average-time-per-task-title = Durchschnittliche Zeit pro Aufgabe
time-recorded-title = Aufgezeichnete Zeit
time-recorded-for-selection-title = Aufgezeichnete Zeit für Auswahl
earnings-for-selection-title = Verdienst für Auswahl

## Alerts
delete-all = Alle löschen
delete-all-question = Alle löschen?
delete-all-description = Möchten Sie wirklich alle Aufgaben in dieser Gruppe dauerhaft löschen?
delete-shortcut-question = Verknüpfung löschen?
delete-shortcut-description = Möchten Sie diese Verknüpfung wirklich löschen?
delete-task-question = Aufgabe löschen?
delete-task-description = Möchten Sie diese Aufgabe wirklich dauerhaft löschen?
idle-alert-title = Sie waren inaktiv für {$duration}
idle-alert-description = Möchten Sie diese Zeit verwerfen oder die Uhr fortsetzen?
break-over-title = Die Pause ist vorbei!
break-over-description = Zeit, wieder an die Arbeit zu gehen.
pomodoro-over-title = Die Zeit ist um!
pomodoro-over-description = Sind Sie bereit, eine Pause zu machen?
snooze-button = {$duration} weitere {$duration ->
    [one] Minute
    *[other] Minuten
}
long-break = Lange Pause
break = Pause
shortcut-exists = Verknüpfung existiert bereits
shortcut-exists-description = Eine Verknüpfung für diese Aufgabe existiert bereits.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} Std
x-m = {$minutes} Min
x-s = {$seconds} Sek

## Notifications
pomodoro-over-notification-body = Es ist Zeit, eine Pause zu machen.
idle-notification-title = Sie waren inaktiv.
idle-notification-body = Öffnen Sie Furtherance, um fortzufahren oder die Leerlaufzeit zu verwerfen.

## Errors
invalid-database = Ungültige Datenbank.
error-upgrading-database = Fehler beim Aktualisieren der alten Datenbank.
error-accessing-database = Fehler beim Zugriff auf die neue Datenbank.
database-loaded = Datenbank geladen.
database-created = Datenbank erstellt.
csv-file-saved = CSV-Datei gespeichert.
error-writing-csv = Fehler beim Schreiben der Daten in CSV.
csv-imported = CSV erfolgreich importiert
invalid-csv-file = Ungültige CSV-Datei
error-retrieving-tasks = Fehler beim Abrufen der Aufgaben aus der Datenbank
error-creating-file = Fehler beim Erstellen der Datei
error-reading-headers = Fehler beim Lesen der Kopfzeilen
wrong-column-order = Falsche Spaltenreihenfolge.
missing-column = Fehlende Spalte
invalid-csv = Ungültige CSV
backup-database-failed = Sicherung der Datenbank fehlgeschlagen
name-cannot-contain = Aufgabenname darf keine #, @ oder $ enthalten.
project-cannot-contain = Projekt darf keine #, @ oder $ enthalten.
tags-cannot-contain = Tags dürfen keine @ oder $ enthalten.
tags-must-start = Tags müssen mit einem # beginnen.
no-symbol-in-rate = Verwenden Sie kein $ im Tarif.
rate-invalid = Der Tarif muss ein gültiger Dollarbetrag sein.
