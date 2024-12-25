### Localization resource for Furtherance (Brazilian Portuguese)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Pedro Sader Azevedo <pedro.saderazevedo@protonmail.com>, 2022
### and Rodrigo dos Santos <rodrigo.sabbat@gmail.com>, 2022
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Salvar backup do Furtherance
sqlite-database = Banco de dados SQLite
sqlite-files = Arquivos SQLite
backup-successful = Backup do banco de dados bem-sucedido
save-csv-title = Salvar CSV do Furtherance
open-csv-title = Abrir CSV do Furtherance
new-database-title = Novo banco de dados do Furtherance
open-database-title = Abrir banco de dados do Furtherance

## General UI
shortcuts = Atalhos
timer = Cronômetro
history = Histórico
report = Relatório
settings = Preferências
today = Hoje
yesterday = Ontem
cancel = Cancelar
save = Salvar
delete = Deletar
edit = Editar
ok = OK
stop = Parar
continue = Continuar
discard = Descartar

## Timer
task-input-placeholder = Nome da atividade #tags
started-at = Iniciado às {$time}
recorded-today = Registrado hoje: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} atividade
    *[other] {$count} atividades
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
breakdown-by-selection = Detalhamento por seleção
total-time = Tempo total
earned = Ganho
past-week = Semana passada
past-thirty-days = Últimos 30 dias
past-six-months = Últimos 180 dias
all-time = Todo o tempo
date-range = Intervalo de datas
title = Título
tags = Tags
rate = Taxa
none = Nenhum
no-tags = sem tags

## Settings
general = Geral
advanced = Avançado
pomodoro = Pomodoro
data = Dados

### General Settings
interface = Interface
default-view = Visualização padrão
show-delete-confirmation = Confirmar antes de deletar
task-history = Histórico de atividades
show-project = Mostrar projeto
show-tags = Mostrar tags
show-earnings = Mostrar ganhos
show-seconds = Mostrar segundos
show-daily-time-total = Mostrar somas diárias

### Advanced Settings
idle = Inatividade
idle-detection = Detecção de inatividade
minutes-until-idle = Minutos para inatividade
dynamic-total = Total dinâmico
dynamic-total-description = O tempo total de hoje aumenta com o cronômetro
days-to-show = Dias para mostrar

### Pomodoro Settings
pomodoro-timer = Cronômetro Pomodoro
notification_alarm_sound = Som de alarme de notificação
countdown-timer = Cronômetro de contagem regressiva
timer-length = Duração do cronômetro
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
time-recorded = Tempo registrado
earnings = Ganhos
average-time-per-task = Tempo médio por atividade
average-earnings-per-task = Ganhos médios por atividade
breakdown-by-selection-section = Seção de detalhamento por seleção
time-recorded-for-selection = Tempo registrado para seleção
earnings-for-selection = Ganhos para seleção

### Data Settings
local-database = Banco de dados local
database-location = Localização do banco de dados
create-new = Criar novo
open-existing = Abrir existente
export-csv = Exportar CSV
import-csv = Importar CSV
backup = Backup
backup-database = Fazer backup do banco de dados

## Inspector
task-name = Nome da atividade
project = Projeto
hashtag-tags = #tags
start-colon = Início:
stop-colon = Fim:
per-hour = /hora
color = Cor
edit-shortcut = Editar atalho
start-to-stop = {$start} a {$stop}
nothing-selected = Nada selecionado.

## Charts
average-earnings-per-task-title = Ganhos médios por atividade
average-time-per-task-title = Tempo médio por atividade
time-recorded-title = Tempo registrado
time-recorded-for-selection-title = Tempo registrado para seleção
earnings-for-selection-title = Ganhos para seleção

## Alerts
delete-all = Deletar tudo
delete-all-question = Deletar tudo?
delete-all-description = Isso vai eliminar todas as ocorrências dessa atividade nesse dia.
delete-shortcut-question = Deletar atalho?
delete-shortcut-description = Tem certeza de que deseja deletar este atalho?
delete-task-question = Deletar atividade?
delete-task-description = Tem certeza de que deseja deletar permanentemente esta atividade?
idle-alert-title = Você ficou inativo por {$duration}
idle-alert-description = Você gostaria de descartar ou continuar o tempo registrado?
break-over-title = A pausa acabou!
break-over-description = Hora de voltar ao trabalho.
pomodoro-over-title = O tempo acabou!
pomodoro-over-description = Você está pronto para fazer uma pausa?
snooze-button = Mais {$duration} {$duration ->
    [one] minuto
    *[other] minutos
}
long-break = Pausa longa
break = Pausa
shortcut-exists = O atalho já existe
shortcut-exists-description = Já existe um atalho para essa atividade.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} h
x-m = {$minutes} m
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = É hora de fazer uma pausa.
idle-notification-title = Você ficou inativo.
idle-notification-body = Abra o Furtherance para continuar ou descartar o tempo de inatividade.

## Errors
invalid-database = Banco de dados inválido.
error-upgrading-database = Erro ao atualizar o banco de dados antigo.
error-accessing-database = Erro ao acessar o novo banco de dados.
database-loaded = Banco de dados carregado.
database-created = Banco de dados criado.
csv-file-saved = Arquivo CSV salvo.
error-writing-csv = Erro ao escrever dados no CSV.
csv-imported = CSV importado com sucesso
invalid-csv-file = Arquivo CSV inválido
error-retrieving-tasks = Falha ao recuperar atividades do banco de dados
error-creating-file = Falha ao criar o arquivo
error-reading-headers = Falha ao ler os cabeçalhos
wrong-column-order = Ordem de colunas incorreta.
missing-column = Coluna ausente
invalid-csv = CSV inválido
backup-database-failed = Falha ao fazer backup do banco de dados
name-cannot-contain = O nome da atividade não pode conter #, @ ou $.
project-cannot-contain = O projeto não pode conter #, @ ou $.
tags-cannot-contain = As tags não podem conter @ ou $.
tags-must-start = As tags devem começar com #.
no-symbol-in-rate = Não inclua $ na taxa.
rate-invalid = A taxa deve ser um valor válido em dólares.
