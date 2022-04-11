project('furtherance', 'rust',
          version: '1.1.0',
    meson_version: '>= 0.50.0',
  default_options: [ 'warning_level=2',
                   ],
)

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

meson.add_install_script('build-aux/meson/postinstall.py')
