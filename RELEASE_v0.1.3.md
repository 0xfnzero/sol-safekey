# Release v0.1.3

## Changes from v0.1.2

### Fixes and cleanup

- **Build warnings**: Resolved all compiler warnings. Imports and items used only under optional features (`solana-ops`, `sol-trade-sdk`) are now gated with `#[cfg(...)]`, so default builds are warning-free.
- **TradeConfig compatibility**: Removed references to the non-existent `use_core_affinity` field in `TradeConfig` (sol-trade-sdk 3.6.x) across `operations.rs`, `solana_ops.rs`, `pumpswap_sell.rs`, and `pumpfun_sell.rs`.
- **Signer trait**: Restored and cfg-gated `Signer` import where `keypair.pubkey()` is used so that builds with `solana-ops` or `sol-trade-sdk` compile correctly.
- **sol-trade-sdk**: Optional dependency upgraded from 3.5.7 to **3.6.4**.
- **sol-trade-sdk feature**: Added optional `solana-client` to the `sol-trade-sdk` feature so that cashback flows (Pump.fun / PumpSwap) can use `RpcClient` when only `sol-trade-sdk` is enabled.

### Documentation

- Version references in README.md and README_CN.md updated to 0.1.3.

No breaking API changes.
