import assert from "node:assert/strict";
import test from "node:test";
import { normalizeApiPath } from "./apiPath";

test("normalizes safe API paths", () => {
  assert.equal(normalizeApiPath(" wallets "), "wallets");
  assert.equal(normalizeApiPath("/wallets/abc-123.export"), "wallets/abc-123.export");
});

test("rejects unsafe API paths", () => {
  for (const path of [
    "",
    "   ",
    "https://example.com/api",
    "wallets\\secret",
    "wallets/../secret",
    "wallets/%2e%2e/secret",
    "wallets?x=1",
  ]) {
    assert.throws(() => normalizeApiPath(path), /Invalid API path/);
  }
});
