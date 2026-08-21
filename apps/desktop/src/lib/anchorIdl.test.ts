import assert from "node:assert/strict";
import test from "node:test";
import {
  defaultAccountAddress,
  encodeAnchorSeedArgToBase64,
  encodeAnchorInstruction,
  isValidAnchorAccountAddress,
  resolveAnchorAccountAddress,
  type AnchorIdlInstruction,
} from "./anchorIdl";

const SYSTEM_PROGRAM_ID = "11111111111111111111111111111111";
const TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

test("resolves Anchor built-in account aliases", () => {
  assert.equal(defaultAccountAddress("system_program"), SYSTEM_PROGRAM_ID);
  assert.equal(defaultAccountAddress("systemProgram"), SYSTEM_PROGRAM_ID);
  assert.equal(resolveAnchorAccountAddress("system_program"), SYSTEM_PROGRAM_ID);
  assert.equal(resolveAnchorAccountAddress("", {
    name: "system_program",
    path: "system_program",
    isSigner: false,
    isWritable: false,
    address: "system_program",
  }), SYSTEM_PROGRAM_ID);
  assert.equal(isValidAnchorAccountAddress(SYSTEM_PROGRAM_ID), true);
});

test("encodes built-in account aliases as real public keys", async () => {
  const instruction: AnchorIdlInstruction = {
    name: "initialize",
    args: [],
    accounts: [
      {
        name: "system_program",
        writable: false,
        signer: false,
      },
    ],
  };

  const encoded = await encodeAnchorInstruction(
    TOKEN_PROGRAM_ID,
    instruction,
    {},
    { system_program: "system_program" },
  );

  assert.equal(encoded.accounts[0].pubkey, SYSTEM_PROGRAM_ID);
  assert.equal(encoded.accounts[0].is_signer, false);
  assert.equal(encoded.accounts[0].is_writable, false);
});

test("encodes Anchor arg PDA seeds using the instruction argument layout", () => {
  assert.equal(encodeAnchorSeedArgToBase64("u64", "1786805250"), "AnyAagAAAAA=");
});
