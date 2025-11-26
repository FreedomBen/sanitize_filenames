Name:           sanitize_filenames
Version:        0.1.0
Release:        1%{?dist}
Summary:        CLI tool to sanitize filenames

License:        AGPLv3+
URL:            https://example.com/sanitize_filenames
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  musl-gcc
BuildRequires:  make
BuildRequires:  gcc

%description
sanitize_filenames is a small Rust-based command-line tool that renames
files and directories to make their names safer and easier to work with.
It replaces problematic characters, supports recursive operation, and
can perform dry runs to preview changes without modifying the filesystem.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --target x86_64-unknown-linux-musl

%install
install -D -m 0755 target/x86_64-unknown-linux-musl/release/%{name} \
    %{buildroot}%{_bindir}/%{name}

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}

%changelog
* Tue Nov 26 2024 Packager Name <packager@example.com> - 0.1.0-1
- Initial RPM release
