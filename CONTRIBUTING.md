# Contributing

Contributions are welcome.

## Development

```bash
cargo fmt
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

Please keep changes focused and avoid introducing telemetry.

## Pull requests

A good pull request should:

- explain the user-facing change;
- include tests for parser or renderer logic when practical;
- pass `cargo fmt`, `cargo test`, and `cargo clippy`;
- preserve the visible Gift Changes API attribution in the shipped application.

## Ideas for future contributions

- backdrop-aware color palettes;
- symbol/pattern composition;
- original gift wallpapers;
- TGS/Lottie rendering;
- more presets;
- custom user backgrounds;
- export to WebP;
- Windows installer;
- Linux AppImage or Flatpak;
- localization.
