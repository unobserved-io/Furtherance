### Localization resource for Furtherance (Russian)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Mikhail Kositsyn <r3pll@yandex.ru>, 2022
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Сохранить резервную копию Furtherance
sqlite-database = База данных SQLite
sqlite-files = Файлы SQLite
backup-successful = Резервное копирование базы данных выполнено успешно
save-csv-title = Сохранить CSV Furtherance
open-csv-title = Открыть CSV Furtherance
new-database-title = Новая база данных Furtherance
open-database-title = Открыть базу данных Furtherance

## General UI
shortcuts = Ярлыки
timer = Таймер
history = История
report = Отчет
settings = Настройки
today = Сегодня
yesterday = Вчера
cancel = Отмена
save = Сохранить
delete = Удалить
edit = Редактировать
ok = OK
stop = Конец
continue = Продолжить
discard = Отменить

## Timer
task-input-placeholder = Название задачи #тег
started-at = Начало в {$time}
recorded-today = Записано сегодня: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} задача
    *[other] {$count} задач
}
total-time-dynamic = Всего: {$time}
total-earnings = ${$amount}
repeat = Повторить

## Shortcuts
new-shortcut = Новый ярлык
create-shortcut = Создать ярлык

## Reports
charts = Графики
list = Список
breakdown-by-selection = Разбивка по выбору
total-time = Общее время
earned = Заработано
past-week = Последняя неделя
past-thirty-days = Последние 30 дней
past-six-months = Последние 180 дней
all-time = За все время
date-range = Диапазон дат
title = Заголовок
tags = Теги
rate = Ставка
none = Нет
no-tags = нет тегов

## Settings
general = Основные
advanced = Расширенные
pomodoro = Pomodoro
data = Данные

### General Settings
interface = Интерфейс
default-view = Вид по умолчанию
show-delete-confirmation = Подтверждение при удалении
task-history = История задач
show-project = Показывать проект
show-tags = Показывать теги
show-earnings = Показывать заработок
show-seconds = Показывать секунды
show-daily-time-total = Показывать общее время за день

### Advanced Settings
idle = Бездействие
idle-detection = Определение бездействия
minutes-until-idle = Минуты до неактивности
dynamic-total = Динамический итог
dynamic-total-description = Общее время за сегодня увеличивается вместе с таймером
days-to-show = Количество дней для отображения

### Pomodoro Settings
pomodoro-timer = Таймер Pomodoro
countdown-timer = Таймер обратного отсчета
timer-length = Длительность таймера
break-length = Длительность перерыва
snooze-length = Длительность отсрочки
extended-break = Длительный перерыв
extended-breaks = Длительные перерывы
extended-break-interval = Интервал длительного перерыва
extended-break-length = Длительность длительного перерыва

### Report Settings
toggle-charts = Переключить графики
total-time-box = Поле общего времени
total-earnings-box = Поле общего заработка
time-recorded = Записанное время
earnings = Заработок
average-time-per-task = Среднее время на задачу
average-earnings-per-task = Средний заработок на задачу
breakdown-by-selection-section = Раздел разбивки по выбору
time-recorded-for-selection = Записанное время для выбора
earnings-for-selection = Заработок для выбора

### Data Settings
local-database = Локальная база данных
database-location = Расположение базы данных
create-new = Создать новую
open-existing = Открыть существующую
export-csv = Экспорт в CSV
import-csv = Импорт из CSV
backup = Резервное копирование
backup-database = Резервное копирование базы данных

## Inspector
task-name = Название задачи
project = Проект
hashtag-tags = #теги
start-colon = Начало:
stop-colon = Конец:
per-hour = /час
color = Цвет
edit-shortcut = Редактировать ярлык
start-to-stop = {$start} до {$stop}
nothing-selected = Ничего не выбрано.

## Charts
average-earnings-per-task-title = Средний заработок на задачу
average-time-per-task-title = Среднее время на задачу
time-recorded-title = Записанное время
time-recorded-for-selection-title = Записанное время для выбора
earnings-for-selection-title = Заработок для выбора

## Alerts
delete-all = Удалить все
delete-all-question = Удалить все?
delete-all-description = Это удалит все записи этой задачи за сегодня.
delete-shortcut-question = Удалить ярлык?
delete-shortcut-description = Вы уверены, что хотите удалить этот ярлык?
delete-task-question = Удалить задачу?
delete-task-description = Вы уверены, что хотите навсегда удалить эту задачу?
idle-alert-title = Вы бездействовали в течение {$duration}
idle-alert-description = Вы хотели бы сбросить время или продолжить отсчет?
break-over-title = Перерыв закончен!
break-over-description = Пора возвращаться к работе.
pomodoro-over-title = Время вышло!
pomodoro-over-description = Вы готовы сделать перерыв?
snooze-button = Ещё {$duration} {$duration ->
    [one] минута
    *[other] минут
}
long-break = Длительный перерыв
break = Перерыв
shortcut-exists = Ярлык уже существует
shortcut-exists-description = Ярлык для этой задачи уже существует.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} ч
x-m = {$minutes} м
x-s = {$seconds} с

## Notifications
pomodoro-over-notification-body = Пора сделать перерыв.
idle-notification-title = Вы были неактивны.
idle-notification-body = Откройте Furtherance, чтобы продолжить или отменить время бездействия.

## Errors
invalid-database = Недопустимая база данных.
error-upgrading-database = Ошибка при обновлении старой базы данных.
error-accessing-database = Ошибка при доступе к новой базе данных.
database-loaded = База данных загружена.
database-created = База данных создана.
csv-file-saved = CSV-файл сохранен.
error-writing-csv = Ошибка при записи данных в CSV.
csv-imported = CSV успешно импортирован
invalid-csv-file = Недопустимый CSV-файл
error-retrieving-tasks = Не удалось получить задачи из базы данных
error-creating-file = Не удалось создать файл
error-reading-headers = Не удалось прочитать заголовки
wrong-column-order = Неправильный порядок столбцов.
missing-column = Отсутствует столбец
invalid-csv = Недопустимый CSV
backup-database-failed = Не удалось создать резервную копию базы данных
name-cannot-contain = Название задачи не может содержать #, @ или $.
project-cannot-contain = Проект не может содержать #, @ или $.
tags-cannot-contain = Теги не могут содержать @ или $.
tags-must-start = Теги должны начинаться с #.
no-symbol-in-rate = Не включайте $ в ставку.
rate-invalid = Ставка должна быть действительной суммой в долларах.
