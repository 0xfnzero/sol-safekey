# Release v0.1.4

## Changes from v0.1.3

### Critical Bug Fixes

- **sol-trade-sdk v4.0.0 compatibility**: Fixed all compilation errors on macOS and other platforms caused by API changes in sol-trade-sdk v4.0.0:
  - Updated all `TradeConfig` initializations to use `swqos_cores_from_end` instead of the removed `use_core_affinity` field
  - Fixed all `TradeSellParams` initializations to use `wait_tx_confirmed` instead of the renamed `wait_transaction_confirmed` field
  - Affected files: `solana_ops.rs`, `pumpswap_sell.rs`, `pumpfun_sell.rs`, `operations.rs`, `cli.rs`

### Dependencies

- **sol-trade-sdk**: Upgraded from 3.6.4 to **4.0.0**

### Platform Compatibility

This release fixes the compilation errors reported on macOS (issue #1). The package now compiles successfully on all platforms supported by Solana and Rust.

No breaking API changes.
