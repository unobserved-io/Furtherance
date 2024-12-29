### Localization resource for Furtherance (Turkish)
### This file is distributed under the same license as the Furtherance package.
### Last updated: 2024-09-12
### Translated by: Sabri Ünal <libreajans@gmail.com>, 2022, 2024
### Modified with Claude AI for transition to Fluent

## File dialogs
save-backup-title = Furtherance Yedeğini Kaydet
sqlite-database = SQLite Veri Tabanı
sqlite-files = SQLite Dosyaları
backup-successful = Veri Tabanı Yedeklemesi Başarılı
save-csv-title = Furtherance CSV'sini Kaydet
open-csv-title = Furtherance CSV'sini Aç
new-database-title = Yeni Furtherance Veri Tabanı
open-database-title = Furtherance Veri Tabanını Aç

## General UI
shortcuts = Kısayollar
timer = Zamanlayıcı
history = Geçmiş
report = Raporla
settings = Tercihler
today = Bugün
yesterday = Dün
cancel = İptal
save = Kaydet
delete = Sil
edit = Düzenle
ok = Tamam
stop = Dur
continue = Devam
discard = Gözden Çıkar

## Timer
task-input-placeholder = Görev Adı #etiket
started-at = Başlangıç: {$time}
recorded-today = Bugün kaydedilen: {$time}

## History
project-prefix = @{$project}
tags-prefix = #{$tags}
task-count = {$count ->
    [one] {$count} görev
    *[other] {$count} görev
}
total-time-dynamic = Toplam: {$time}
total-earnings = ${$amount}
repeat = Tekrarla

## Shortcuts
new-shortcut = Yeni Kısayol
create-shortcut = Kısayol Oluştur

## Reports
charts = Grafikler
list = Liste
breakdown-by-selection = Seçime Göre Dökümü
total-time = Toplam Süre
earned = Kazanılan
past-week = Geçen hafta
past-thirty-days = Son 30 gün
past-six-months = Son 180 gün
all-time = Tüm zamanlar
date-range = Tarih aralığı
title = Başlık
tags = Etiket
rate = Oran
none = Yok
no-tags = etiket yok

## Settings
general = Genel
advanced = Gelişmiş
pomodoro = Pomodoro
data = Veri

### General Settings
interface = Arayüz
default-view = Varsayılan görünüm
show-delete-confirmation = Silme doğrulaması
task-history = Görev geçmişi
show-project = Projeyi göster
show-tags = Etiketleri göster
show-earnings = Kazançları göster
show-seconds = Saniyeleri göster
show-daily-time-total = Günlük toplamları göster

### Advanced Settings
idle = Boşta
idle-detection = Boşta algılama
minutes-until-idle = Boştaya kalan dakika
dynamic-total = Dinamik toplam
dynamic-total-description = Bugünün toplam süresine devam eden zamanlayıcı dahildir
days-to-show = Gösterilecek gün sayısı

### Pomodoro Settings
pomodoro-timer = Pomodoro zamanlayıcısı
notification-alarm-sound = Bildirim alarm sesi
countdown-timer = Geri sayım zamanlayıcısı
timer-length = Zamanlayıcı uzunluğu
break-length = Mola uzunluğu
snooze-length = Erteleme uzunluğu
extended-break = Uzatılmış mola
extended-breaks = Uzatılmış molalar
extended-break-interval = Uzatılmış mola aralığı
extended-break-length = Uzatılmış mola uzunluğu

### Report Settings
toggle-charts = Grafikleri aç/kapat
total-time-box = Toplam süre kutusu
total-earnings-box = Toplam kazanç kutusu
time-recorded = Kaydedilen süre
earnings = Kazançlar
average-time-per-task = Görev başına ortalama süre
average-earnings-per-task = Görev başına ortalama kazanç
breakdown-by-selection-section = Seçime göre döküm bölümü
time-recorded-for-selection = Seçim için kaydedilen süre
earnings-for-selection = Seçim için kazançlar

