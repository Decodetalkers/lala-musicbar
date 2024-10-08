project(
  'lala-bar',
  'rust',
  version: '0.3.10',
  license: 'MIT',
  meson_version: '>= 1.0',
)

cargo = find_program('cargo', required: true, version: '>=1.80')
find_program('rustc', required: true, version: '>=1.80')
if get_option('debug')
  command = [
    cargo,
    'build',
    '-p', meson.project_name(),
    '&&',
    'cp',
    meson.global_source_root() / 'target' / 'debug' / meson.project_name(),
    '@OUTPUT@',
  ]
else
  command = [
    cargo,
    'build',
    '-p', meson.project_name(),
    '--release', '&&',
    'cp',
    meson.global_source_root() / 'target' / 'release' / meson.project_name(),
    '@OUTPUT@',
  ]
endif

prefix = get_option('prefix')

bindir = prefix / get_option('bindir')

systemd_dir = prefix / get_option('libdir') / 'systemd' / 'user'

global_conf = configuration_data()

global_conf.set('bin_install_dir', bindir)

systemd_service_file = configure_file(
  input: './misc/lala-bar.service.in',
  output: 'lala-bar.service',
  configuration: global_conf,
)

custom_target(
  'lala-bar',
  output: 'lala-bar',
  build_by_default: true,
  install: true,
  install_dir: bindir,
  console: true,
  command: command,
)

install_data(systemd_service_file, install_dir: systemd_dir)
