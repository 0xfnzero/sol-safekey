import { isLikelySolanaPublicKey } from "@/lib/programDeploy";

const BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const SYSTEM_PROGRAM_ID = "11111111111111111111111111111111";
const TOKEN_PROGRAM_ID = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const ASSOCIATED_TOKEN_PROGRAM_ID = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const SYSVAR_RENT_ID = "SysvarRent111111111111111111111111111111111";

export interface AnchorIdlArg {
  name: string;
  type: unknown;
}

export interface AnchorIdlAccount {
  name: string;
  writable?: boolean;
  mut?: boolean;
  signer?: boolean;
  isMut?: boolean;
  isSigner?: boolean;
  address?: string;
  accounts?: AnchorIdlAccount[];
}

export interface AnchorIdlInstruction {
  name: string;
  args: AnchorIdlArg[];
  accounts: AnchorIdlAccount[];
}

export interface AnchorIdlProgram {
  address?: string;
  metadata?: { address?: string; name?: string };
  name?: string;
  instructions: AnchorIdlInstruction[];
}

export interface FlatAnchorAccount {
  name: string;
  path: string;
  isSigner: boolean;
  isWritable: boolean;
  address?: string;
}

export interface EncodedAnchorInstruction {
  programId: string;
  accounts: Array<{ pubkey: string; is_signer: boolean; is_writable: boolean }>;
  dataBase64: string;
}

export function parseAnchorIdlJson(value: string): AnchorIdlProgram {
  const parsed = JSON.parse(value) as Partial<AnchorIdlProgram>;
  if (!parsed || typeof parsed !== "object" || !Array.isArray(parsed.instructions)) {
    throw new Error("invalid-idl");
  }
  return {
    ...parsed,
    instructions: parsed.instructions.filter((instruction): instruction is AnchorIdlInstruction =>
      Boolean(
        instruction &&
          typeof instruction.name === "string" &&
          Array.isArray(instruction.accounts) &&
          Array.isArray(instruction.args),
      ),
    ),
  } as AnchorIdlProgram;
}

export function anchorIdlProgramId(idl: AnchorIdlProgram): string {
  return String(idl.address || idl.metadata?.address || "").trim();
}

export function flattenAnchorAccounts(
  accounts: AnchorIdlAccount[],
  prefix = "",
): FlatAnchorAccount[] {
  const flattened: FlatAnchorAccount[] = [];
  for (const account of accounts) {
    const path = prefix ? `${prefix}.${account.name}` : account.name;
    if (Array.isArray(account.accounts)) {
      flattened.push(...flattenAnchorAccounts(account.accounts, path));
      continue;
    }
    flattened.push({
      name: account.name,
      path,
      isSigner: account.signer === true || account.isSigner === true,
      isWritable: account.writable === true || account.mut === true || account.isMut === true,
      address: account.address,
    });
  }
  return flattened;
}

export function defaultAccountAddress(accountName: string, idlAccount?: FlatAnchorAccount): string {
  if (idlAccount?.address && isLikelySolanaPublicKey(idlAccount.address)) return idlAccount.address;
  const normalized = accountName.replace(/[_\s-]/g, "").toLowerCase();
  if (normalized === "systemprogram") return SYSTEM_PROGRAM_ID;
  if (normalized === "tokenprogram") return TOKEN_PROGRAM_ID;
  if (normalized === "associatedtokenprogram") return ASSOCIATED_TOKEN_PROGRAM_ID;
  if (normalized === "rent") return SYSVAR_RENT_ID;
  return "";
}

export function idlTypeLabel(type: unknown): string {
  if (typeof type === "string") return type;
  if (type && typeof type === "object") {
    const record = type as Record<string, unknown>;
    if (record.option !== undefined) return `option<${idlTypeLabel(record.option)}>`;
    if (record.vec !== undefined) return `vec<${idlTypeLabel(record.vec)}>`;
    if (record.defined !== undefined) return `defined<${String(record.defined)}>`;
    if (record.array !== undefined) return `array`;
  }
  return "json";
}

export function isUnsupportedIdlType(type: unknown): boolean {
  if (typeof type === "string") {
    return ![
      "bool",
      "u8",
      "i8",
      "u16",
      "i16",
      "u32",
      "i32",
      "u64",
      "i64",
      "u128",
      "i128",
      "string",
      "pubkey",
      "publicKey",
      "bytes",
    ].includes(type);
  }
  if (type && typeof type === "object") {
    const record = type as Record<string, unknown>;
    if (record.option !== undefined) return isUnsupportedIdlType(record.option);
  }
  return true;
}

async function sha256(bytes: Uint8Array): Promise<Uint8Array> {
  const copy = new Uint8Array(bytes.length);
  copy.set(bytes);
  const digest = await crypto.subtle.digest("SHA-256", copy.buffer as ArrayBuffer);
  return new Uint8Array(digest);
}

function utf8(value: string): Uint8Array {
  return new TextEncoder().encode(value);
}

