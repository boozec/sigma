# sigma

This repo refers to a "more beautiful" version of [`strace(1)`](https://www.man7.org/linux/man-pages/man1/strace.1.html) in Rust.

Trace a command
[![asciicast](https://asciinema.org/a/bvkc2sTphVwm77kB4GRLY5XMQ.svg)](https://asciinema.org/a/bvkc2sTphVwm77kB4GRLY5XMQ)

Attach a PID
[![asciicast](https://asciinema.org/a/LExqUuW3Y3AUvyI7V67XGrgZX.svg)](https://asciinema.org/a/LExqUuW3Y3AUvyI7V67XGrgZX)

## Install

From Crates.io
```
cargo install sigma-trace
```

or build source
```
git clone https://github.com/boozec/sigma
cd sigma
cargo build --release
```

## Help

```
Monitor Linux executables with an interface easier than strace(1)

Usage: sigma-trace [OPTIONS]

Options:
  -c, --command <COMMAND>     Command to execute from ptrace
  -p, --attach <ATTACH>       Attach the tracing to an existing process ID. We're using the `-p` short flag because strace uses it
  -f, --filter <FILTER>       Show only defined sys calls. Multi values separated by comma `,`
      --file <FILE_TO_PRINT>  Write the output to a file instead of the standard output
      --no-tui                If defined, it hides the TUI
  -h, --help                  Print help
  -V, --version               Print version
```
