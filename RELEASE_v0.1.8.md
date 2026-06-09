# Release v0.1.8

## Changes from v0.1.7

### Dependency Updates

- **sol-trade-sdk**: Upgraded from 4.0.3 to **4.0.20**.
- **openssl**: Upgraded from 0.10.73 to **0.10.80** to keep full-feature binary builds working with the current dependency set.

### Compatibility Fixes

- Updated Pump.fun and PumpSwap sell flows for the newer `TradeSellParams` API by setting `wait_for_all_submits` explicitly.
- Kept existing behavior unchanged by using the SDK default value `false` while retaining `wait_tx_confirmed: true`.

### Validation

- `cargo check --no-default-features`
- `cargo check --features sol-trade-sdk`
- `cargo check --features full --all-targets`
- `cargo run --features full -- --help`