function concatBytes(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const result = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  return result;
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function decodeBase58(value: string): Uint8Array {
  const input = value.trim();
  if (!input) throw new Error("invalid-pubkey");
  let leadingZeroes = 0;
  while (leadingZeroes < input.length && input[leadingZeroes] === BASE58_ALPHABET[0]) {
    leadingZeroes += 1;
  }
  const bytes: number[] = [];
  for (let i = leadingZeroes; i < input.length; i += 1) {
    const value = BASE58_ALPHABET.indexOf(input[i]);
    if (value < 0) throw new Error("invalid-pubkey");
    let carry = value;
    for (let j = 0; j < bytes.length; j += 1) {
      carry += bytes[j] * 58;
      bytes[j] = carry & 0xff;
      carry >>= 8;
    }
    while (carry > 0) {
      bytes.push(carry & 0xff);
      carry >>= 8;
    }
  }
  const result = new Uint8Array(leadingZeroes + bytes.length);
  for (let i = 0; i < leadingZeroes; i += 1) result[i] = 0;
  for (let i = 0; i < bytes.length; i += 1) result[result.length - 1 - i] = bytes[i];
  return result;
}

function encodeInteger(value: string, byteLength: number, signed: boolean): Uint8Array {
  const trimmed = value.trim();
  if (!/^-?\d+$/.test(trimmed)) throw new Error("invalid-integer");
  let bigint = BigInt(trimmed);
  const bits = BigInt(byteLength * 8);
  const max = signed ? (BigInt(1) << (bits - BigInt(1))) - BigInt(1) : (BigInt(1) << bits) - BigInt(1);
  const min = signed ? -(BigInt(1) << (bits - BigInt(1))) : BigInt(0);
  if (bigint < min || bigint > max) throw new Error("integer-out-of-range");
  if (bigint < BigInt(0)) bigint = (BigInt(1) << bits) + bigint;
  const result = new Uint8Array(byteLength);
  for (let i = 0; i < byteLength; i += 1) {
    result[i] = Number((bigint >> BigInt(i * 8)) & BigInt(0xff));
  }
  return result;
}

function encodeU32Length(length: number): Uint8Array {
  if (!Number.isSafeInteger(length) || length < 0 || length > 0xffff_ffff) {
    throw new Error("invalid-length");
  }
  const result = new Uint8Array(4);
  result[0] = length & 0xff;
  result[1] = (length >> 8) & 0xff;
  result[2] = (length >> 16) & 0xff;
  result[3] = (length >> 24) & 0xff;
  return result;
}

function encodeArgValue(type: unknown, rawValue: string): Uint8Array {
  if (type && typeof type === "object" && "option" in (type as Record<string, unknown>)) {
    const inner = (type as Record<string, unknown>).option;
    if (!rawValue.trim()) return new Uint8Array([0]);
    return concatBytes([new Uint8Array([1]), encodeArgValue(inner, rawValue)]);
  }
  switch (type) {
    case "bool": {
      const normalized = rawValue.trim().toLowerCase();
      if (normalized === "true" || normalized === "1") return new Uint8Array([1]);
      if (normalized === "false" || normalized === "0") return new Uint8Array([0]);
      throw new Error("invalid-bool");
    }
    case "u8":
      return encodeInteger(rawValue, 1, false);
    case "i8":
      return encodeInteger(rawValue, 1, true);
    case "u16":
      return encodeInteger(rawValue, 2, false);
    case "i16":
      return encodeInteger(rawValue, 2, true);
    case "u32":
      return encodeInteger(rawValue, 4, false);
    case "i32":
      return encodeInteger(rawValue, 4, true);
    case "u64":
      return encodeInteger(rawValue, 8, false);
    case "i64":
      return encodeInteger(rawValue, 8, true);
    case "u128":
      return encodeInteger(rawValue, 16, false);
    case "i128":
      return encodeInteger(rawValue, 16, true);
    case "string": {
      const bytes = utf8(rawValue);
      return concatBytes([encodeU32Length(bytes.length), bytes]);
    }
    case "pubkey":
    case "publicKey": {
      const bytes = decodeBase58(rawValue);
      if (bytes.length !== 32) throw new Error("invalid-pubkey");
      return bytes;
    }
    case "bytes": {
      const compact = rawValue.trim().replace(/^0x/, "");
      if (compact.length % 2 !== 0 || !/^[a-fA-F0-9]*$/.test(compact)) throw new Error("invalid-bytes");
      const bytes = new Uint8Array(compact.length / 2);
      for (let i = 0; i < bytes.length; i += 1) {
        bytes[i] = Number.parseInt(compact.slice(i * 2, i * 2 + 2), 16);
      }
      return concatBytes([encodeU32Length(bytes.length), bytes]);
    }
    default:
      throw new Error("unsupported-idl-type");
  }
}

export async function encodeAnchorInstruction(
  programId: string,
  instruction: AnchorIdlInstruction,
  argValues: Record<string, string>,
  accountValues: Record<string, string>,
): Promise<EncodedAnchorInstruction> {
  if (!isLikelySolanaPublicKey(programId)) throw new Error("invalid-program-id");
  const discriminator = (await sha256(utf8(`global:${instruction.name}`))).subarray(0, 8);
  const encodedArgs = instruction.args.map((arg) =>
    encodeArgValue(arg.type, String(argValues[arg.name] ?? "")),
  );
  const flatAccounts = flattenAnchorAccounts(instruction.accounts);
  const accounts = flatAccounts.map((account) => {
    const pubkey = String(accountValues[account.path] || account.address || "").trim();
    if (!isLikelySolanaPublicKey(pubkey)) throw new Error(`invalid-account:${account.path}`);
    return {
      pubkey,
      is_signer: account.isSigner,
      is_writable: account.isWritable,
    };
  });
  return {
    programId,
    accounts,
    dataBase64: bytesToBase64(concatBytes([discriminator, ...encodedArgs])),
  };
}
