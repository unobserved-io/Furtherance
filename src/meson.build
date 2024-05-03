pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())

gnome.compile_resources('furtherance',
  'furtherance.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('GETTEXT_PACKAGE', 'furtherance')
conf.set_quoted('LOCALEDIR', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('PKGDATADIR', pkgdatadir)
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('PROFILE', profile)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.project_build_root(), 'src', 'config.rs'),
  join_paths(meson.project_source_root(), 'src', 'config.rs'),
  check: true
)

rust_sources = files(
  'ui.rs',
  'ui/preferences_dialog.rs',
  'ui/report.rs',
  'ui/task_details.rs',
  'ui/task_row.rs',
  'ui/tasks_group.rs',
  'ui/tasks_page.rs',
  'ui/history_box.rs',
  'ui/window.rs',

  'application.rs',
  'config.rs',
  'main.rs',
  'database.rs',
  'settings_manager.rs',
)

sources = [cargo_sources, rust_sources]

cargo_script = find_program(join_paths(meson.project_source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.project_build_root(),
    meson.project_source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
