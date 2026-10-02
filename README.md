# GiftWallpaper

Open-source desktop wallpaper generator for Telegram collectible gifts, written in Rust.

> Gift metadata and assets are powered by [@GiftChanges](https://t.me/GiftChanges) — [api.changes.tg](https://api.changes.tg/).


## v0.2 changes

- Always-visible scrollbars for gifts, models and the sidebar
- Model search
- Gift scale slider
- Vertical-position slider
- Correct landscape/square preview aspect ratio
- Phone, desktop, 4K, square and 16:10 presets
- Fully custom width/height from 256 to 7680 pixels

## MVP features

- Native Rust desktop UI with `egui` / `eframe`
- Loads the live list of upgradable Telegram gifts
- Loads models from `GET /gift/:gift`
- Downloads 1024px model assets
- Four generated wallpaper styles
- Phone and desktop resolutions
- PNG export
- Local image cache
- Background networking so the UI does not freeze
- Windows and Linux GitHub Actions builds
- GitHub Release packaging

## Screens / flow

1. Search for a gift.
2. Select a gift.
3. Select one of its models.
4. Choose a wallpaper preset.
5. Choose phone or desktop resolution.
6. Export PNG into the `exports/` directory.

## Run locally

### Windows

1. Install Rust from `https://rustup.rs/`.
2. Extract this repository.
3. Double-click `RUN_WINDOWS.bat`.

To validate formatting, compilation, tests and Clippy, double-click:

```text
CHECK_WINDOWS.bat
```

### Linux (Debian/Ubuntu)

First install native build dependencies:

```bash
./INSTALL_LINUX_DEPS.sh
```

Install Rust from `https://rustup.rs/`, then:

```bash
./RUN_LINUX.sh
```

Validation:

```bash
./CHECK_LINUX.sh
```

Manual release build on either platform:

```bash
cargo build --release
```

The binary will be located at:

- Windows: `target/release/giftwallpaper.exe`
- Linux: `target/release/giftwallpaper`

## GitHub Releases

The repository includes `.github/workflows/release.yml`.

Create and push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build:

- `giftwallpaper-windows-x64.zip`
- `giftwallpaper-linux-x64.tar.gz`

and attach them to the GitHub Release for the tag.

You can also run the Release workflow manually from the Actions tab.

## Project layout

```text
giftwallpaper/
├── src/
│   ├── main.rs       # native application entry point
│   ├── app.rs        # egui UI and worker state
│   ├── api.rs        # api.changes.tg client and defensive JSON parsing
│   └── renderer.rs   # wallpaper rendering
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── CREDITS.md
├── CONTRIBUTING.md
├── LICENSE
└── Cargo.toml
```

## Data source

GiftWallpaper uses:

- `GET /gifts`
- `GET /gift/:gift`
- `GET /model/:gift/:model.png?size=1024`

The app intentionally keeps the upstream integration small for the first release. Backdrops, symbols, original gifts, TGS animation and custom emoji support can be added later.

## Privacy

GiftWallpaper does not contain analytics or telemetry. Requests are sent only to the configured upstream Gift Changes API when data/assets are needed.

## License

MIT. See [LICENSE](LICENSE).

## Credits

Thanks to **[@GiftChanges](https://t.me/GiftChanges)** for providing the gift metadata and assets API at **api.changes.tg**.


## Project repository

GitHub: https://github.com/Kolt5ik

The desktop app footer now includes an **Open Source · GitHub · Kolt5ik** link, while preserving the visible Gift Changes attribution required by the upstream API.


## v0.2.2

- Fixed footer GitHub link visibility.
- `GitHub · Kolt5ik` is now always visible in the bottom center.
- Added a separate `Open Source` label.
