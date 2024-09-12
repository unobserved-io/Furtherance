### Localization resource for Furtherance
### Last updated: 2024-09-11

## File dialogs
save-backup-title = Save Furtherance Backup
sqlite-database = SQLite Database
sqlite-files = SQLite Files
backup-successful = Database Backup Successful
save-csv-title = Save Furtherance CSV
open-csv-title = Open Furtherance CSV
new-database-title = New Furtherance Database
open-database-title = Open Furtherance Database

## General UI
shortcuts = Shortcuts
timer = Timer
history = History
report = Report
settings = Settings
today = Today
yesterday = Yesterday
cancel = Cancel
save = Save
delete = Delete
edit = Edit
ok = OK
stop = Stop
continue = Continue
discard = Discard

## Timer
task-input-placeholder = Task name @Project #tags $rate
started-at = Started at {$time}
recorded-today = Recorded today: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} task
    *[other] {$count} tasks
}
total-time-dynamic = Total: {$time}
total-earnings = ${$amount}
repeat = Repeat

## Shortcuts
new-shortcut = New Shortcut
create-shortcut = Create shortcut

## Reports
charts = Charts
list = List
breakdown-by-selection = Breakdown By Selection
total-time = Total Time
earned = Earned
past-week = Past week
past-thirty-days = Past 30 days
past-six-months = Past 6 months
all-time = All time
date-range = Date range
title = Title
tags = Tags
rate = Rate

## Settings
general = General
advanced = Advanced
pomodoro = Pomodoro
data = Data

### General Settings
interface = Interface
default-view = Default view
show-delete-confirmation = Show delete confirmation
task-history = Task History
show-project = Show project
show-tags = Show tags
show-earnings = Show earnings
show-seconds = Show seconds
show-daily-time-total = Show daily time total

### Advanced Settings
idle = Idle
idle-detection = Idle detection
minutes-until-idle = Minutes until idle
dynamic-total = Dynamic total
dynamic-total-description = Today's total time ticks up with the timer
days-to-show = Days to show

### Pomodoro Settings
pomodoro-timer = Pomodoro timer
countdown-timer = Countdown timer
timer-length = Timer length
break-length = Break length
snooze-length = Snooze length
extended-break = Extended break
extended-breaks = Extended breaks
extended-break-interval = Extended break interval
extended-break-length = Extended break length

### Report Settings
toggle-charts = Toggle charts
total-time-box = Total time box
total-earnings-box = Total earnings box
time-recorded = Time recorded
earnings = Earnings
average-time-per-task = Average time per task
average-earnings-per-task = Average earnings per task
breakdown-by-selection-section = Breakdown by selection section
time-recorded-for-selection = Time recorded for selection
earnings-for-selection = Earnings for selection

### Data Settings
local-database = Local Database
database-location = Database location
create-new = Create New
open-existing = Open Existing
export-csv = Export CSV
import-csv = Import CSV
backup = Backup
backup-database = Backup Database

## Inspector
task-name = Task name
project = Project
hashtag-tags = #tags
start-colon = Start:
stop-colon = Stop:
per-hour = /hr
color = Color
edit-shortcut = Edit Shortcut
start-to-stop = {$start} to {$stop}
nothing-selected = Nothing selected.

## Charts
average-earnings-per-task-title = Average Earnings Per Task
average-time-per-task-title = Average Time Per Task
time-recorded-title = Time Recorded
time-recorded-for-selection-title = Time Recorded For Selection
earnings-for-selection-title = Earnings For Selection

## Alerts
delete-all = Delete All
delete-all-question = Delete all?
delete-all-description = Are you sure you want to permanently delete all tasks in this group?
delete-shortcut-question = Delete shortcut?
delete-shortcut-description = Are you sure you want to delete this shortcut?
delete-task-question = Delete task?
delete-task-description = Are you sure you want to permanently delete this task?
idle-alert-title = You have been idle for {$duration}
idle-alert-description = Would you like to discard that time, or continue the clock?
break-over-title = Break's over!
break-over-description = Time to get back to work.
pomodoro-over-title = Time's up!
pomodoro-over-description = Are you ready to take a break?
snooze-button = {$duration} more {$duration ->
    [one] minute
    *[other] minutes
}
long-break = Long break
break = Break
shortcut-exists = Shortcut exists
shortcut-exists-description = A shortcut for that task already exists.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} h
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = It's time to take a break.
idle-notification-title = You've been idle.
idle-notification-body = Open Furtherance to continue or discard the idle time.

## Errors
invalid-database = Invalid database.
error-upgrading-database = Error upgrading legacy database.
error-accessing-database = Error accessing new database.
database-loaded = Database loaded.
database-created = Database created.
csv-file-saved = CSV file saved.
error-writing-csv = Error writing data to CSV.
csv-imported = CSV imported successfully
invalid-csv-file = Invalid CSV file
error-retrieving-tasks = Failed to retrieve tasks from the database
error-creating-file = Failed to create the file
error-reading-headers = Failed to read the headers
wrong-column-order = Wrong column order.
missing-column = Missing column
invalid-csv = Invalid CSV
backup-database-failed = Failed to backup database
name-cannot-contain = Task name cannot contain #, @, or $.
project-cannot-contain = Project cannot contain #, @, or $.
tags-cannot-contain = Tags cannot contain @ or $.
tags-must-start = Tags must start with a #.
no-symbol-in-rate = Do not include a $ in the rate.
rate-invalid = Rate must be a valid dollar amount.
