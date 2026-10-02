

# GiftWallpaper <img width="32" height="32" alt="icon" src="https://github.com/user-attachments/assets/be55b017-eb5d-45bd-b958-7bae449c8308" />

Open-source приложение для создания обоев с коллекционными подарками Telegram, написанное на Rust.

> Метаданные и ресурсы подарков предоставляются [@GiftChanges](https://t.me/GiftChanges)<img width="20" height="20" alt="photo_2026-10-02_22-54-39" src="https://github.com/user-attachments/assets/08a73ffb-6aa0-4461-a99d-4aca4d9c7e4b" />— [api.changes.tg](https://api.changes.tg/).

## Изменения в v0.2

- Всегда видимые полосы прокрутки для подарков, моделей и боковой панели
- Поиск по моделям
- Ползунок масштаба подарка
- Ползунок вертикального положения
- Корректное соотношение сторон предпросмотра для горизонтального и квадратного формата
- Готовые пресеты для телефона, рабочего стола, 4K, квадрата и 16:10
- Возможность вручную задать ширину и высоту от 256 до 7680 пикселей

## Возможности MVP


Uploading photo_2025-01-04_11-37-16.mp4…


- Нативный desktop-интерфейс на Rust с `egui` / `eframe`
- Загрузка актуального списка доступных для улучшения Telegram Gifts
- Загрузка моделей через `GET /gift/:gift`
- Загрузка PNG-изображений моделей размером 1024 px
- Четыре генерируемых стиля обоев
- Разрешения для телефона и рабочего стола
- Экспорт PNG
- Локальный кэш изображений
- Сетевые запросы выполняются в фоне, поэтому интерфейс не зависает
- Сборки для Windows и Linux через GitHub Actions
- Подготовка файлов для GitHub Releases

## Как пользоваться

1. Найдите подарок.
2. Выберите подарок.
3. Выберите одну из его моделей.
4. Выберите стиль обоев.
5. Выберите разрешение для телефона или рабочего стола.
6. Экспортируйте PNG в папку `exports/`.

## Локальный запуск

### Windows

1. Установите Rust с `https://rustup.rs/`.
2. Распакуйте репозиторий.
3. Откройте PowerShell в папке проекта и выполните:

```powershell
cargo run --release
```

Для проверки форматирования, компиляции, тестов и Clippy:

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo test
cargo clippy --all-targets -- -D warnings
```

Ручная release-сборка:

```powershell
cargo build --release
```

Готовый бинарный файл будет находиться здесь:

- Windows: `target/release/giftwallpaper.exe`
- Linux: `target/release/giftwallpaper`

### Linux (Debian / Ubuntu)

Сначала установите системные зависимости:

```bash
./INSTALL_LINUX_DEPS.sh
```

Установите Rust с `https://rustup.rs/`, затем:

```bash
./RUN_LINUX.sh
```

Проверка:

```bash
./CHECK_LINUX.sh
```

Ручная release-сборка:

```bash
cargo build --release
```

## GitHub Releases

В репозитории уже есть `.github/workflows/release.yml`.

Создайте и отправьте тег версии:

```bash
git tag v0.2.2
git push origin v0.2.2
```

GitHub Actions соберёт:

- `giftwallpaper-windows-x64.zip`
- `giftwallpaper-linux-x64.tar.gz`

и добавит их в GitHub Release для этого тега.

Workflow также можно запустить вручную во вкладке **Actions**.

## Структура проекта

```text
giftwallpaper/
├── src/
│   ├── main.rs       # точка входа нативного приложения
│   ├── app.rs        # интерфейс egui и состояние приложения
│   ├── api.rs        # клиент api.changes.tg и обработка JSON
│   └── renderer.rs   # рендеринг обоев
├── .github/workflows/
│   ├── ci.yml
│   └── release.yml
├── CREDITS.md
├── CONTRIBUTING.md
├── LICENSE
└── Cargo.toml
```

## Источник данных

GiftWallpaper использует:

- `GET /gifts`
- `GET /gift/:gift`
- `GET /model/:gift/:model.png?size=1024`

На первом этапе интеграция с upstream API специально остаётся небольшой. Поддержку backdrops, symbols, оригинальных подарков, TGS-анимаций и custom emoji можно добавить позже.

## Конфиденциальность

GiftWallpaper не содержит аналитики или телеметрии. Запросы отправляются только к Gift Changes API, когда приложению нужно получить данные или ресурсы.

## Лицензия

MIT. См. файл [LICENSE](LICENSE).

## Благодарности

Спасибо **[@GiftChanges](https://t.me/GiftChanges)** <img width="20" height="20" alt="photo_2026-10-02_22-54-39" src="https://github.com/user-attachments/assets/08a73ffb-6aa0-4461-a99d-4aca4d9c7e4b" />за API с метаданными и ресурсами подарков на **api.changes.tg**.


## Open Source

Проект открыт для просмотра исходного кода и участия в разработке.

GitHub: https://github.com/Kolt5ik/GiftWallpaper