### Data Settings
local-database = Yerel Veri Tabanı
database-location = Veri tabanı konumu
create-new = Yeni Oluştur
open-existing = Var Olanı Aç
export-csv = CSV Olarak Dışa Aktar
import-csv = CSV İçe Aktar
backup = Yedekle
backup-database = Veri Tabanını Yedekle

## Inspector
task-name = Görev adı
project = Proje
hashtag-tags = #etiket
start-colon = Başlangıç:
stop-colon = Bitiş:
per-hour = /saat
color = Renk
edit-shortcut = Kısayolu Düzenle
start-to-stop = {$start} - {$stop}
nothing-selected = Hiçbir şey seçilmedi.

## Charts
average-earnings-per-task-title = Görev Başına Ortalama Kazanç
average-time-per-task-title = Görev Başına Ortalama Süre
time-recorded-title = Kaydedilen Süre
time-recorded-for-selection-title = Seçim İçin Kaydedilen Süre
earnings-for-selection-title = Seçim İçin Kazançlar

## Alerts
delete-all = Tümünü sil
delete-all-question = Tümü Silinsin Mi?
delete-all-description = Bu işlem görevin bu gündeki tüm oluşumlarını siler
delete-shortcut-question = Kısayol silinsin mi?
delete-shortcut-description = Bu kısayolu silmek istediğinizden emin misiniz?
delete-task-question = Görev silinsin mi?
delete-task-description = Bu görevi kalıcı olarak silmek istediğinizden emin misiniz?
idle-alert-title = {$duration} süredir boştasınız
idle-alert-description = Bu zamanı gözden çıkarmak mı yoksa saate devam etmek mi istersiniz?
break-over-title = Mola bitti!
break-over-description = İşe geri dönme zamanı.
pomodoro-over-title = Süre doldu!
pomodoro-over-description = Mola vermeye hazır mısınız?
snooze-button = {$duration} daha {$duration ->
    [one] dakika
    *[other] dakika
}
long-break = Uzun mola
break = Mola
shortcut-exists = Kısayol zaten var
shortcut-exists-description = Bu görev için zaten bir kısayol var.

## Sidebar
# Number of hours, mins, secs with only one letter formatter
x-h = {$hours} s
x-m = {$minutes} d
x-s = {$seconds} s

## Notifications
pomodoro-over-notification-body = Mola verme zamanı.
idle-notification-title = Boşta kaldınız.
idle-notification-body = Devam etmek veya boşta geçen zamanı gözden çıkarmak için Furtherance'ı açın.

## Errors
invalid-database = Geçersiz veri tabanı.
error-upgrading-database = Eski veri tabanını yükseltirken hata oluştu.
error-accessing-database = Yeni veri tabanına erişirken hata oluştu.
database-loaded = Veri tabanı yüklendi.
database-created = Veri tabanı oluşturuldu.
csv-file-saved = CSV dosyası kaydedildi.
error-writing-csv = CSV'ye veri yazılırken hata oluştu.
csv-imported = CSV başarıyla içe aktarıldı
invalid-csv-file = Geçersiz CSV dosyası
error-retrieving-tasks = Veri tabanından görevler alınamadı
error-creating-file = Dosya oluşturulamadı
error-reading-headers = Başlıklar okunamadı
wrong-column-order = Yanlış sütun sırası.
missing-column = Eksik sütun
invalid-csv = Geçersiz CSV
backup-database-failed = Veri tabanı yedeklenemedi
name-cannot-contain = Görev adı #, @ veya $ içeremez.
project-cannot-contain = Proje #, @ veya $ içeremez.
tags-cannot-contain = Etiketler @ veya $ içeremez.
tags-must-start = Etiketler # ile başlamalıdır.
no-symbol-in-rate = Oranda $ kullanmayın.
rate-invalid = Oran geçerli bir dolar miktarı olmalıdır.
