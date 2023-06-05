project('furtherance', 'rust',
          version: '1.8.0',
    meson_version: '>= 0.59.0',
  default_options: [ 'warning_level=2',
                   ],
)

gnome = import('gnome')

# Dependencies
dependency('sqlite3', version: '>= 3.20')
dependency('dbus-1')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>=1.0.0')

name       = 'Furtherance'
app_id     = 'com.lakoliu.Furtherance'
profile    = get_option('profile')

# Change app id during development
if profile == 'development'
  app_id = '@0@.Devel'.format(app_id)
endif

i18n = import('i18n')


cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock',
)

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
