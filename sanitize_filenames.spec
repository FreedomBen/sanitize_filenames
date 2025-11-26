Name:           sanitize_filenames
Version:        0.1.0
Release:        1%{?dist}
Summary:        CLI tool to sanitize filenames

License:        AGPLv3+
URL:            https://example.com/sanitize_filenames
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  make
BuildRequires:  gcc
BuildRequires:  musl-gcc

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
install -D -m 0644 completions/sanitize_filenames.bash \
    %{buildroot}%{_datadir}/bash-completion/completions/%{name}
install -D -m 0644 completions/_sanitize_filenames \
    %{buildroot}%{_datadir}/zsh/site-functions/_%{name}
install -D -m 0644 completions/sanitize_filenames.fish \
    %{buildroot}%{_datadir}/fish/vendor_completions.d/%{name}.fish
install -D -m 0644 man/sanitize_filenames.1 \
    %{buildroot}%{_mandir}/man1/%{name}.1

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_datadir}/bash-completion/completions/%{name}
%{_datadir}/zsh/site-functions/_%{name}
%{_datadir}/fish/vendor_completions.d/%{name}.fish
%{_mandir}/man1/%{name}.1*

%changelog
* Tue Nov 26 2024 Packager Name <packager@example.com> - 0.1.0-1
- Initial RPM release
