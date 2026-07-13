# Sol SafeKey UI

This directory contains the integrated Next.js interface, local Rust API, and Tauri desktop shell for Sol SafeKey. The API uses the root workspace crate through a local path dependency; do not clone or vendor another copy of `sol-safekey` here.

## User guides

- [English UI wallet guide](../UI_USER_GUIDE.md)
- [中文 UI 钱包指南](../UI_USER_GUIDE_CN.md)

## Development

Run commands from the repository root:

```bash
make install
make ui-dev
```

Development endpoints:

- English UI: `http://127.0.0.1:3840/en/`
- Chinese UI: `http://127.0.0.1:3840/zh/`
- Local API: `http://127.0.0.1:3841/api/health`

Build and check the integrated application with:

```bash
make api-build
make desktop-build
make check
```

## Layout

```text
ui/src/             Next.js application and translations
ui/backend/         Local Axum API and wallet persistence
ui/src-tauri/       Tauri desktop shell and secure API bridge
ui/public/          Token and application assets
ui/scripts/         Development and security checks
```

Wallet data is stored locally in `ui/data/` by default and is ignored by Git. Passwords remain in memory only long enough to process each request and are never persisted by the frontend.
