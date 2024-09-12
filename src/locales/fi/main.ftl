### Localization resource for Furtherance (Finnish)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Jiri Grönroos <jiri.gronroos@iki.fi>, 2023
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Tallenna Furtherance-varmuuskopio
sqlite-database = SQLite-tietokanta
sqlite-files = SQLite-tiedostot
backup-successful = Tietokannan varmuuskopiointi onnistui
save-csv-title = Tallenna Furtherance CSV
open-csv-title = Avaa Furtherance CSV
new-database-title = Uusi Furtherance-tietokanta
open-database-title = Avaa Furtherance-tietokanta

## General UI
shortcuts = Pikanäppäimet
timer = Ajastin
history = Historia
report = Raportti
settings = Asetukset
today = Tänään
yesterday = Eilen
cancel = Peru
save = Tallenna
delete = Poista
edit = Muokkaa
ok = OK
stop = Loppu
continue = Jatka
discard = Hylkää

## Timer
task-input-placeholder = Tehtävän nimi #tunnisteet
started-at = Aloitettu {$time}
recorded-today = Tallennettu tänään: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} tehtävä
    *[other] {$count} tehtävää
}
total-time-dynamic = Yhteensä: {$time}
total-earnings = ${$amount}
repeat = Toista

## Shortcuts
new-shortcut = Uusi pikakuvake
create-shortcut = Luo pikakuvake

## Reports
charts = Kaaviot
list = Lista
breakdown-by-selection = Erittely valinnan mukaan
total-time = Kokonaisaika
earned = Ansaittu
past-week = Mennyt viikko
past-thirty-days = Viimeiset 30 päivää
past-six-months = Viimeiset 180 päivää
all-time = Koko aika
date-range = Aikaväli
title = Otsikko
tags = Tunnisteet
rate = Tuntihinta
none = Ei mitään
no-tags = ei tunnisteita

## Settings
general = Yleiset
advanced = Lisäasetukset
pomodoro = Pomodoro
data = Tiedot

### General Settings
interface = Käyttöliittymä
default-view = Oletusnäkymä
show-delete-confirmation = Poistovahvistus
task-history = Tehtävähistoria
show-project = Näytä projekti
show-tags = Näytä tunnisteet
show-earnings = Näytä ansiot
show-seconds = Näytä sekunnit
show-daily-time-total = Näytä päivittäinen kokonaisaika

### Advanced Settings
idle = Joutenolo
idle-detection = Joutenolon tunnistus
minutes-until-idle = Minuutteja joutenoloon
dynamic-total = Dynaaminen kokonaisaika
dynamic-total-description = Päivän kokonaisaika päivittyy ajastimen mukana
days-to-show = Näytettävät päivät

### Pomodoro Settings
pomodoro-timer = Pomodoro-ajastin
countdown-timer = Lähtölaskenta-ajastin
timer-length = Ajastimen pituus
break-length = Tauon pituus
snooze-length = Torkkuajan pituus
extended-break = Pidennetty tauko
extended-breaks = Pidennetyt tauot
extended-break-interval = Pidennetyn tauon aikaväli
extended-break-length = Pidennetyn tauon pituus

### Report Settings
toggle-charts = Näytä/piilota kaaviot
total-time-box = Kokonaisaikalaatikko
total-earnings-box = Kokonaisansiolaatikko
time-recorded = Tallennettu aika
earnings = Ansiot
average-time-per-task = Keskimääräinen aika per tehtävä
average-earnings-per-task = Keskimääräiset ansiot per tehtävä
breakdown-by-selection-section = Erittely valinnan mukaan -osio
time-recorded-for-selection = Tallennettu aika valinnalle
earnings-for-selection = Ansiot valinnalle

### Data Settings
local-database = Paikallinen tietokanta
database-location = Tietokannan sijainti
create-new = Luo uusi
open-existing = Avaa olemassa oleva
export-csv = Vie CSV-muotoon
import-csv = Tuo CSV
backup = Varmuuskopioi
backup-database = Varmuuskopioi tietokanta

## Inspector
task-name = Tehtävän nimi
project = Projekti
hashtag-tags = #tunnisteet
start-colon = Alku:
stop-colon = Loppu:
per-hour = /tunti
color = Väri
edit-shortcut = Muokkaa pikakuvaketta
start-to-stop = {$start} - {$stop}
nothing-selected = Ei valintaa.

## Charts
average-earnings-per-task-title = Keskimääräiset ansiot per tehtävä
average-time-per-task-title = Keskimääräinen aika per tehtävä
time-recorded-title = Tallennettu aika
time-recorded-for-selection-title = Tallennettu aika valinnalle
earnings-for-selection-title = Ansiot valinnalle

## Alerts
delete-all = Poista kaikki
delete-all-question = Poistetaanko kaikki?
delete-all-description = Haluatko varmasti poistaa pysyvästi kaikki tämän ryhmän tehtävät?
delete-shortcut-question = Poistetaanko pikakuvake?
delete-shortcut-description = Haluatko varmasti poistaa tämän pikakuvakkeen?
delete-task-question = Poistetaanko tehtävä?
delete-task-description = Haluatko varmasti poistaa tämän tehtävän pysyvästi?
idle-alert-title = Olet ollut jouten {$duration}
idle-alert-description = Haluatko hylätä kyseisen ajan, vai jatkaa kellon kanssa?
break-over-title = Tauko on ohi!
break-over-description = Aika palata työhön.
pomodoro-over-title = Aika loppui!
pomodoro-over-description = Oletko valmis pitämään tauon?
snooze-button = {$duration} lisää {$duration ->
    [one] minuutti
    *[other] minuuttia
}
long-break = Pitkä tauko
break = Tauko
shortcut-exists = Pikakuvake on jo olemassa
shortcut-exists-description = Tälle tehtävälle on jo olemassa pikakuvake.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} t
x-m = {$minutes} min
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = On aika pitää tauko.
idle-notification-title = Olet ollut jouten.
idle-notification-body = Avaa Furtherance jatkaaksesi tai hylätäksesi joutenoloajan.

## Errors
invalid-database = Virheellinen tietokanta.
error-upgrading-database = Virhe päivitettäessä vanhaa tietokantaa.
error-accessing-database = Virhe käytettäessä uutta tietokantaa.
database-loaded = Tietokanta ladattu.
database-created = Tietokanta luotu.
csv-file-saved = CSV-tiedosto tallennettu.
error-writing-csv = Virhe kirjoitettaessa tietoja CSV-muotoon.
csv-imported = CSV tuotu onnistuneesti
invalid-csv-file = Virheellinen CSV-tiedosto
error-retrieving-tasks = Tehtävien haku tietokannasta epäonnistui
error-creating-file = Tiedoston luonti epäonnistui
error-reading-headers = Otsikoiden luku epäonnistui
wrong-column-order = Väärä sarakejärjestys.
missing-column = Puuttuva sarake
invalid-csv = Virheellinen CSV
backup-database-failed = Tietokannan varmuuskopiointi epäonnistui
name-cannot-contain = Tehtävän nimi ei voi sisältää merkkejä #, @ tai $.
project-cannot-contain = Projekti ei voi sisältää merkkejä #, @ tai $.
tags-cannot-contain = Tunnisteet eivät voi sisältää merkkejä @ tai $.
tags-must-start = Tunnisteiden tulee alkaa #-merkillä.
no-symbol-in-rate = Älä sisällytä $-merkkiä tuntihintaan.
rate-invalid = Tuntihinnan tulee olla kelvollinen dollarimäärä.
