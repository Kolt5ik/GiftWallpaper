# Publish to GitHub

The project is ready to be used as a public GitHub repository.

## Fastest option with GitHub CLI

Install and authenticate the GitHub CLI first:

```bash
gh auth login
```

Then, from the `giftwallpaper` directory:

```bash
git init
git add .
git commit -m "Initial GiftWallpaper MVP"
git branch -M main
gh repo create giftwallpaper --public --source=. --remote=origin --push
```

## Create the first downloadable Windows/Linux release

After the repository is online:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The included GitHub Actions workflow will build and publish:

- `giftwallpaper-windows-x64.zip`
- `giftwallpaper-linux-x64.tar.gz`

Open the repository's **Actions** tab if you want to run the Release workflow manually.

## Before publishing

You may optionally update the package metadata in `Cargo.toml` with your final GitHub repository URL.

Do not remove the visible Gift Changes attribution from the application. GiftWallpaper uses
`api.changes.tg`, whose usage terms require end-user-visible attribution.
