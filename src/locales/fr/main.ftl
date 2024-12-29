### Localization resource for Furtherance (French)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Mathieu Heurtevin <mathieu.heurtevinruiz@protonmail.com>, 2022
### and J. Lavoie <j.lavoie@net-c.ca>, 2023
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Sauvegarder la sauvegarde de Furtherance
sqlite-database = Base de données SQLite
sqlite-files = Fichiers SQLite
backup-successful = Sauvegarde de la base de données réussie
save-csv-title = Sauvegarder le CSV Furtherance
open-csv-title = Ouvrir le CSV Furtherance
new-database-title = Nouvelle base de données Furtherance
open-database-title = Ouvrir une base de données Furtherance

## General UI
shortcuts = Raccourcis
timer = Minuteur
history = Historique
report = Rapport
settings = Paramètres
today = Aujourd'hui
yesterday = Hier
cancel = Annuler
save = Sauvegarder
delete = Supprimer
edit = Modifier
ok = OK
stop = Arrêter
continue = Continuer
discard = Annuler

## Timer
task-input-placeholder = Nom de la tâche #étiquette
started-at = Commencé à {$time}
recorded-today = Enregistré aujourd'hui : {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} tâche
    *[other] {$count} tâches
}
total-time-dynamic = Total : {$time}
total-earnings = ${$amount}
repeat = Répéter

## Shortcuts
new-shortcut = Nouveau raccourci
create-shortcut = Créer un raccourci

## Reports
charts = Graphiques
list = Liste
breakdown-by-selection = Répartition par sélection
total-time = Temps total
earned = Gagné
past-week = Semaine dernière
past-thirty-days = Les 30 derniers jours
past-six-months = Les 180 derniers jours
all-time = Tout le temps
date-range = Intervalle de temps
title = Titre
tags = Étiquettes
rate = Taux
none = Aucun
no-tags = aucune étiquette

## Settings
general = Général
advanced = Avancé
pomodoro = Pomodoro
data = Données

### General Settings
interface = Interface
default-view = Vue par défaut
show-delete-confirmation = Confirmation de supression
task-history = Historique des tâches
show-project = Afficher le projet
show-tags = Montrer les étiquettes
show-earnings = Afficher les gains
show-seconds = Afficher les secondes
show-daily-time-total = Afficher le temps total par jour

### Advanced Settings
idle = Inactivité
idle-detection = Détection d'inactivité
minutes-until-idle = Minutes avant inactivité
dynamic-total = Total dynamique
dynamic-total-description = Le temps total d'aujourd'hui augmente avec le minuteur
days-to-show = Jours à afficher

### Pomodoro Settings
pomodoro-timer = Minuteur Pomodoro
notification-alarm-sound = Son d'alarme de notification
countdown-timer = Compte à rebours
timer-length = Durée du minuteur
break-length = Durée de la pause
snooze-length = Durée du report
extended-break = Pause prolongée
extended-breaks = Pauses prolongées
extended-break-interval = Intervalle de pause prolongée
extended-break-length = Durée de la pause prolongée

### Report Settings
toggle-charts = Afficher/masquer les graphiques
total-time-box = Boîte de temps total
total-earnings-box = Boîte de gains totaux
time-recorded = Temps enregistré
earnings = Gains
average-time-per-task = Temps moyen par tâche
average-earnings-per-task = Gains moyens par tâche
breakdown-by-selection-section = Section de répartition par sélection
time-recorded-for-selection = Temps enregistré pour la sélection
earnings-for-selection = Gains pour la sélection

### Data Settings
local-database = Base de données locale
database-location = Emplacement de la base de données
create-new = Créer nouveau
open-existing = Ouvrir existant
export-csv = Exporter en CSV
import-csv = Importer CSV
backup = Sauvegarder
backup-database = Sauvegarder la base de données

## Inspector
task-name = Nom de la tâche
project = Projet
hashtag-tags = #étiquettes
start-colon = Début :
stop-colon = Fin :
per-hour = /heure
color = Couleur
edit-shortcut = Modifier le raccourci
start-to-stop = {$start} à {$stop}
nothing-selected = Rien de sélectionné.

## Charts
average-earnings-per-task-title = Gains moyens par tâche
average-time-per-task-title = Temps moyen par tâche
time-recorded-title = Temps enregistré
time-recorded-for-selection-title = Temps enregistré pour la sélection
earnings-for-selection-title = Gains pour la sélection

## Alerts
delete-all = Tout supprimer
delete-all-question = Tout supprimer ?
delete-all-description = Cela va supprimer tous les évènements de cette tâche pour ce jour.
delete-shortcut-question = Supprimer le raccourci ?
delete-shortcut-description = Voulez-vous vraiment supprimer ce raccourci ?
delete-task-question = Supprimer la tâche ?
delete-task-description = Voulez-vous vraiment supprimer cette tâche ?
idle-alert-title = Vous avez été inactif•ve pendant {$duration}
idle-alert-description = Voulez vous abandonner ce temps ou continuer le chronomètre ?
break-over-title = La pause est terminée !
break-over-description = Il est temps de se remettre au travail.
pomodoro-over-title = Temps écoulé !
pomodoro-over-description = Êtes-vous prêt à faire une pause ?
snooze-button = {$duration} de plus {$duration ->
    [one] minute
    *[other] minutes
}
long-break = Longue pause
break = Pause
shortcut-exists = Le raccourci existe déjà
shortcut-exists-description = Un raccourci pour cette tâche existe déjà.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} h
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = Il est temps de faire une pause.
idle-notification-title = Vous avez été inactif•ve.
idle-notification-body = Ouvrez Furtherance pour continuer ou abandonner le temps d'inactivité.

## Errors
invalid-database = Base de données invalide.
error-upgrading-database = Erreur lors de la mise à niveau de l'ancienne base de données.
error-accessing-database = Erreur lors de l'accès à la nouvelle base de données.
database-loaded = Base de données chargée.
database-created = Base de données créée.
csv-file-saved = Fichier CSV sauvegardé.
error-writing-csv = Erreur lors de l'écriture des données en CSV.
csv-imported = CSV importé avec succès
invalid-csv-file = Fichier CSV invalide
error-retrieving-tasks = Échec de la récupération des tâches depuis la base de données
error-creating-file = Échec de la création du fichier
error-reading-headers = Échec de la lecture des en-têtes
wrong-column-order = Ordre des colonnes incorrect.
missing-column = Colonne manquante
invalid-csv = CSV invalide
backup-database-failed = Échec de la sauvegarde de la base de données
name-cannot-contain = Le nom de la tâche ne peut pas contenir #, @ ou $.
project-cannot-contain = Le projet ne peut pas contenir #, @ ou $.
tags-cannot-contain = Les étiquettes ne peuvent pas contenir @ ou $.
tags-must-start = Les étiquettes doivent commencer par #.
no-symbol-in-rate = N'incluez pas de $ dans le taux.
rate-invalid = Le taux doit être un montant en dollars valide.
