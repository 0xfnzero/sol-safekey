import assert from "node:assert/strict";
import test from "node:test";
import {
  currentNetwork,
  DEFAULT_RPC_PROFILES,
  mergeRpcProfiles,
  rpcProfileKey,
  rpcRequestValue,
  validateRpcUrl,
  type RpcProfile,
} from "./appStorage";

test("normalizes network values to the supported cluster set", () => {
  assert.equal(currentNetwork("devnet"), "devnet");
  assert.equal(currentNetwork("testnet"), "testnet");
  assert.equal(currentNetwork("mainnet"), "mainnet");
  assert.equal(currentNetwork("custom"), "mainnet");
  assert.equal(currentNetwork(undefined), "mainnet");
});

test("validates RPC URLs with explicit http or https origins", () => {
  assert.equal(validateRpcUrl(" https://api.devnet.solana.com/ "), "https://api.devnet.solana.com");
  assert.equal(validateRpcUrl("http://127.0.0.1:8899"), "http://127.0.0.1:8899");
  assert.equal(validateRpcUrl("ftp://example.com"), null);
  assert.equal(validateRpcUrl("https://user:pass@example.com"), null);
  assert.equal(validateRpcUrl("https://example.com#fragment"), null);
  assert.equal(validateRpcUrl("https://.example.com"), null);
  assert.equal(validateRpcUrl("https://example.com."), null);
  assert.equal(validateRpcUrl("not-a-url"), null);
});

test("merges custom RPC profiles without duplicating built-in endpoints", () => {
  const duplicateBuiltin: RpcProfile = {
    id: "custom-duplicate",
    name: "Duplicate",
    url: DEFAULT_RPC_PROFILES[0].url,
    network: DEFAULT_RPC_PROFILES[0].network,
  };
  const custom: RpcProfile = {
    id: "custom-dev",
    name: "Custom Devnet",
    url: "https://example.invalid/rpc",
    network: "devnet",
  };
  const merged = mergeRpcProfiles([duplicateBuiltin, custom]);

  assert.equal(merged.filter((profile) => rpcProfileKey(profile) === rpcProfileKey(duplicateBuiltin)).length, 1);
  assert.equal(merged.some((profile) => profile.id === custom.id), true);
  assert.equal(rpcRequestValue(custom), "rpc:devnet:https%3A%2F%2Fexample.invalid%2Frpc");
});
