Name:           demonhide
Version:        0.1.1
Release:        1%{?dist}
Summary:        Automatic pointer constraint daemon for XWayland fullscreen applications

License:        MIT
URL:            https://github.com/homeos-linux/demonhide
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.90
BuildRequires:  cargo
BuildRequires:  wayland-devel
BuildRequires:  wayland-protocols-devel
BuildRequires:  glib2-devel
BuildRequires:  libX11-devel
BuildRequires:  libXfixes-devel
BuildRequires:  pkgconfig

Requires:       libwayland-cursor
Requires:       libwayland-egl
Requires:       libX11
Requires:       libXfixes
Requires:       glib2

%description
DemonHide is a lightweight daemon that automatically manages pointer constraints 
on Wayland compositors for XWayland fullscreen applications with hidden cursors.
It monitors XWayland applications and locks the mouse pointer when fullscreen 
applications with hidden cursors are detected, preventing cursor movement 
outside the application window.

%prep
%autosetup

# Copy the vendored crates and config to a temp CARGO_HOME
mkdir -p %{_builddir}/cargo_home
cp -r vendor %{_builddir}/cargo_home/vendor
mkdir -p %{_builddir}/cargo_home/.cargo
cat > %{_builddir}/cargo_home/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
# Tell cargo to use the temp CARGO_HOME
CARGO_HOME=%{_builddir}/cargo_home cargo build --release --offline

%install
install -D -m 755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}

%changelog
* Thu Oct 09 2025 Leonie Ain <me@koyu.space> - 0.1.1-1
- Initial package release
- Automatic pointer constraint management for XWayland applications
- Fullscreen application detection with cursor visibility checking
- Wayland pointer locking support
