# flatpak-sync

Easily your flatpaks between your linux PCs!

# Usage

## Installation

1. Ensure the [Rust Programming Language](https://www.rust-lang.org/) is installed
2. Clone and build the repository

```shell
$ git clone git@github.com:NoSpawnn/flatpak-sync.git
$ cd flatpak-sync
$ cargo build --release
```

## Running

```shell
flatpak-sync [OPTIONS] --username <USERNAME> --remote-host <REMOTE_HOST> [--port <PORT>]
```

- See `flatpak-sync --help` for more info

# TOD

- [ ] Blacklist flatpaks from sync
- [ ] Systemd (probably) service for ongoing syncing
- [ ] Config sync (maybe)
- [ ] GUI

# Reference links

- [Flatpak Command Reference - Flatpak documentation](https://docs.flatpak.org/en/latest/flatpak-command-reference.html)
- [ssh-keygen(1) - Linux manual page](https://www.man7.org/linux/man-pages/man1/ssh-keygen.1.html)
- [ssh-copy-id(1) - Linux manual page](https://www.man7.org/linux/man-pages/man1/ssh-copy-id.1.html)
