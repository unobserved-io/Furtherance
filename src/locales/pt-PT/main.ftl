### Localization resource for Furtherance (Portuguese)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: ssantos <ssantos@web.de>, 2023, 2024
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Gravar cópia de segurança do Furtherance
sqlite-database = Base de dados SQLite
sqlite-files = Ficheiros SQLite
backup-successful = Cópia de segurança da base de dados bem-sucedida
save-csv-title = Gravar CSV do Furtherance
open-csv-title = Abrir CSV do Furtherance
new-database-title = Nova base de dados do Furtherance
open-database-title = Abrir base de dados do Furtherance

## General UI
shortcuts = Atalhos
timer = Temporizador
history = Histórico
report = Relatório
settings = Preferências
today = Hoje
yesterday = Ontem
cancel = Cancelar
save = Gravar
delete = Apagar
edit = Editar
ok = OK
stop = Parar
continue = Continuar
discard = Descartar

## Timer
task-input-placeholder = Nome da tarefa #tags
started-at = Iniciado às {$time}
recorded-today = Gravado hoje: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} tarefa
    *[other] {$count} tarefas
}
total-time-dynamic = Total: {$time}
total-earnings = ${$amount}
repeat = Repetir

## Shortcuts
new-shortcut = Novo atalho
create-shortcut = Criar atalho

## Reports
charts = Gráficos
list = Lista
breakdown-by-selection = Detalhe por seleção
total-time = Tempo total
earned = Ganho
past-week = Semana passada
past-thirty-days = Últimos 30 dias
past-six-months = Últimos 180 dias
all-time = Todo o tempo
date-range = Intervalo de datas
title = Título
tags = Etiquetas
rate = Taxa
none = Nenhum
no-tags = sem etiquetas

## Settings
general = Geral
advanced = Avançado
pomodoro = Pomodoro
data = Dados

### General Settings
interface = Interface
default-view = Vista predefinida
show-delete-confirmation = Confirmar antes de apagar
task-history = Histórico de tarefas
show-project = Mostrar projeto
show-tags = Mostrar etiquetas
show-earnings = Mostrar ganhos
show-seconds = Mostrar segundos
show-daily-time-total = Mostrar somas diárias

### Advanced Settings
idle = Inatividade
idle-detection = Deteção de inatividade
minutes-until-idle = Minutos para inatividade
dynamic-total = Total dinâmico
dynamic-total-description = O tempo total de hoje inclui o temporizador em curso
days-to-show = Dias a mostrar

### Pomodoro Settings
pomodoro-timer = Temporizador Pomodoro
notification-alarm-sound = Som de alarme de notificação
countdown-timer = Temporizador de contagem decrescente
timer-length = Duração do temporizador
break-length = Duração da pausa
snooze-length = Duração da soneca
extended-break = Pausa prolongada
extended-breaks = Pausas prolongadas
extended-break-interval = Intervalo de pausa prolongada
extended-break-length = Duração da pausa prolongada

### Report Settings
toggle-charts = Alternar gráficos
total-time-box = Caixa de tempo total
total-earnings-box = Caixa de ganhos totais
time-recorded = Tempo gravado
earnings = Ganhos
average-time-per-task = Tempo médio por tarefa
average-earnings-per-task = Ganhos médios por tarefa
breakdown-by-selection-section = Secção de detalhe por seleção
time-recorded-for-selection = Tempo gravado para seleção
earnings-for-selection = Ganhos para seleção

### Data Settings
local-database = Base de dados local
database-location = Localização da base de dados
create-new = Criar nova
open-existing = Abrir existente
export-csv = Exportar CSV
import-csv = Importar CSV
backup = Cópia de segurança
backup-database = Criar cópia de segurança da base de dados

## Inspector
task-name = Nome da tarefa
project = Projeto
hashtag-tags = #etiquetas
start-colon = Início:
stop-colon = Fim:
per-hour = /hora
color = Cor
edit-shortcut = Editar atalho
start-to-stop = {$start} a {$stop}
nothing-selected = Nada selecionado.

## Charts
average-earnings-per-task-title = Ganhos médios por tarefa
average-time-per-task-title = Tempo médio por tarefa
time-recorded-title = Tempo gravado
time-recorded-for-selection-title = Tempo gravado para seleção
earnings-for-selection-title = Ganhos para seleção

## Alerts
delete-all = Apagar tudo
delete-all-question = Apagar tudo?
delete-all-description = Isto irá apagar todas as ocorrências desta atividade de hoje.
delete-shortcut-question = Apagar atalho?
delete-shortcut-description = Tem a certeza de que quer apagar este atalho?
delete-task-question = Apagar a atividade?
delete-task-description = Tem a certeza de que quer apagar permanentemente esta tarefa?
idle-alert-title = Ficou inativo por {$duration}
idle-alert-description = Gostaria de descartar ou continuar o tempo registado?
break-over-title = A pausa terminou!
break-over-description = Hora de voltar ao trabalho.
pomodoro-over-title = O tempo acabou!
pomodoro-over-description = Está pronto para fazer uma pausa?
snooze-button = Mais {$duration} {$duration ->
    [one] minuto
    *[other] minutos
}
long-break = Pausa longa
break = Pausa
shortcut-exists = O atalho já existe
shortcut-exists-description = Já existe um atalho para essa tarefa.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} h
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = É hora de fazer uma pausa.
idle-notification-title = Esteve inativo.
idle-notification-body = Abra o Furtherance para continuar ou descartar o tempo de inatividade.

## Errors
invalid-database = Base de dados inválida.
error-upgrading-database = Erro ao atualizar a base de dados antiga.
error-accessing-database = Erro ao aceder à nova base de dados.
database-loaded = Base de dados carregada.
database-created = Base de dados criada.
csv-file-saved = Ficheiro CSV gravado.
error-writing-csv = Erro ao escrever dados para CSV.
csv-imported = CSV importado com sucesso
invalid-csv-file = Ficheiro CSV inválido
error-retrieving-tasks = Falha ao recuperar tarefas da base de dados
error-creating-file = Falha ao criar o ficheiro
error-reading-headers = Falha ao ler os cabeçalhos
wrong-column-order = Ordem das colunas errada.
missing-column = Coluna em falta
invalid-csv = CSV inválido
backup-database-failed = Falha ao criar cópia de segurança da base de dados
name-cannot-contain = O nome da tarefa não pode conter #, @ ou $.
project-cannot-contain = O projeto não pode conter #, @ ou $.
tags-cannot-contain = As etiquetas não podem conter @ ou $.
tags-must-start = As etiquetas devem começar com #.
no-symbol-in-rate = Não inclua $ na taxa.
rate-invalid = A taxa deve ser um valor válido em dólares.
