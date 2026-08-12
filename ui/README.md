# Sol SafeKey UI

This directory contains the integrated Next.js interface, local Rust API, and Tauri desktop shell for Sol SafeKey. The API uses the root workspace crate through a local path dependency; do not clone or vendor another copy of `sol-safekey` here.

## User guides

- [English UI wallet guide](../UI_USER_GUIDE.md)
- [中文 UI 钱包指南](../UI_USER_GUIDE_CN.md)
- [General Solana Program deployment guide](PROGRAM_DEPLOYMENT.md)
- [通用 Solana Program 部署指南](PROGRAM_DEPLOYMENT_CN.md)

## Development

Prerequisites: Rust 1.89 or newer for workspace/API builds, and Node.js 20 or 22 and newer (Node.js 21 is unsupported). The release gate uses Node.js 20.19.5 and npm 10.8.2 exactly. The standalone Tauri crate declares its independently audited Rust 1.88 minimum, but repository-wide commands require Rust 1.89.

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
make release-check
```

`make release-check` is a non-deploying quality gate for the Sol SafeKey application
itself. Install `cargo-audit 0.22.1` before running it. Dependency audits may access
their advisory registries, but the gate does not audit or approve any user-supplied
Solana program or `.so`. It does not replace a program's security review,
reproducible SBF build, local-validator testing, or release approval, and it performs
no Solana RPC, signing, or deployment action.

## Layout

```text
ui/src/             Next.js application and translations
ui/backend/         Local Axum API and wallet persistence
ui/src-tauri/       Tauri desktop shell and secure API bridge
ui/public/          Token and application assets
ui/scripts/         Development and security checks
```

Wallet data is stored locally in `ui/data/` by default and is ignored by Git. Passwords remain in memory only long enough to process each request and are never persisted by the frontend.
