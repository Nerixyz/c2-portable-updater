# Chatterino Portable Updater

This is a drop-in replacement for the original Chatterino portable updater on Windows that used WinForms.
This updater doesn't have any runtime dependencies (not even the CRT) and has a similar size (about 260KiB) to the original updater.

## Building

To build the updater, you need to have [Rust](https://rust-lang.org) installed.

```powershell
cargo build -r
```

The resulting updater will be in `target/release`.
