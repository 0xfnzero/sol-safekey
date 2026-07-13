"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { toast } from "sonner";
import { useTranslations } from '@/hooks/useTranslations';
import {
  Key,
  Lock,
  Download,
  Upload,
  Wallet,
  ArrowLeft,
  ChevronRight,
  ChevronDown,
  Copy,
  Check,
  RefreshCw,
  Hash,
  Coins,
  Plus,
  ArrowRightLeft,
  Send,
  Unlock,
  X,
  ShieldCheck,
  Pencil,
  Save,
  Settings,
  Trash2,
  ExternalLink,
  Menu,
} from "lucide-react";
import LanguageSwitcher from '@/components/LanguageSwitcher';
import { SavedWalletPicker } from "@/components/SavedWalletPicker";
import { DEFAULT_API_PORT } from "@/lib/api";
import { apiFetch } from "@/lib/apiFetch";
import { localTokenMetadata } from "@/lib/localTokenRegistry";
import {
  applyWalletAuth,
  authFormsWithWallets,
  fetchWallets,
  normalizeWalletAuth,
  type ApiRequestBody,
  type FormState,
  type SavedWallet,
  type WalletAuthTab,
  validateWalletAuth,
} from "@/lib/walletAuth";

const MAX_KEYSTORE_FILE_BYTES = 128 * 1024;
const MAX_PROGRAM_SO_FILE_BYTES = 3 * 1024 * 1024;
const DEFAULT_NETWORK = "mainnet";
const NETWORK_STORAGE_KEY = "sol-safekey-network-v1";
const RPC_PROFILES_STORAGE_KEY = "sol-safekey-rpc-profiles-v1";
const CURRENT_WALLET_STORAGE_KEY = "sol-safekey-current-wallet-v1";
const WORKSPACE_STORAGE_KEY = "sol-safekey-squads-workspace-v1";
const TRANSACTION_PAGE_SIZE = 20;
const MAX_TRANSACTION_HISTORY = 100;
const ASSET_AUTO_REFRESH_TTL_MS = 60_000;
const WALLET_ASSET_AUTO_REFRESH_INTERVAL_MS = 30_000;
const ACTIVE_OPERATION_ASSET_REFRESH_INTERVAL_MS = 12_000;
const TRANSACTION_AUTO_REFRESH_TTL_MS = 90_000;
const NONCE_AUTO_REFRESH_TTL_MS = 120_000;
const TOKEN_ASSET_PAGE_SIZE = 30;
const DEFAULT_POST_MUTATION_ASSET_REFRESH_DELAYS_MS = [0, 1800];
const POST_SELL_ASSET_REFRESH_DELAYS_MS = [0, 1200, 3000, 7000, 12000];
const WRAPPED_SOL_MINT = "So11111111111111111111111111111111111111112";
const SOLANA_TOKEN_LOGO_URI = "/token-icons/solana.png";
const ALLOW_DIRECT_SECRET_INPUT =
  process.env.NEXT_PUBLIC_ALLOW_DIRECT_SECRET_INPUT === "true";

const WALLET_PASSWORD_FORM_IDS = new Set([
  "decrypt",
  "unlock",
  "get-pubkey",
  "transfer-sol",
  "transfer-token",
  "create-wsol-ata",
  "wrap-sol",
  "unwrap-sol",
  "close-wsol-ata",
  "create-nonce",
  "program-deploy",
  "squads-create",
  "squads-sol-transfer",
  "squads-token-transfer",
  "squads-prepare-upgrade-buffer",
  "squads-program-upgrade",
  "squads-set-authority",
  "squads-approve",
  "squads-reject",
  "squads-execute",
  "pumpfun-sell",
  "pumpswap-sell",
  "create-tfa",
  "pumpfun-cashback",
  "pumpswap-cashback",
]);

const CREATE_PASSWORD_FORM_IDS = new Set([
  "create-encrypted",
  "create-keystore",
  "import-keystore",
]);

const MASTER_PASSWORD_FORM_IDS = new Set([
  "setup-2fa",
  "create-tfa",
  "unlock-tfa",
]);

const TOKEN_ACTION_FORM_IDS = new Set(["pumpfun-sell", "transfer-token"]);
const CURRENT_WALLET_TOKEN_BALANCE_FORM_IDS = new Set(["transfer-token", "pumpfun-sell", "pumpswap-sell"]);

type AppNetwork = "mainnet" | "devnet" | "testnet";
type RpcNetwork = AppNetwork;

interface RpcProfile {
  id: string;
  name: string;
  url: string;
  network: RpcNetwork;
  builtin?: boolean;
}

const DEFAULT_RPC_PROFILES: RpcProfile[] = [
  {
    id: "solana-mainnet",
    name: "Solana Mainnet",
    url: "https://api.mainnet-beta.solana.com",
    network: "mainnet",
    builtin: true,
  },
  {
    id: "publicnode-mainnet",
    name: "PublicNode Mainnet",
    url: "https://solana.publicnode.com",
    network: "mainnet",
    builtin: true,
  },
  {
    id: "solana-devnet",
    name: "Solana Devnet",
    url: "https://api.devnet.solana.com",
    network: "devnet",
    builtin: true,
  },
  {
    id: "solana-testnet",
    name: "Solana Testnet",
    url: "https://api.testnet.solana.com",
    network: "testnet",
    builtin: true,
  },
  {
    id: "publicnode-testnet",
    name: "PublicNode Testnet",
    url: "https://solana-testnet-rpc.publicnode.com",
    network: "testnet",
    builtin: true,
  },
];

interface MenuItem {
  id: string;
  label: string;
  icon: React.ReactNode;
  children?: MenuItem[];
  network?: boolean;
}

interface WorkspaceMultisig {
  address: string;
  vault?: string;
  label?: string;
  createdBy?: string;
  createdByLabel?: string;
  network: AppNetwork;
  updatedAt: number;
}

interface WorkspaceProgram {
  address: string;
  label?: string;
  network: AppNetwork;
  updatedAt: number;
}

interface WorkspaceProposal {
  address: string;
  multisig: string;
  transactionIndex: string;
  status?: string;
  kind?: string;
  createdBy?: string;
  createdByLabel?: string;
  network: AppNetwork;
  updatedAt: number;
}

type WorkspaceProposalAction = "approve" | "reject" | "execute";
type PasswordPromptField = "password" | "master_password";

type PasswordPromptRequest =
  | { kind: "form"; formId: string; formState: FormState; fields?: PasswordPromptField[] }
  | { kind: "create-password"; formId: string; formState: FormState }
  | { kind: "master-password"; formId: string; formState: FormState }
  | { kind: "proposal"; proposal: WorkspaceProposal; action: WorkspaceProposalAction; formState: FormState }
  | { kind: "export-keystore"; wallet: SavedWallet; formState: FormState }
  | { kind: "export-private-key"; wallet: SavedWallet; formState: FormState };

interface WalletTokenAsset {
  account: string;
  mint: string;
  amount: string;
  ui_amount_string: string;
  decimals: number;
  name?: string;
  symbol?: string;
  logo_uri?: string;
}

interface WalletAssetsState {
  address: string;
  network: AppNetwork;
  solBalance: string;
  tokens: WalletTokenAsset[];
  loading: boolean;
  refreshing?: boolean;
  cached?: boolean;
  updatedAt?: number;
  error?: string;
}

interface WalletTransactionRecord {
  signature: string;
  slot: number;
  block_time?: number | null;
  confirmation_status?: string | null;
  err?: unknown;
  memo?: string | null;
  action?: string;
  summary?: string;
  counterparty?: string | null;
  programs?: string[];
  changes?: WalletTransactionChange[];
}

interface WalletTransactionChange {
  asset: string;
  mint?: string | null;
  amount: string;
  ui_amount: string;
  direction: "in" | "out";
  decimals: number;
}

interface WalletTransactionsState {
  address: string;
  network: AppNetwork;
  transactions: WalletTransactionRecord[];
  nextBefore?: string;
  hasMore: boolean;
  loaded: number;
  loading: boolean;
  error?: string;
}

interface NonceAccountRecord {
  id: string;
  wallet_id?: string;
  owner: string;
  network: AppNetwork;
  nonce_account: string;
  signature: string;
  created_at: number;
}

interface NonceAccountsState {
  owner: string;
  network: AppNetwork;
  nonceAccounts: NonceAccountRecord[];
  loading: boolean;
  error?: string;
}

interface CreatedNonceAccount {
  nonce_account?: string;
  signature?: string;
}

interface TokenMintInfoState {
  mint: string;
  network: AppNetwork;
  decimals?: number;
  supply?: string;
  uiAmountString?: string;
  loading: boolean;
  error?: string;
}

interface CashbackInfoState {
  owner: string;
  network: AppNetwork;
  dex: "pumpfun" | "pumpswap";
  accumulator?: string;
  amountLamports?: number;
  uiAmountString?: string;
  asset?: string;
  available?: boolean;
  loading: boolean;
  error?: string;
}

interface WorkspaceActor {
  createdBy?: string;
  createdByLabel?: string;
}

interface SquadsWorkspace {
  multisigs: WorkspaceMultisig[];
  programs: WorkspaceProgram[];
  proposals: WorkspaceProposal[];
}

const emptyWorkspace: SquadsWorkspace = {
  multisigs: [],
  programs: [],
  proposals: [],
};

function currentNetwork(value: string | number | undefined): AppNetwork {
  return value === "devnet" || value === "testnet" ? value : "mainnet";
}

function loadStoredNetwork(): AppNetwork {
  if (typeof window === "undefined") return DEFAULT_NETWORK;
  return currentNetwork(window.localStorage.getItem(NETWORK_STORAGE_KEY) || DEFAULT_NETWORK);
}

function rpcProfileKey(profile: Pick<RpcProfile, "url" | "network">): string {
  return `${profile.network}:${profile.url.trim()}`;
}

function sanitizeRpcProfile(raw: Partial<RpcProfile> | undefined): RpcProfile | null {
  const id = String(raw?.id ?? "").trim();
  const name = String(raw?.name ?? "").trim();
  const url = validateRpcUrl(String(raw?.url ?? ""));
  const network = currentNetwork(raw?.network);
  if (!id || !name || !url) return null;
  return { id, name, url, network, builtin: raw?.builtin === true };
}

function mergeRpcProfiles(customProfiles: RpcProfile[]): RpcProfile[] {
  const profiles: RpcProfile[] = [];
  const seen = new Set<string>();
  for (const profile of [...DEFAULT_RPC_PROFILES, ...customProfiles]) {
    const key = rpcProfileKey(profile);
    if (seen.has(key)) continue;
    seen.add(key);
    profiles.push(profile);
  }
  return profiles;
}

function loadStoredRpcProfiles(): RpcProfile[] {
  if (typeof window === "undefined") return DEFAULT_RPC_PROFILES;
  try {
    const raw = window.localStorage.getItem(RPC_PROFILES_STORAGE_KEY);
    if (!raw) return DEFAULT_RPC_PROFILES;
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return DEFAULT_RPC_PROFILES;
    const customProfiles = parsed
      .map((item) => sanitizeRpcProfile(item as Partial<RpcProfile>))
      .filter((item): item is RpcProfile => Boolean(item))
      .filter((item) => !item.builtin);
    return mergeRpcProfiles(customProfiles);
  } catch {
    return DEFAULT_RPC_PROFILES;
  }
}

function saveCustomRpcProfiles(profiles: RpcProfile[]): void {
  if (typeof window === "undefined") return;
  window.localStorage.setItem(
    RPC_PROFILES_STORAGE_KEY,
    JSON.stringify(profiles.filter((profile) => !profile.builtin)),
  );
}

function defaultRpcProfileId(network: AppNetwork): string {
  return DEFAULT_RPC_PROFILES.find((profile) => profile.network === network)?.id || DEFAULT_RPC_PROFILES[0].id;
}

function normalizeStoredRpcSelection(rpcProfiles: RpcProfile[], storedNetwork: AppNetwork): string {
  if (typeof window !== "undefined") {
    const storedValue = window.localStorage.getItem(NETWORK_STORAGE_KEY) || "";
    if (rpcProfiles.some((profile) => profile.id === storedValue)) {
      return storedValue;
    }
  }
  return defaultRpcProfileId(storedNetwork);
}

function validateRpcUrl(value: string): string | null {
  const url = value.trim();
  if (!url) return null;
  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:" ? parsed.toString().replace(/\/$/, "") : null;
  } catch {
    return null;
  }
}

function rpcRequestValue(profile: RpcProfile | undefined): string {
  if (!profile) return DEFAULT_NETWORK;
  return `rpc:${profile.network}:${encodeURIComponent(profile.url)}`;
}

function networkLabel(t: (key: string, vars?: Record<string, string | number>) => string, network: AppNetwork): string {
  return t(`features.check-balance.${network}`);
}

function loadStoredWalletId(): string {
  if (typeof window === "undefined") return "";
  return window.localStorage.getItem(CURRENT_WALLET_STORAGE_KEY) || "";
}

function loadInitialRpcState(): { profiles: RpcProfile[]; selectedId: string } {
  const profiles = loadStoredRpcProfiles();
  return {
    profiles,
    selectedId: normalizeStoredRpcSelection(profiles, loadStoredNetwork()),
  };
}

function shortAddress(value: string): string {
  return value.length > 12 ? `${value.slice(0, 4)}...${value.slice(-4)}` : value;
}

function shortSignature(value: string): string {
  return value.length > 20 ? `${value.slice(0, 8)}...${value.slice(-8)}` : value;
}

function solscanTransactionUrl(signature: string, network: AppNetwork): string {
  const baseUrl = `https://solscan.io/tx/${encodeURIComponent(signature)}`;
  return network === "mainnet" ? baseUrl : `${baseUrl}?cluster=${network}`;
}

function formatTransactionDate(value: number | null | undefined): string {
  if (!value) return "Unknown Date";
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value * 1000));
}

function formatTransactionTime(value: number | null | undefined): string {
  if (!value) return "";
  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value * 1000));
}

function transactionAssetLabel(change: WalletTransactionChange, tokenSymbols?: Map<string, string>): string {
  if (change.asset === "SOL") return "SOL";
  const mint = change.mint?.trim();
  const localMetadata = localTokenMetadata(mint);
  if (localMetadata) return localMetadata.symbol;
  if (change.asset === WRAPPED_SOL_MINT) return "WSOL";
  if (mint) {
    const symbol = tokenSymbols?.get(mint)?.trim();
    if (symbol) return symbol;
  }
  return change.asset.length > 10 ? `${change.asset.slice(0, 4)}...${change.asset.slice(-4)}` : change.asset;
}

function transactionAmountLabel(change: WalletTransactionChange, tokenSymbols?: Map<string, string>): string {
  return `${change.ui_amount} ${transactionAssetLabel(change, tokenSymbols)}`;
}

function workspaceLabel(label: string | undefined, address: string): string {
  return label?.trim() || shortAddress(address);
}

function walletActor(wallet: SavedWallet | undefined): WorkspaceActor {
  if (!wallet) return {};
  return {
    createdBy: wallet.public_key,
    createdByLabel: wallet.name,
  };
}

function actorLabel(item: WorkspaceActor): string {
  return item.createdByLabel?.trim() || (item.createdBy ? shortAddress(item.createdBy) : "");
}

function walletAvatarText(wallet: SavedWallet | undefined): string {
  if (!wallet) return "?";
  const name = wallet.name.trim();
  return (name || wallet.public_key).slice(0, 2).toUpperCase();
}

function tokenDisplayName(token: WalletTokenAsset): string {
  const localMetadata = localTokenMetadata(token.mint);
  if (localMetadata) return localMetadata.name;
  return token.name?.trim() || token.symbol?.trim() || shortAddress(token.mint);
}

function tokenMetadataName(token: WalletTokenAsset): string | undefined {
  const localMetadata = localTokenMetadata(token.mint);
  return localMetadata?.name.trim() || token.name?.trim() || undefined;
}

function tokenDisplaySymbol(token: WalletTokenAsset): string {
  const localMetadata = localTokenMetadata(token.mint);
  if (localMetadata) return localMetadata.symbol;
  return token.symbol?.trim() || shortAddress(token.mint);
}

function tokenAvatarText(token: WalletTokenAsset): string {
  const source = tokenDisplaySymbol(token) || tokenDisplayName(token) || token.mint;
  return source.slice(0, 2).toUpperCase();
}

function tokenLogoUri(token: WalletTokenAsset): string | undefined {
  const localMetadata = localTokenMetadata(token.mint);
  if (localMetadata) return localMetadata.logoUri;
  return token.logo_uri?.trim() || undefined;
}

function tokenColorPair(value: string): { from: string; to: string } {
  const palettes = [
    { from: "#7c3aed", to: "#06b6d4" },
    { from: "#059669", to: "#f59e0b" },
    { from: "#dc2626", to: "#f97316" },
    { from: "#2563eb", to: "#a855f7" },
    { from: "#0f766e", to: "#84cc16" },
    { from: "#be123c", to: "#ec4899" },
  ];
  let hash = 0;
  for (let i = 0; i < value.length; i += 1) {
    hash = (hash * 31 + value.charCodeAt(i)) >>> 0;
  }
  return palettes[hash % palettes.length];
}

function cleanOptionalTokenText(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function mergeTokenAssetMetadata(nextTokens: WalletTokenAsset[], previousTokens: WalletTokenAsset[]): WalletTokenAsset[] {
  const previousByAccount = new Map(previousTokens.map((token) => [token.account, token]));
  const previousByMint = new Map(previousTokens.map((token) => [token.mint, token]));
  return nextTokens.map((token) => {
    const previous = previousByAccount.get(token.account) || previousByMint.get(token.mint);
    const localMetadata = localTokenMetadata(token.mint);
    if (!previous && !localMetadata) return token;
    return {
      ...token,
      name: localMetadata?.name || cleanOptionalTokenText(token.name) || cleanOptionalTokenText(previous?.name),
      symbol: localMetadata?.symbol || cleanOptionalTokenText(token.symbol) || cleanOptionalTokenText(previous?.symbol),
      logo_uri: localMetadata?.logoUri || cleanOptionalTokenText(token.logo_uri) || cleanOptionalTokenText(previous?.logo_uri),
    };
  });
}

function uniqueAddressList(items: Array<string | number | undefined>): string[] {
  const seen = new Set<string>();
  const addresses: string[] = [];
  for (const item of items) {
    for (const address of parseAddressList(item)) {
      if (!seen.has(address)) {
        seen.add(address);
        addresses.push(address);
      }
    }
  }
  return addresses;
}

function safeFilename(value: string): string {
  return value.trim().replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "") || "wallet";
}

function defaultBackTarget(formId: string): string | null {
  switch (formId) {
    case "create-encrypted":
    case "create-keystore":
    case "import-keystore":
    case "decrypt":
      return "wallet-list";
    case "unlock":
    case "check-balance":
    case "get-pubkey":
    case "setup-2fa":
    case "create-tfa":
    case "unlock-tfa":
      return "wallet-list";
    case "transfer-sol":
    case "transfer-token":
      return "wallet-list";
    case "create-wsol-ata":
    case "wrap-sol":
    case "unwrap-sol":
    case "close-wsol-ata":
      return "wsol-workbench";
    case "pumpfun-sell":
    case "pumpswap-sell":
    case "pumpfun-cashback":
    case "pumpswap-cashback":
      return "pump-workbench";
    case "program-deploy":
    case "program-info":
      return "program-workbench";
    case "create-nonce":
      return "nonce-workbench";
    case "squads-proposals":
    case "squads-programs":
    case "squads-create":
    case "squads-info":
    case "squads-sol-transfer":
    case "squads-token-transfer":
    case "squads-prepare-upgrade-buffer":
    case "squads-program-upgrade":
    case "squads-set-authority":
    case "squads-approve":
    case "squads-reject":
    case "squads-execute":
      return "squads-workspace";
    default:
      return null;
  }
}

function keystoreMetadataName(keystoreJson: string): string | undefined {
  try {
    const parsed = JSON.parse(keystoreJson) as {
      metadata?: Record<string, unknown>;
    };
    const value = parsed.metadata?.name;
    return typeof value === "string" && value.trim() ? value.trim() : undefined;
  } catch {
    return undefined;
  }
}

function loadWorkspace(): SquadsWorkspace {
  if (typeof window === "undefined") return emptyWorkspace;
  try {
    const raw = window.localStorage.getItem(WORKSPACE_STORAGE_KEY);
    if (!raw) return emptyWorkspace;
    const parsed = JSON.parse(raw) as Partial<SquadsWorkspace>;
    return {
      multisigs: Array.isArray(parsed.multisigs) ? parsed.multisigs : [],
      programs: Array.isArray(parsed.programs) ? parsed.programs : [],
      proposals: Array.isArray(parsed.proposals) ? parsed.proposals : [],
    };
  } catch {
    return emptyWorkspace;
  }
}

/** 滑点百分比（如 1 表示 1%）→ 后端 basis points（×100），空则默认 1% */
function slippagePercentToBasisPoints(v: string | number | undefined): number {
  if (v === undefined || v === "") return 100;
  const n = typeof v === "number" ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return 100;
  return Math.floor(Math.min(Math.max(n, 0), 100) * 100);
}

function slippageInputDisplay(v: string | number | undefined): string {
  if (v === undefined || v === "") return "1";
  return String(v);
}

function parsePositiveDecimal(value: string | number | undefined): string | null {
  if (value === undefined || value === "") return null;
  const text = String(value).trim();
  if (!/^(?:\d+|\d*\.\d+)$/.test(text)) return null;
  if (!/[1-9]/.test(text)) return null;
  return text;
}

function parseNonceCount(value: string | number | undefined): number {
  const raw = String(value ?? "1").trim();
  const count = Number(raw);
  if (!Number.isInteger(count) || count < 1) return 1;
  return Math.min(count, 20);
}

function parseSellPercentBps(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isFinite(value)) return undefined;
  const normalized = Math.floor(value);
  return normalized >= 1 && normalized <= 10_000 ? normalized : undefined;
}

function hasValidSellAmount(formState: FormState): boolean {
  return parsePositiveDecimal(formState.amount) !== null || parseSellPercentBps(formState.sell_percent) !== undefined;
}

type TokenBalanceSnapshot = { amount: string; rawAmount: string; decimals: number };
type TokenBalanceAdjustment = {
  mint: string;
  sourceAccount?: string;
  soldRawAmount: string;
  decimals: number;
};
type PendingTokenBalanceAdjustment = TokenBalanceAdjustment & {
  expectedRawAmount: string;
  createdAt: number;
};

function rawTokenAmountToUi(raw: string, decimals: number): string {
  if (!/^\d+$/.test(raw) || !Number.isInteger(decimals) || decimals < 0) return "";
  const padded = raw.padStart(decimals + 1, "0");
  const whole = decimals > 0 ? padded.slice(0, -decimals) : padded;
  const fractional = decimals > 0 ? padded.slice(-decimals).replace(/0+$/, "") : "";
  return fractional ? `${whole}.${fractional}` : whole;
}

function uiTokenAmountToRaw(value: string | number | undefined, decimals: number): string | null {
  const text = parsePositiveDecimal(value);
  if (text === null || !Number.isInteger(decimals) || decimals < 0) return null;
  const [whole, fractional = ""] = text.split(".");
  if (fractional.length > decimals) return null;
  const raw = `${whole}${fractional.padEnd(decimals, "0")}`.replace(/^0+(?=\d)/, "");
  return raw || "0";
}

function tokenBalanceSnapshotFromFormState(formState: FormState | null, mint: string): TokenBalanceSnapshot | null {
  if (!formState) return null;
  const formMint = String(formState.mint ?? "").trim();
  if (formMint && formMint !== mint) return null;
  const decimals =
    typeof formState.decimals === "number"
      ? formState.decimals
      : Number.parseInt(String(formState.decimals ?? ""), 10);
  if (!Number.isInteger(decimals) || decimals < 0) return null;

  const rawCandidate = String(formState.token_raw_amount ?? "").trim();
  const rawAmount = /^\d+$/.test(rawCandidate)
    ? rawCandidate
    : uiTokenAmountToRaw(formState.token_balance, decimals);
  if (!rawAmount) return null;

  return {
    amount: rawTokenAmountToUi(rawAmount, decimals),
    rawAmount,
    decimals,
  };
}

function aggregateTokenBalance(tokens: WalletTokenAsset[], mint: string): TokenBalanceSnapshot | null {
  const matchingTokens = tokens.filter((item) => String(item.mint).trim() === mint);
  if (matchingTokens.length === 0 || matchingTokens.some((item) => !/^\d+$/.test(item.amount))) {
    return null;
  }
  const decimals = matchingTokens[0]?.decimals;
  if (!Number.isInteger(decimals) || matchingTokens.some((item) => item.decimals !== decimals)) {
    return null;
  }
  const rawBalance = matchingTokens.reduce((total, item) => total + BigInt(item.amount), BigInt(0));
  return {
    amount: rawTokenAmountToUi(rawBalance.toString(), decimals),
    rawAmount: rawBalance.toString(),
    decimals,
  };
}

function normalizedRawAmount(value: unknown): string | null {
  const raw = String(value ?? "").trim();
  return /^\d+$/.test(raw) ? raw : null;
}

function applyTokenBalanceAdjustmentToTokens(
  tokens: WalletTokenAsset[],
  adjustment?: TokenBalanceAdjustment,
): WalletTokenAsset[] {
  if (!adjustment) return tokens;
  const soldRawAmount = normalizedRawAmount(adjustment.soldRawAmount);
  if (!soldRawAmount || BigInt(soldRawAmount) <= BigInt(0)) return tokens;

  let remainingSoldRaw = BigInt(soldRawAmount);
  const sourceAccount = adjustment.sourceAccount?.trim();
  const matchingIndexes = tokens
    .map((token, index) => ({ token, index }))
    .filter(({ token }) => {
      if (String(token.mint).trim() !== adjustment.mint || token.decimals !== adjustment.decimals) {
        return false;
      }
      return sourceAccount ? token.account === sourceAccount : true;
    })
    .map(({ index }) => index);
  const indexes = matchingIndexes.length > 0
    ? matchingIndexes
    : tokens
        .map((token, index) => ({ token, index }))
        .filter(({ token }) => String(token.mint).trim() === adjustment.mint && token.decimals === adjustment.decimals)
        .map(({ index }) => index);
  if (indexes.length === 0) return tokens;

  return tokens
    .map((token, index) => {
      if (!indexes.includes(index) || remainingSoldRaw <= BigInt(0) || !/^\d+$/.test(token.amount)) {
        return token;
      }
      const currentRaw = BigInt(token.amount);
      const deductedRaw = currentRaw < remainingSoldRaw ? currentRaw : remainingSoldRaw;
      remainingSoldRaw -= deductedRaw;
      const nextRaw = currentRaw - deductedRaw;
      return {
        ...token,
        amount: nextRaw.toString(),
        ui_amount_string: rawTokenAmountToUi(nextRaw.toString(), token.decimals),
      };
    })
    .filter((token) => token.amount !== "0");
}

function pendingAdjustmentFromAssets(
  assets: WalletAssetsState | null,
  adjustment?: TokenBalanceAdjustment,
): PendingTokenBalanceAdjustment | undefined {
  if (!assets || !adjustment) return undefined;
  const soldRawAmount = normalizedRawAmount(adjustment.soldRawAmount);
  if (!soldRawAmount) return undefined;
  const adjustedTokens = applyTokenBalanceAdjustmentToTokens(assets.tokens, adjustment);
  const balance = aggregateTokenBalance(adjustedTokens, adjustment.mint);
  return {
    ...adjustment,
    soldRawAmount,
    expectedRawAmount: balance?.rawAmount ?? "0",
    createdAt: Date.now(),
  };
}

function pendingAdjustmentFromFormState(
  formState: FormState,
  adjustment?: TokenBalanceAdjustment,
): PendingTokenBalanceAdjustment | undefined {
  if (!adjustment) return undefined;
  const currentRawAmount = normalizedRawAmount(formState.token_raw_amount);
  const soldRawAmount = normalizedRawAmount(adjustment.soldRawAmount);
  if (!currentRawAmount || !soldRawAmount) return undefined;
  const currentRaw = BigInt(currentRawAmount);
  const soldRaw = BigInt(soldRawAmount);
  return {
    ...adjustment,
    soldRawAmount,
    expectedRawAmount: (currentRaw > soldRaw ? currentRaw - soldRaw : BigInt(0)).toString(),
    createdAt: Date.now(),
  };
}

function applyPendingTokenBalanceAdjustmentToAssets(
  assets: WalletAssetsState,
  adjustment?: PendingTokenBalanceAdjustment,
): WalletAssetsState {
  if (!adjustment || Date.now() - adjustment.createdAt > 30_000) return assets;
  const balance = aggregateTokenBalance(assets.tokens, adjustment.mint);
  if (!balance || balance.decimals !== adjustment.decimals) return assets;
  if (BigInt(balance.rawAmount) <= BigInt(adjustment.expectedRawAmount)) return assets;
  const adjustedTokens = applyTokenBalanceAdjustmentToTokens(assets.tokens, {
    mint: adjustment.mint,
    sourceAccount: adjustment.sourceAccount,
    soldRawAmount: (BigInt(balance.rawAmount) - BigInt(adjustment.expectedRawAmount)).toString(),
    decimals: adjustment.decimals,
  });
  return {
    ...assets,
    tokens: adjustedTokens,
  };
}

function pendingTokenBalanceAdjustmentStillNeeded(
  assets: WalletAssetsState,
  adjustment?: PendingTokenBalanceAdjustment,
): boolean {
  if (!adjustment || Date.now() - adjustment.createdAt > 30_000) return false;
  const balance = aggregateTokenBalance(assets.tokens, adjustment.mint);
  return Boolean(
    balance &&
      balance.decimals === adjustment.decimals &&
      BigInt(balance.rawAmount) > BigInt(adjustment.expectedRawAmount),
  );
}

function pumpSellTokenAdjustment(formState: FormState, data: Record<string, unknown>): TokenBalanceAdjustment | undefined {
  const mint = String(formState.mint ?? "").trim();
  const soldRawAmount = normalizedRawAmount(data.sold_raw_amount);
  const decimals = Number(data.decimals);
  if (!mint || !soldRawAmount || !Number.isInteger(decimals) || decimals < 0) return undefined;
  const sourceAccount = typeof data.source_account === "string" && data.source_account.trim()
    ? data.source_account.trim()
    : undefined;
  return {
    mint,
    soldRawAmount,
    decimals,
    sourceAccount,
  };
}

function parseAddressList(value: string | number | undefined): string[] {
  return String(value ?? "")
    .split(/[\n,]+/)
    .map((item) => item.trim())
    .filter(Boolean);
}

export default function Home() {
  const t = useTranslations();

  const menuItems: MenuItem[] = [
    {
      id: "wallet-list",
      label: t("features.wallet-list.title"),
      icon: <Wallet className="w-5 h-5" />,
    },
    {
      id: "transfer-menu",
      label: t("features.transfer-menu.title"),
      icon: <Coins className="w-5 h-5" />,
      children: [
        {
          id: "pump-workbench",
          label: t("features.pump-workbench.title"),
          icon: <Coins className="w-4 h-4" />,
          network: true,
        },
        {
          id: "wsol-workbench",
          label: t("features.wsol-workbench.title"),
          icon: <RefreshCw className="w-4 h-4" />,
          network: true,
        },
        {
          id: "nonce-workbench",
          label: t("features.nonce-workbench.title"),
          icon: <RefreshCw className="w-4 h-4" />,
          network: true,
        },
      ],
    },
    {
      id: "program-workbench",
      label: t("features.program-workbench.title"),
      icon: <Hash className="w-5 h-5" />,
      network: true,
    },
    {
      id: "squads-workspace",
      label: t("features.workspace.title"),
      icon: <ShieldCheck className="w-5 h-5" />,
      network: true,
    },
    {
      id: "settings",
      label: t("features.settings.title"),
      icon: <Settings className="w-5 h-5" />,
    },
  ];
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [selectedForm, setSelectedForm] = useState<string | null>("wallet-list");
  const [formData, setFormData] = useState<FormState>({});
  const [copied, setCopied] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [authMethod, setAuthMethod] = useState<{ [key: string]: "keystore" | "private" | "encrypted" }>({});
  const [wallets, setWallets] = useState<SavedWallet[]>([]);
  const [walletsLoading, setWalletsLoading] = useState(false);
  const [workspace, setWorkspace] = useState<SquadsWorkspace>(emptyWorkspace);
  const [editingWalletId, setEditingWalletId] = useState<string | null>(null);
  const [rpcProfiles, setRpcProfiles] = useState<RpcProfile[]>(DEFAULT_RPC_PROFILES);
  const [selectedRpcId, setSelectedRpcId] = useState(defaultRpcProfileId(DEFAULT_NETWORK));
  const [settingsNetwork, setSettingsNetwork] = useState<AppNetwork>(DEFAULT_NETWORK);
  const [currentWalletId, setCurrentWalletId] = useState("");
  const [newRpcName, setNewRpcName] = useState("");
  const [newRpcUrl, setNewRpcUrl] = useState("");
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [walletActionsMenuOpen, setWalletActionsMenuOpen] = useState<string | null>(null);
  const [backTarget, setBackTarget] = useState<string | null>(null);
  const [passwordPrompt, setPasswordPrompt] = useState<PasswordPromptRequest | null>(null);
  const [passwordPromptValue, setPasswordPromptValue] = useState("");
  const [masterPasswordPromptValue, setMasterPasswordPromptValue] = useState("");
  const [walletAssets, setWalletAssets] = useState<WalletAssetsState | null>(null);
  const [walletTransactions, setWalletTransactions] = useState<WalletTransactionsState | null>(null);
  const [walletOverviewTab, setWalletOverviewTab] = useState<"assets" | "transactions">("assets");
  const [visibleTokenCount, setVisibleTokenCount] = useState(TOKEN_ASSET_PAGE_SIZE);
  const clientSettingsLoadedRef = useRef(false);
  const walletAssetsRef = useRef<WalletAssetsState | null>(null);
  const walletTransactionsRef = useRef<WalletTransactionsState | null>(null);
  const walletAssetsInFlightRef = useRef<Map<string, Promise<WalletAssetsState | null>>>(new Map());
  const walletTransactionsInFlightRef = useRef<Map<string, Promise<void>>>(new Map());
  const nonceAccountsInFlightRef = useRef<Map<string, Promise<void>>>(new Map());
  const pendingTokenBalanceAdjustmentRef = useRef<PendingTokenBalanceAdjustment | undefined>(undefined);
  const lastAssetRefreshRef = useRef<Map<string, number>>(new Map());
  const lastTransactionRefreshRef = useRef<Map<string, number>>(new Map());
  const lastNonceRefreshRef = useRef<Map<string, number>>(new Map());
  const [tokenActionContext, setTokenActionContext] = useState<FormState | null>(null);
  const [nonceAccounts, setNonceAccounts] = useState<NonceAccountsState | null>(null);
  const [createdNonceAccounts, setCreatedNonceAccounts] = useState<CreatedNonceAccount[]>([]);
  const [nonceCreateOpen, setNonceCreateOpen] = useState(false);
  const [cashbackInfo, setCashbackInfo] = useState<CashbackInfoState | null>(null);
  const [tokenMintInfo, setTokenMintInfo] = useState<TokenMintInfoState | null>(null);
  const walletAuth = (formId: string): WalletAuthTab =>
    normalizeWalletAuth((authMethod[formId] ?? "keystore") as WalletAuthTab);

  const selectedRpc = rpcProfiles.find((profile) => profile.id === selectedRpcId) || rpcProfiles[0] || DEFAULT_RPC_PROFILES[0];
  const effectiveNetwork = currentNetwork(selectedRpc.network);
  const effectiveRpcRequest = rpcRequestValue(selectedRpc);
  const effectiveRpcLabel = selectedRpc.name;
  const effectiveNetworkLabel = networkLabel(t, effectiveNetwork);
  const visibleRpcProfiles = rpcProfiles.filter((profile) => profile.network === settingsNetwork);
  const effectiveWalletId = currentWalletId || wallets[0]?.id || "";
  const effectiveWallet = wallets.find((wallet) => wallet.id === effectiveWalletId);

  useEffect(() => {
    walletTransactionsRef.current = walletTransactions;
  }, [walletTransactions]);

  useEffect(() => {
    walletAssetsRef.current = walletAssets;
  }, [walletAssets]);

  useEffect(() => {
    const tokenCount = walletAssets?.tokens.length ?? 0;
    setVisibleTokenCount((count) => {
      if (tokenCount === 0) return TOKEN_ASSET_PAGE_SIZE;
      return Math.max(TOKEN_ASSET_PAGE_SIZE, Math.min(count, Math.ceil(tokenCount / TOKEN_ASSET_PAGE_SIZE) * TOKEN_ASSET_PAGE_SIZE));
    });
  }, [walletAssets?.address, walletAssets?.network, walletAssets?.tokens.length]);

  useEffect(() => {
    if (
      !walletAssets ||
      !selectedForm ||
      !CURRENT_WALLET_TOKEN_BALANCE_FORM_IDS.has(selectedForm) ||
      walletAssets.network !== effectiveNetwork
    ) {
      return;
    }
    const mint = String((tokenActionContext?.mint ?? formData.mint) || "").trim();
    if (!mint) return;
    const balance = aggregateTokenBalance(walletAssets.tokens, mint);
    if (!balance) return;
    setTokenActionContext((prev) => {
      if (!prev || String(prev.mint ?? "").trim() !== mint) return prev;
      if (String(prev.token_raw_amount ?? "") === balance.rawAmount) return prev;
      return {
        ...prev,
        amount: balance.amount,
        token_balance: balance.amount,
        token_raw_amount: balance.rawAmount,
        decimals: balance.decimals,
      };
    });
    setFormData((prev) => {
      const formMint = String(prev.mint ?? "").trim();
      if (formMint !== mint || String(prev.token_raw_amount ?? "") === balance.rawAmount) return prev;
      return {
        ...prev,
        token_balance: balance.amount,
        token_raw_amount: balance.rawAmount,
        decimals: balance.decimals,
      };
    });
  }, [effectiveNetwork, formData.mint, formData.token_raw_amount, selectedForm, tokenActionContext?.mint, walletAssets]);

  const requestNetwork = (value?: unknown): string => {
    const hasFormNetwork = typeof formData.network === "string" || typeof formData.network === "number";
    if (value === undefined && (!hasFormNetwork || currentNetwork(formData.network) === effectiveNetwork)) {
      return rpcRequestValue(selectedRpc);
    }
    const raw =
      typeof value === "string" || typeof value === "number"
        ? value
        : hasFormNetwork
          ? formData.network
          : effectiveNetwork;
    return typeof raw === "string" && raw.startsWith("rpc:") ? raw : currentNetwork(raw);
  };

  const pumpSellSuccessMessage = (
    data: { dex?: unknown },
    fallbackFormId: "pumpfun-sell" | "pumpswap-sell",
  ): string => {
    const dex = typeof data.dex === "string" ? data.dex : "";
    if (fallbackFormId === "pumpfun-sell" && dex === "pumpswap") {
      return t("features.pumpfun-sell.autoPumpswapSuccess");
    }
    if (dex === "pumpfun") return t("features.pumpfun-sell.success");
    if (dex === "pumpswap") return t("features.pumpswap-sell.success");
    return t(`features.${fallbackFormId}.success`);
  };

  const setCurrentWallet = useCallback((walletId: string) => {
    setCurrentWalletId(walletId);
    if (typeof window !== "undefined") {
      if (walletId) {
        window.localStorage.setItem(CURRENT_WALLET_STORAGE_KEY, walletId);
      } else {
        window.localStorage.removeItem(CURRENT_WALLET_STORAGE_KEY);
      }
    }
    setFormData((prev) => {
      if (!selectedForm || !authFormsWithWallets.has(selectedForm)) return prev;
      return walletId ? { ...prev, wallet_id: walletId } : prev;
    });
  }, [selectedForm]);

  const setAppRpc = useCallback((profileId: string) => {
    const profile = rpcProfiles.find((item) => item.id === profileId);
    if (!profile) return;
    setSelectedRpcId(profileId);
    setSettingsNetwork(profile.network);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(NETWORK_STORAGE_KEY, profileId);
    }
    setFormData((prev) => ({ ...prev, network: profile.network }));
  }, [rpcProfiles]);

  const setAppNetwork = useCallback((network: AppNetwork) => {
    setSettingsNetwork(network);
    const nextProfile = rpcProfiles.find((profile) => profile.network === network && profile.id === selectedRpcId)
      || rpcProfiles.find((profile) => profile.network === network)
      || DEFAULT_RPC_PROFILES.find((profile) => profile.network === network)
      || DEFAULT_RPC_PROFILES[0];
    setSelectedRpcId(nextProfile.id);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(NETWORK_STORAGE_KEY, nextProfile.id);
    }
    setFormData((prev) => ({ ...prev, network }));
  }, [rpcProfiles, selectedRpcId]);

  const handleAddRpcProfile = () => {
    const url = validateRpcUrl(newRpcUrl);
    const name = newRpcName.trim();
    if (!name || !url) {
      toast.error(t("features.settings.rpcInvalid"));
      return;
    }

    const nextProfile: RpcProfile = {
      id: `custom-${Date.now().toString(36)}`,
      name,
      url,
      network: settingsNetwork,
    };
    const nextProfiles = mergeRpcProfiles([...rpcProfiles.filter((profile) => profile.id !== nextProfile.id), nextProfile]);
    const existing = rpcProfiles.find((profile) => rpcProfileKey(profile) === rpcProfileKey(nextProfile));
    if (existing) {
      setAppRpc(existing.id);
      setNewRpcName("");
      setNewRpcUrl("");
      toast.success(t("features.settings.rpcSelected"));
      return;
    }

    setRpcProfiles(nextProfiles);
    saveCustomRpcProfiles(nextProfiles);
    setNewRpcName("");
    setNewRpcUrl("");
    setSelectedRpcId(nextProfile.id);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(NETWORK_STORAGE_KEY, nextProfile.id);
    }
    setFormData((prev) => ({ ...prev, network: nextProfile.network }));
    toast.success(t("features.settings.rpcAdded"));
  };

  const handleRemoveRpcProfile = (profileId: string) => {
    const profile = rpcProfiles.find((item) => item.id === profileId);
    if (!profile || profile.builtin) return;
    const nextProfiles = rpcProfiles.filter((item) => item.id !== profileId);
    setRpcProfiles(nextProfiles);
    saveCustomRpcProfiles(nextProfiles);
    if (selectedRpcId === profileId) {
      const fallbackId = defaultRpcProfileId(profile.network);
      setSelectedRpcId(fallbackId);
      setSettingsNetwork(profile.network);
      if (typeof window !== "undefined") {
        window.localStorage.setItem(NETWORK_STORAGE_KEY, fallbackId);
      }
      setFormData((prev) => ({ ...prev, network: profile.network }));
    }
    toast.success(t("features.settings.rpcRemoved"));
  };

  const loadWallets = useCallback(async () => {
    setWalletsLoading(true);
    try {
      const loadedWallets = await fetchWallets();
      setWallets(loadedWallets);
      const storedWalletId = loadStoredWalletId();
      const nextWalletId = loadedWallets.some((wallet) => wallet.id === storedWalletId)
        ? storedWalletId
        : loadedWallets[0]?.id || "";
      setCurrentWallet(nextWalletId);
      setFormData((prev) => {
        const prevWalletId = String(prev.wallet_id ?? "").trim();
        const prevWalletStillExists =
          !prevWalletId || loadedWallets.some((wallet) => wallet.id === prevWalletId);
        if (
          !selectedForm ||
          !authFormsWithWallets.has(selectedForm) ||
          String(prev.keystoreJson ?? "").trim()
        ) {
          return prevWalletStillExists ? prev : { ...prev, wallet_id: undefined };
        }
        if (!prevWalletStillExists) {
          return nextWalletId ? { ...prev, wallet_id: nextWalletId } : { ...prev, wallet_id: undefined };
        }
        if (!nextWalletId || prevWalletId) {
          return prev;
        }
        return { ...prev, wallet_id: nextWalletId };
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : "加载钱包列表失败";
      toast.error(message);
    } finally {
      setWalletsLoading(false);
    }
  }, [selectedForm, setCurrentWallet]);

  const selectedSavedWallet = (): SavedWallet | undefined => {
    const walletId = String(formData.wallet_id ?? effectiveWalletId).trim();
    return walletId ? wallets.find((wallet) => wallet.id === walletId) : undefined;
  };

  const savedWalletFromForm = (state: FormState = formData): SavedWallet | undefined => {
    const walletId = String(state.wallet_id ?? "").trim();
    return walletId ? wallets.find((wallet) => wallet.id === walletId) : undefined;
  };

  const loadWalletAssets = useCallback(async (
    wallet?: SavedWallet,
    options: { refresh?: boolean; background?: boolean; force?: boolean } = {},
  ): Promise<WalletAssetsState | null> => {
    if (!wallet) {
      setWalletAssets(null);
      return null;
    }
    const refresh = options.refresh !== false;
    const background = options.background === true;
    const requestKey = `${wallet.public_key}:${effectiveRpcRequest}:${refresh ? "refresh" : "cache"}`;
    const walletKey = `${wallet.public_key}:${effectiveRpcRequest}`;
    if (refresh && !options.force) {
      const lastRefresh = lastAssetRefreshRef.current.get(walletKey) ?? 0;
      if (Date.now() - lastRefresh < ASSET_AUTO_REFRESH_TTL_MS) {
        const currentAssets = walletAssetsRef.current;
        return currentAssets?.address === wallet.public_key && currentAssets.network === effectiveNetwork
          ? currentAssets
          : null;
      }
    }
    const inFlight = walletAssetsInFlightRef.current.get(requestKey);
    if (inFlight) return inFlight;

    const request = (async (): Promise<WalletAssetsState | null> => {
      setWalletAssets((prev) => ({
        address: wallet.public_key,
        network: effectiveNetwork,
        solBalance:
          prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.solBalance : "--",
        tokens: prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.tokens : [],
        loading: !background,
        refreshing: background,
        cached: prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.cached : undefined,
        updatedAt: prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.updatedAt : undefined,
        error: prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.error : undefined,
      }));

      const response = await apiFetch("wallet/assets", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          address: wallet.public_key,
          network: effectiveRpcRequest,
          refresh,
        }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("features.wallet-list.assetLoadFailed"));
      }
      if (!refresh && !data.updated_at) {
        setWalletAssets((prev) => ({
          address: wallet.public_key,
          network: effectiveNetwork,
          solBalance:
            prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.solBalance : "--",
          tokens: prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.tokens : [],
          loading: false,
          refreshing: false,
          cached: true,
        }));
        return null;
      }
      if (refresh) {
        lastAssetRefreshRef.current.set(walletKey, Date.now());
      }
      const nextAssetsBase: WalletAssetsState = {
        address: data.address || wallet.public_key,
        network: currentNetwork(data.network),
        solBalance: String(data.sol_balance ?? "0"),
        tokens: Array.isArray(data.tokens) ? data.tokens : [],
        loading: false,
        refreshing: false,
        cached: Boolean(data.cached),
        updatedAt: typeof data.updated_at === "number" ? data.updated_at : undefined,
        error: undefined,
      };
      const pendingAdjustment = pendingTokenBalanceAdjustmentRef.current;
      if (!pendingTokenBalanceAdjustmentStillNeeded(nextAssetsBase, pendingAdjustment)) {
        pendingTokenBalanceAdjustmentRef.current = undefined;
      }
      let nextAssets = applyPendingTokenBalanceAdjustmentToAssets(nextAssetsBase, pendingAdjustment);
      setWalletAssets((prev) => {
        const previousTokens =
          prev?.address === wallet.public_key && prev.network === effectiveNetwork ? prev.tokens : [];
        const mergedAssets = {
          ...nextAssetsBase,
          tokens: mergeTokenAssetMetadata(nextAssets.tokens, previousTokens),
        };
        nextAssets = applyPendingTokenBalanceAdjustmentToAssets(mergedAssets, pendingAdjustment);
        return nextAssets;
      });
      return nextAssets;
    })();

    walletAssetsInFlightRef.current.set(requestKey, request);
    try {
      return await request;
    } catch (err) {
      setWalletAssets((prev) => ({
        address: wallet.public_key,
        network: effectiveNetwork,
        solBalance: prev?.solBalance && prev.solBalance !== "--" ? prev.solBalance : "0",
        tokens: prev?.address === wallet.public_key ? prev.tokens : [],
        loading: false,
        refreshing: false,
        cached: prev?.address === wallet.public_key ? prev.cached : undefined,
        updatedAt: prev?.address === wallet.public_key ? prev.updatedAt : undefined,
        error: err instanceof Error ? err.message : "",
      }));
      return null;
    } finally {
      walletAssetsInFlightRef.current.delete(requestKey);
    }
  }, [effectiveNetwork, effectiveRpcRequest, t]);

  const loadWalletTransactions = useCallback(async (
    wallet?: SavedWallet,
    mode: "replace" | "append" = "replace",
    options: { force?: boolean } = {},
  ) => {
    if (!wallet) {
      setWalletTransactions(null);
      return;
    }
    const appendState =
      mode === "append" &&
      walletTransactionsRef.current?.address === wallet.public_key &&
      walletTransactionsRef.current.network === effectiveNetwork
        ? walletTransactionsRef.current
        : null;
    if (appendState && (!appendState.hasMore || appendState.loaded >= MAX_TRANSACTION_HISTORY || !appendState.nextBefore)) {
      return;
    }
    const before = appendState?.nextBefore;
    const walletKey = `${wallet.public_key}:${effectiveRpcRequest}`;
    const requestKey = `${walletKey}:${mode}:${before || ""}`;
    if (mode === "replace" && !options.force) {
      const lastRefresh = lastTransactionRefreshRef.current.get(walletKey) ?? 0;
      if (Date.now() - lastRefresh < TRANSACTION_AUTO_REFRESH_TTL_MS) {
        return;
      }
    }
    const inFlight = walletTransactionsInFlightRef.current.get(requestKey);
    if (inFlight) return inFlight;

    const request = (async () => {
      setWalletTransactions((prev) => ({
        address: wallet.public_key,
        network: effectiveNetwork,
        transactions:
          mode === "append" && prev?.address === wallet.public_key && prev.network === effectiveNetwork
            ? prev.transactions
            : [],
        nextBefore:
          mode === "append" && prev?.address === wallet.public_key && prev.network === effectiveNetwork
            ? prev.nextBefore
            : undefined,
        hasMore:
          mode === "append" && prev?.address === wallet.public_key && prev.network === effectiveNetwork
            ? prev.hasMore
            : true,
        loaded:
          mode === "append" && prev?.address === wallet.public_key && prev.network === effectiveNetwork
            ? prev.loaded
            : 0,
        loading: true,
      }));

      const response = await apiFetch("wallet/transactions", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          address: wallet.public_key,
          network: effectiveRpcRequest,
          limit: TRANSACTION_PAGE_SIZE,
          ...(before ? { before } : {}),
        }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("features.wallet-list.transactionsLoadFailed"));
      }
      const nextPage = Array.isArray(data.transactions) ? data.transactions : [];
      const previous = appendState?.transactions ?? [];
      const seen = new Set(previous.map((item) => item.signature));
      const merged = [
        ...previous,
        ...nextPage.filter((item: WalletTransactionRecord) => !seen.has(item.signature)),
      ].slice(0, MAX_TRANSACTION_HISTORY);
      setWalletTransactions({
        address: data.address || wallet.public_key,
        network: currentNetwork(data.network),
        transactions: merged,
        nextBefore: typeof data.next_before === "string" ? data.next_before : undefined,
        hasMore: Boolean(data.has_more) && merged.length < MAX_TRANSACTION_HISTORY,
        loaded: merged.length,
        loading: false,
      });
      if (mode === "replace") {
        lastTransactionRefreshRef.current.set(walletKey, Date.now());
      }
    })();

    walletTransactionsInFlightRef.current.set(requestKey, request);
    try {
      return await request;
    } catch (err) {
      setWalletTransactions((prev) => ({
        address: wallet.public_key,
        network: effectiveNetwork,
        transactions: prev?.address === wallet.public_key ? prev.transactions : [],
        nextBefore: prev?.address === wallet.public_key ? prev.nextBefore : undefined,
        hasMore: prev?.address === wallet.public_key ? prev.hasMore : false,
        loaded: prev?.address === wallet.public_key ? prev.loaded : 0,
        loading: false,
        error: err instanceof Error ? err.message : "",
      }));
    } finally {
      walletTransactionsInFlightRef.current.delete(requestKey);
    }
  }, [effectiveNetwork, effectiveRpcRequest, t]);

  const loadNonceAccounts = useCallback(async (
    wallet?: SavedWallet,
    options: { force?: boolean } = {},
  ) => {
    if (!wallet) {
      setNonceAccounts(null);
      return;
    }
    const requestKey = `${wallet.public_key}:${effectiveRpcRequest}`;
    if (!options.force) {
      const lastRefresh = lastNonceRefreshRef.current.get(requestKey) ?? 0;
      if (Date.now() - lastRefresh < NONCE_AUTO_REFRESH_TTL_MS) {
        return;
      }
    }
    const inFlight = nonceAccountsInFlightRef.current.get(requestKey);
    if (inFlight) return inFlight;

    const request = (async () => {
      setNonceAccounts((prev) => ({
        owner: wallet.public_key,
        network: effectiveNetwork,
        nonceAccounts:
          prev?.owner === wallet.public_key && prev.network === effectiveNetwork ? prev.nonceAccounts : [],
        loading: true,
      }));

      const response = await apiFetch("nonce/list", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          owner: wallet.public_key,
          network: effectiveRpcRequest,
        }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("features.create-nonce.error"));
      }
      setNonceAccounts({
        owner: data.owner || wallet.public_key,
        network: currentNetwork(data.network),
        nonceAccounts: Array.isArray(data.nonce_accounts) ? data.nonce_accounts : [],
        loading: false,
      });
      lastNonceRefreshRef.current.set(requestKey, Date.now());
    })();

    nonceAccountsInFlightRef.current.set(requestKey, request);
    try {
      return await request;
    } catch (err) {
      setNonceAccounts((prev) => ({
        owner: wallet.public_key,
        network: effectiveNetwork,
        nonceAccounts: prev?.owner === wallet.public_key ? prev.nonceAccounts : [],
        loading: false,
        error: err instanceof Error ? err.message : "",
      }));
    } finally {
      nonceAccountsInFlightRef.current.delete(requestKey);
    }
  }, [effectiveNetwork, effectiveRpcRequest, t]);

  const loadCashbackInfo = useCallback(async (dex: "pumpfun" | "pumpswap", wallet?: SavedWallet) => {
    if (!wallet) {
      setCashbackInfo(null);
      return;
    }
    setCashbackInfo((prev) => ({
      owner: wallet.public_key,
      network: effectiveNetwork,
      dex,
      accumulator:
        prev?.owner === wallet.public_key && prev.network === effectiveNetwork && prev.dex === dex
          ? prev.accumulator
          : undefined,
      amountLamports:
        prev?.owner === wallet.public_key && prev.network === effectiveNetwork && prev.dex === dex
          ? prev.amountLamports
          : undefined,
      uiAmountString:
        prev?.owner === wallet.public_key && prev.network === effectiveNetwork && prev.dex === dex
          ? prev.uiAmountString
          : undefined,
      asset:
        prev?.owner === wallet.public_key && prev.network === effectiveNetwork && prev.dex === dex
          ? prev.asset
          : undefined,
      available:
        prev?.owner === wallet.public_key && prev.network === effectiveNetwork && prev.dex === dex
          ? prev.available
          : undefined,
      loading: true,
    }));
    try {
      const response = await apiFetch(`${dex}/cashback-info`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          owner: wallet.public_key,
          network: effectiveRpcRequest,
        }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t(`features.${dex}-cashback.error`));
      }
      setCashbackInfo({
        owner: data.owner || wallet.public_key,
        network: currentNetwork(data.network),
        dex,
        accumulator: String(data.accumulator ?? ""),
        amountLamports: Number(data.amount_lamports ?? 0),
        uiAmountString: String(data.ui_amount_string ?? "0"),
        asset: String(data.asset ?? (dex === "pumpfun" ? "SOL" : "WSOL")),
        available: data.available === true,
        loading: false,
      });
    } catch (err) {
      setCashbackInfo({
        owner: wallet.public_key,
        network: effectiveNetwork,
        dex,
        loading: false,
        error: err instanceof Error ? err.message : "",
      });
    }
  }, [effectiveNetwork, effectiveRpcRequest, t]);

  useEffect(() => {
    if (!clientSettingsLoadedRef.current) {
      clientSettingsLoadedRef.current = true;
      const initialRpc = loadInitialRpcState();
      const initialRpcProfile =
        initialRpc.profiles.find((profile) => profile.id === initialRpc.selectedId) ||
        initialRpc.profiles[0] ||
        DEFAULT_RPC_PROFILES[0];
      setRpcProfiles(initialRpc.profiles);
      setSelectedRpcId(initialRpcProfile.id);
      setSettingsNetwork(initialRpcProfile.network || loadStoredNetwork());
      setCurrentWalletId(loadStoredWalletId());
    }
    void loadWallets();
  }, [loadWallets]);

  useEffect(() => {
    if (!effectiveWallet) {
      void loadWalletAssets(undefined);
      void loadWalletTransactions(undefined);
      void loadNonceAccounts(undefined);
      setVisibleTokenCount(TOKEN_ASSET_PAGE_SIZE);
      return;
    }
    void (async () => {
      await loadWalletAssets(effectiveWallet, { refresh: false });
      void loadWalletAssets(effectiveWallet, { refresh: true, background: true });
    })();
    setVisibleTokenCount(TOKEN_ASSET_PAGE_SIZE);
  }, [effectiveWallet, effectiveRpcRequest, loadWalletAssets, loadWalletTransactions, loadNonceAccounts]);

  useEffect(() => {
    if (!effectiveWallet) return;
    const activeOperationForm =
      selectedForm !== "wallet-list" &&
      (
        CURRENT_WALLET_TOKEN_BALANCE_FORM_IDS.has(selectedForm || "") ||
        selectedForm === "transfer-sol" ||
        selectedForm === "wsol-workbench" ||
        selectedForm === "pump-workbench"
      );
    const intervalMs = activeOperationForm
      ? ACTIVE_OPERATION_ASSET_REFRESH_INTERVAL_MS
      : WALLET_ASSET_AUTO_REFRESH_INTERVAL_MS;
    const refresh = () => {
      void loadWalletAssets(effectiveWallet, { refresh: true, background: true, force: true });
    };
    const timer = window.setInterval(refresh, intervalMs);
    return () => window.clearInterval(timer);
  }, [effectiveWallet, effectiveRpcRequest, loadWalletAssets, selectedForm]);

  useEffect(() => {
    if (walletOverviewTab !== "transactions" || !effectiveWallet) {
      return;
    }
    const currentTransactions = walletTransactionsRef.current;
    const lastRefresh = lastTransactionRefreshRef.current.get(`${effectiveWallet.public_key}:${effectiveRpcRequest}`) ?? 0;
    if (
      currentTransactions?.address === effectiveWallet.public_key &&
      currentTransactions.network === effectiveNetwork &&
      Date.now() - lastRefresh < TRANSACTION_AUTO_REFRESH_TTL_MS
    ) {
      return;
    }
    void loadWalletTransactions(effectiveWallet);
  }, [effectiveWallet, effectiveNetwork, effectiveRpcRequest, walletOverviewTab, loadWalletTransactions]);

  useEffect(() => {
    if (selectedForm !== "nonce-workbench" && selectedForm !== "create-nonce") {
      return;
    }
    if (!effectiveWallet) {
      void loadNonceAccounts(undefined);
      return;
    }
    const currentNonceAccounts = nonceAccounts;
    const lastRefresh = lastNonceRefreshRef.current.get(`${effectiveWallet.public_key}:${effectiveRpcRequest}`) ?? 0;
    if (
      currentNonceAccounts?.owner === effectiveWallet.public_key &&
      currentNonceAccounts.network === effectiveNetwork &&
      Date.now() - lastRefresh < NONCE_AUTO_REFRESH_TTL_MS
    ) {
      return;
    }
    void loadNonceAccounts(effectiveWallet);
  }, [selectedForm, effectiveWallet, effectiveNetwork, effectiveRpcRequest, nonceAccounts, loadNonceAccounts]);

  useEffect(() => {
    if (selectedForm === "pumpfun-cashback") {
      void loadCashbackInfo("pumpfun", effectiveWallet);
    } else if (selectedForm === "pumpswap-cashback") {
      void loadCashbackInfo("pumpswap", effectiveWallet);
    }
  }, [selectedForm, effectiveWallet, loadCashbackInfo]);

  useEffect(() => {
    const needsMintInfo = selectedForm === "transfer-token" || selectedForm === "squads-token-transfer";
    const mint = String(formData.mint ?? "").trim();
    const network = currentNetwork(
      typeof formData.network === "string" || typeof formData.network === "number"
        ? formData.network
        : effectiveNetwork,
    );
    if (!needsMintInfo || !mint) {
      setTokenMintInfo(null);
      return;
    }

    let cancelled = false;
    setTokenMintInfo((prev) => ({
      mint,
      network,
      decimals: prev?.mint === mint && prev.network === network ? prev.decimals : undefined,
      supply: prev?.mint === mint && prev.network === network ? prev.supply : undefined,
      uiAmountString: prev?.mint === mint && prev.network === network ? prev.uiAmountString : undefined,
      loading: true,
    }));

    const timer = window.setTimeout(() => {
      void (async () => {
        try {
          const response = await apiFetch("token/mint-info", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              mint,
              network: network === effectiveNetwork ? effectiveRpcRequest : network,
            }),
          });
          const data = await response.json();
          if (!response.ok) {
            throw new Error(data.error || t("features.transfer-token.mintInfoError"));
          }
          if (cancelled) return;
          const decimals = Number(data.decimals);
          setTokenMintInfo({
            mint: String(data.mint || mint),
            network: currentNetwork(data.network),
            decimals: Number.isInteger(decimals) ? decimals : undefined,
            supply: String(data.supply ?? ""),
            uiAmountString: String(data.ui_amount_string ?? ""),
            loading: false,
          });
          setFormData((prev) => {
            if (String(prev.mint ?? "").trim() !== mint) return prev;
            return {
              ...prev,
              decimals: Number.isInteger(decimals) ? decimals : undefined,
              tokenSupply: String(data.supply ?? ""),
              tokenUiSupply: String(data.ui_amount_string ?? ""),
            };
          });
        } catch (err) {
          if (cancelled) return;
          setTokenMintInfo({
            mint,
            network,
            loading: false,
            error: err instanceof Error ? err.message : t("features.transfer-token.mintInfoError"),
          });
          setFormData((prev) => {
            if (String(prev.mint ?? "").trim() !== mint) return prev;
            const next = { ...prev };
            delete next.decimals;
            delete next.tokenSupply;
            delete next.tokenUiSupply;
            return next;
          });
        }
      })();
    }, 350);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [selectedForm, formData.mint, formData.network, effectiveNetwork, effectiveRpcRequest, t]);

  useEffect(() => {
    setWorkspace(loadWorkspace());
  }, []);

  useEffect(() => {
    const preventNumberInputWheel = (event: WheelEvent) => {
      const target = event.target;
      if (!(target instanceof HTMLInputElement) || target.type !== "number") {
        return;
      }
      event.preventDefault();
      target.blur();
    };

    window.addEventListener("wheel", preventNumberInputWheel, { capture: true, passive: false });
    return () => {
      window.removeEventListener("wheel", preventNumberInputWheel, { capture: true });
    };
  }, []);

  const closeDropdownMenus = useCallback(() => {
    setWalletActionsMenuOpen(null);
    document.querySelectorAll<HTMLDetailsElement>("details[data-close-on-outside][open]").forEach((details) => {
      details.open = false;
    });
  }, []);

  useEffect(() => {
    const closeOnOutsidePointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }

      if (!(target instanceof Element) || !target.closest("[data-wallet-actions-menu]")) {
        setWalletActionsMenuOpen(null);
      }

      document.querySelectorAll<HTMLDetailsElement>("details[data-close-on-outside][open]").forEach((details) => {
        if (!details.contains(target)) {
          details.open = false;
        }
      });
    };

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeDropdownMenus();
      }
    };

    document.addEventListener("pointerdown", closeOnOutsidePointerDown);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsidePointerDown);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [closeDropdownMenus]);

  const updateWorkspace = (updater: (prev: SquadsWorkspace) => SquadsWorkspace) => {
    setWorkspace((prev) => {
      const next = updater(prev);
      if (typeof window !== "undefined") {
        window.localStorage.setItem(WORKSPACE_STORAGE_KEY, JSON.stringify(next));
      }
      return next;
    });
  };

  const saveWorkspaceMultisig = (
    addressValue: unknown,
    vaultValue?: unknown,
    labelValue?: unknown,
    networkValue?: unknown,
    actor: WorkspaceActor = {},
  ) => {
    const address = String(addressValue ?? "").trim();
    if (!address) return;
    const vault = String(vaultValue ?? "").trim() || undefined;
    const label = String(labelValue ?? "").trim() || undefined;
    const network = currentNetwork(
      typeof networkValue === "string" || typeof networkValue === "number"
        ? networkValue
        : effectiveNetwork,
    );
    const updatedAt = Date.now();
    updateWorkspace((prev) => ({
      ...prev,
      multisigs: (() => {
        const existing = prev.multisigs.find((item) => item.address === address && item.network === network);
        return [
          {
            ...existing,
            address,
            vault,
            label: label ?? existing?.label,
            network,
            updatedAt,
            createdBy: actor.createdBy ?? existing?.createdBy,
            createdByLabel: actor.createdByLabel ?? existing?.createdByLabel,
          },
          ...prev.multisigs.filter((item) => item.address !== address || item.network !== network),
        ].slice(0, 50);
      })(),
    }));
  };

  const saveWorkspaceProgram = (addressValue: unknown, labelValue?: unknown, networkValue?: unknown) => {
    const address = String(addressValue ?? "").trim();
    if (!address) return;
    const label = String(labelValue ?? "").trim() || undefined;
    const network = currentNetwork(
      typeof networkValue === "string" || typeof networkValue === "number"
        ? networkValue
        : effectiveNetwork,
    );
    const updatedAt = Date.now();
    updateWorkspace((prev) => ({
      ...prev,
      programs: [
        { address, label, network, updatedAt },
        ...prev.programs.filter((item) => item.address !== address || item.network !== network),
      ].slice(0, 50),
    }));
  };

  const saveWorkspaceProposal = (
    addressValue: unknown,
    multisigValue: unknown,
    transactionIndexValue: unknown,
    kind?: string,
    status = "active",
    networkValue?: unknown,
    actor: WorkspaceActor = {},
  ) => {
    const address = String(addressValue ?? "").trim();
    const multisig = String(multisigValue ?? "").trim();
    if (!address || !multisig) return;
    const transactionIndex = String(transactionIndexValue ?? "").trim();
    const network = currentNetwork(
      typeof networkValue === "string" || typeof networkValue === "number"
        ? networkValue
        : effectiveNetwork,
    );
    const updatedAt = Date.now();
    updateWorkspace((prev) => ({
      ...prev,
      proposals: (() => {
        const existing = prev.proposals.find((item) => item.address === address && item.network === network);
        return [
          {
            ...existing,
            address,
            multisig,
            transactionIndex,
            status,
            kind: kind ?? existing?.kind,
            network,
            updatedAt,
            createdBy: actor.createdBy ?? existing?.createdBy,
            createdByLabel: actor.createdByLabel ?? existing?.createdByLabel,
          },
          ...prev.proposals.filter((item) => item.address !== address || item.network !== network),
        ].slice(0, 100);
      })(),
    }));
  };

  const removeWorkspaceItem = (
    kind: "multisigs" | "programs" | "proposals",
    address: string,
    network: AppNetwork,
  ) => {
    updateWorkspace((prev) => ({
      ...prev,
      [kind]: prev[kind].filter((item) => item.address !== address || item.network !== network),
    }));
  };

  const refreshWorkspaceProposals = async (multisig?: WorkspaceMultisig) => {
    const network = currentNetwork(effectiveNetwork);
    const targets = multisig ? [multisig] : workspace.multisigs.filter((item) => item.network === network);
    if (targets.length === 0) {
      toast.error(t("features.workspace.noSavedMultisigs"));
      return;
    }

    try {
      const fetched: WorkspaceProposal[] = [];
      for (const item of targets) {
        const response = await apiFetch("squads/proposals", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            multisig: item.address,
            network: item.network,
            limit: 20,
          }),
        });
        const data = await response.json();
        if (!response.ok) {
          throw new Error(data.error || t("features.workspace.refreshFailed"));
        }
        saveWorkspaceMultisig(data.multisig, data.vault, undefined, data.network);
        if (Array.isArray(data.proposals)) {
          for (const proposal of data.proposals) {
            fetched.push({
              address: proposal.address,
              multisig: data.multisig,
              transactionIndex: String(proposal.transaction_index),
              status: proposal.status,
              network: currentNetwork(data.network),
              updatedAt: Date.now(),
            });
          }
        }
      }
      updateWorkspace((prev) => {
        const keys = new Set(fetched.map((item) => `${item.network}:${item.address}`));
        const merged = fetched.map((item) => {
          const existing = prev.proposals.find(
            (proposal) => proposal.address === item.address && proposal.network === item.network,
          );
          return {
            ...existing,
            ...item,
            kind: item.kind ?? existing?.kind,
            createdBy: existing?.createdBy,
            createdByLabel: existing?.createdByLabel,
          };
        });
        return {
          ...prev,
          proposals: [
            ...merged,
            ...prev.proposals.filter((item) => !keys.has(`${item.network}:${item.address}`)),
          ].slice(0, 100),
        };
      });
      toast.success(t("features.workspace.refreshSuccess"));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t("features.workspace.refreshFailed"));
    }
  };

  const handleWorkspaceProposalAction = async (
    proposal: WorkspaceProposal,
    action: WorkspaceProposalAction,
    formState: FormState = formData,
  ) => {
    const m = walletAuth("squads-workspace");
    if (!validateWalletAuth(m, formState, "private_key")) {
      toast.error(t("features.squads-vote.fillAllFields"));
      return;
    }

    const transactionIndex = parseInt(proposal.transactionIndex, 10);
    if (action === "execute" && !Number.isInteger(transactionIndex)) {
      toast.error(t("features.squads-execute.fillAllFields"));
      return;
    }

    setLoading(true);
    try {
      const requestBody: ApiRequestBody = {
        multisig: proposal.multisig,
        proposal: proposal.address,
        network: proposal.network,
      };
      if (action === "execute") {
        requestBody.transaction_index = transactionIndex;
      } else {
        requestBody.memo = formState.memo;
      }
      applyWalletAuth(requestBody, m, formState, "private_key");

      const endpoint =
        action === "approve"
          ? "squads/proposal/approve"
          : action === "reject"
            ? "squads/proposal/reject"
            : "squads/proposal/execute";
      const response = await apiFetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(requestBody),
      });
      const data = await response.json();
      if (response.ok) {
        toast.success(
          action === "approve"
            ? t("features.squads-approve.success")
            : action === "reject"
              ? t("features.squads-reject.success")
              : t("features.squads-execute.success"),
        );
        saveWorkspaceProposal(
          proposal.address,
          proposal.multisig,
          proposal.transactionIndex,
          proposal.kind,
          action === "approve" ? "voted" : action === "reject" ? "rejected" : "executed",
          proposal.network,
          {
            createdBy: proposal.createdBy,
            createdByLabel: proposal.createdByLabel,
          },
        );
        setFormData((prev) => ({
          ...prev,
          multisig: proposal.multisig,
          proposal: proposal.address,
          transactionIndex: proposal.transactionIndex,
          signature: data.signature,
          network: data.network || proposal.network,
        }));
      } else {
        toast.error(
          data.error ||
            (action === "execute"
              ? t("features.squads-execute.error")
              : t("features.squads-vote.error")),
        );
      }
    } catch (err) {
      const message =
        err instanceof Error ? err.message : typeof err === "string" ? err : t("errors.unknownError");
      toast.error(
        t("errors.requestFailedWithHint", {
          message,
          port: String(DEFAULT_API_PORT),
        }),
      );
    } finally {
      setLoading(false);
    }
  };

  const handleSaveWalletMetadata = async (wallet: SavedWallet) => {
    const name = String(formData[`walletName:${wallet.id}`] ?? wallet.name).trim();
    if (!name) {
      toast.error(t("formUi.walletNameRequired"));
      return;
    }

    setLoading(true);
    try {
      const response = await apiFetch(`wallets/${wallet.id}`, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name,
        }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("formUi.walletMetadataSaveFailed"));
      }
      toast.success(t("formUi.walletMetadataSaved"));
      await loadWallets();
      setEditingWalletId(null);
      setFormData((prev) => {
        const next = { ...prev };
        delete next[`walletName:${wallet.id}`];
        return next;
      });
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t("formUi.walletMetadataSaveFailed"));
    } finally {
      setLoading(false);
    }
  };

  const handleCancelWalletMetadata = (wallet: SavedWallet) => {
    setEditingWalletId(null);
    setFormData((prev) => {
      const next = { ...prev };
      delete next[`walletName:${wallet.id}`];
      return next;
    });
  };

  const handleExportKeystore = async (wallet: SavedWallet, passwordValue: string) => {
    const password = passwordValue;
    if (password.length === 0) {
      toast.error(t("features.settings.exportPasswordRequired"));
      return;
    }

    setLoading(true);
    try {
      const response = await apiFetch(`wallets/${wallet.id}/export`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("features.settings.exportFailed"));
      }
      downloadFile(
        data.keystore_json,
        `${safeFilename(wallet.name)}-${wallet.public_key.slice(0, 8)}-keystore.json`,
      );
      toast.success(t("features.settings.exportSuccess"));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t("features.settings.exportFailed"));
    } finally {
      setLoading(false);
    }
  };

  const handleExportPrivateKey = async (wallet: SavedWallet, passwordValue: string) => {
    const password = passwordValue;
    if (password.length === 0) {
      toast.error(t("features.settings.exportPasswordRequired"));
      return;
    }

    setLoading(true);
    try {
      const response = await apiFetch(`wallets/${wallet.id}/export-private-key`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("features.settings.exportPrivateKeyFailed"));
      }
      const content = [
        "Sol SafeKey plaintext private key export",
        `Wallet: ${wallet.name}`,
        `Public Key: ${wallet.public_key}`,
        "",
        "Private Key (base58):",
        data.private_key,
        "",
        "Keep this file offline. Anyone with this private key can spend assets in this wallet.",
      ].join("\n");
      downloadFile(
        content,
        `${safeFilename(wallet.name)}-${wallet.public_key.slice(0, 8)}-private-key.txt`,
        "text/plain",
      );
      toast.success(t("features.settings.exportPrivateKeySuccess"));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t("features.settings.exportPrivateKeyFailed"));
    } finally {
      setLoading(false);
    }
  };

  const requestExportKeystore = (wallet: SavedWallet) => {
    setPasswordPromptValue("");
    setPasswordPrompt({ kind: "export-keystore", wallet, formState: {} });
  };

  const requestExportPrivateKey = (wallet: SavedWallet) => {
    setPasswordPromptValue("");
    setPasswordPrompt({ kind: "export-private-key", wallet, formState: {} });
  };

  const handleDeleteWallet = async (wallet: SavedWallet) => {
    const confirmed = window.confirm(
      t("features.settings.deleteWalletConfirm", { name: wallet.name }),
    );
    if (!confirmed) return;

    setLoading(true);
    try {
      const response = await apiFetch(`wallets/${wallet.id}/delete`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({}),
      });
      const data = await response.json();
      if (!response.ok) {
        throw new Error(data.error || t("features.settings.deleteWalletFailed"));
      }
      if (currentWalletId === wallet.id) {
        setCurrentWallet("");
      }
      setFormData((prev) => {
        const next = { ...prev };
        if (next.wallet_id === wallet.id) {
          delete next.wallet_id;
        }
        delete next[`walletName:${wallet.id}`];
        delete next[`exportPassword:${wallet.id}`];
        return next;
      });
      setEditingWalletId((prev) => (prev === wallet.id ? null : prev));
      await loadWallets();
      toast.success(t("features.settings.deleteWalletSuccess"));
    } catch (err) {
      toast.error(err instanceof Error ? err.message : t("features.settings.deleteWalletFailed"));
    } finally {
      setLoading(false);
    }
  };


  const toggleMenu = (menuId: string) => {
    setActiveMenu(activeMenu === menuId ? null : menuId);
  };

  const openParentMenuForForm = (formId: string) => {
    const parentMenu = menuItems.find((item) => item.children?.some((child) => child.id === formId));
    if (parentMenu) {
      setActiveMenu(parentMenu.id);
    }
  };

  const handleFormChange = (field: string, value: string | number | undefined) => {
    if (field === "password" || field === "master_password") {
      return;
    }
    if (field === "mint" && String(formData.mint ?? "").trim() !== String(value ?? "").trim()) {
      setTokenActionContext(null);
    }
    setFormData((prev) => {
      if (value === undefined) {
        const next = { ...prev };
        delete next[field];
        if (field === "mint") {
          delete next.decimals;
          delete next.tokenSupply;
          delete next.tokenUiSupply;
          delete next.token_balance;
          delete next.token_raw_amount;
          delete next.token_account;
          delete next.sell_percent;
        }
        return next;
      }
      if (field === "mint" && String(prev.mint ?? "").trim() !== String(value ?? "").trim()) {
        const next: FormState = { ...prev, [field]: value };
        delete next.decimals;
        delete next.tokenSupply;
        delete next.tokenUiSupply;
        delete next.token_balance;
        delete next.token_raw_amount;
        delete next.token_account;
        if (selectedForm === "pumpfun-sell" || selectedForm === "pumpswap-sell") {
          delete next.amount;
          delete next.sell_percent;
        }
        return next;
      }
      const next: FormState = { ...prev, [field]: value };
      if (field === "amount" && (selectedForm === "pumpfun-sell" || selectedForm === "pumpswap-sell")) {
        delete next.sell_percent;
      }
      return next;
    });
  };

  const handleSellPercentShortcut = async (
    formId: "pumpfun-sell" | "pumpswap-sell",
    percent: number,
  ) => {
    const mint = String(formData.mint ?? "").trim();
    if (!mint) {
      toast.error(t(`features.${formId}.fillMintFirst`));
      return;
    }

    const wallet = selectedSavedWallet() ?? effectiveWallet;
    if (!wallet) {
      toast.error(t(`features.${formId}.balanceNotFound`));
      return;
    }

    const matchesWallet = walletAssets?.address === wallet.public_key && walletAssets.network === effectiveNetwork;
    let assets = matchesWallet && !walletAssets.loading ? walletAssets : null;
    let tokenBalance =
      tokenBalanceSnapshotFromFormState(formData, mint) ||
      tokenBalanceSnapshotFromFormState(tokenActionContext, mint) ||
      (assets ? aggregateTokenBalance(assets.tokens, mint) : null);

    if (!tokenBalance) {
      const refreshed = await loadWalletAssets(wallet, { refresh: true, force: true });
      if (!refreshed) {
        toast.error(t("features.wallet-list.assetLoadFailed"));
        return;
      }
      assets = refreshed;
      tokenBalance = aggregateTokenBalance(assets.tokens, mint);
    }

    if (!tokenBalance) {
      toast.error(t(`features.${formId}.balanceNotFound`));
      return;
    }

    const rawBalance = BigInt(tokenBalance.rawAmount);
    const rawAmount =
      percent >= 100
        ? rawBalance
        : (rawBalance * BigInt(percent)) / BigInt(100);
    if (rawAmount <= BigInt(0)) {
      toast.error(t(`features.${formId}.amountTooSmall`));
      return;
    }

    const amount = rawTokenAmountToUi(rawAmount.toString(), tokenBalance.decimals);
    if (!parsePositiveDecimal(amount)) {
      toast.error(t(`features.${formId}.amountTooSmall`));
      return;
    }
    setFormData((prev) => ({
      ...prev,
      amount,
      sell_percent: percent * 100,
    }));
  };

  const refreshWalletAfterMutation = useCallback((
    wallet?: SavedWallet,
    options: { tokenAdjustment?: TokenBalanceAdjustment; refreshDelaysMs?: number[]; formSnapshot?: FormState } = {},
  ) => {
    if (!wallet) return;
    const walletKey = `${wallet.public_key}:${effectiveRpcRequest}`;
    const syncCurrentTokenBalance = (assets: WalletAssetsState) => {
      setTokenActionContext((prev) => {
        const mint = String(prev?.mint ?? "").trim();
        if (!prev || !mint) return prev;
        const balance = aggregateTokenBalance(assets.tokens, mint);
        if (!balance) {
          return {
            ...prev,
            amount: "0",
            token_balance: "0",
            token_raw_amount: "0",
          };
        }
        return {
          ...prev,
          amount: balance.amount,
          token_balance: balance.amount,
          token_raw_amount: balance.rawAmount,
          decimals: balance.decimals,
        };
      });
      if (!selectedForm || !CURRENT_WALLET_TOKEN_BALANCE_FORM_IDS.has(selectedForm)) {
        return;
      }
      setFormData((prev) => {
        const mint = String(prev.mint ?? "").trim();
        if (!mint) return prev;
        const balance = aggregateTokenBalance(assets.tokens, mint);
        if (!balance) {
          return {
            ...prev,
            token_balance: "0",
            token_raw_amount: "0",
          };
        }
        return {
          ...prev,
          token_balance: balance.amount,
          token_raw_amount: balance.rawAmount,
          decimals: balance.decimals,
        };
      });
    };
    if (options.tokenAdjustment) {
      const fallbackPendingAdjustment = pendingAdjustmentFromFormState(
        options.formSnapshot ?? formData,
        options.tokenAdjustment,
      );
      setWalletAssets((prev) => {
        if (!prev || prev.address !== wallet.public_key || prev.network !== effectiveNetwork) {
          pendingTokenBalanceAdjustmentRef.current = fallbackPendingAdjustment;
          return prev;
        }
        const pendingAdjustment = pendingAdjustmentFromAssets(prev, options.tokenAdjustment);
        pendingTokenBalanceAdjustmentRef.current = pendingAdjustment ?? fallbackPendingAdjustment;
        const adjustedAssets = applyPendingTokenBalanceAdjustmentToAssets(
          prev,
          pendingAdjustment ?? fallbackPendingAdjustment,
        );
        syncCurrentTokenBalance(adjustedAssets);
        return adjustedAssets;
      });
      if (fallbackPendingAdjustment) {
        const fallbackBalance = rawTokenAmountToUi(
          fallbackPendingAdjustment.expectedRawAmount,
          fallbackPendingAdjustment.decimals,
        );
        setTokenActionContext((prev) => {
          const mint = String(prev?.mint ?? "").trim();
          if (!prev || mint !== fallbackPendingAdjustment.mint) return prev;
          return {
            ...prev,
            amount: fallbackBalance,
            token_balance: fallbackBalance,
            token_raw_amount: fallbackPendingAdjustment.expectedRawAmount,
            decimals: fallbackPendingAdjustment.decimals,
          };
        });
        setFormData((prev) => {
          const mint = String(prev.mint ?? "").trim();
          if (mint !== fallbackPendingAdjustment.mint) return prev;
          return {
            ...prev,
            token_balance: fallbackBalance,
            token_raw_amount: fallbackPendingAdjustment.expectedRawAmount,
            decimals: fallbackPendingAdjustment.decimals,
          };
        });
      }
    }
    const refreshAssets = () =>
      loadWalletAssets(wallet, { refresh: true, background: true, force: true }).then((assets) => {
        if (assets) syncCurrentTokenBalance(assets);
      });
    const refreshDelaysMs = options.refreshDelaysMs ?? DEFAULT_POST_MUTATION_ASSET_REFRESH_DELAYS_MS;
    lastAssetRefreshRef.current.delete(walletKey);
    refreshDelaysMs.forEach((delay) => {
      window.setTimeout(() => {
        walletAssetsInFlightRef.current.delete(`${wallet.public_key}:${effectiveRpcRequest}:refresh`);
        lastAssetRefreshRef.current.delete(walletKey);
        void refreshAssets();
      }, delay);
    });
    lastTransactionRefreshRef.current.delete(walletKey);
    if (walletOverviewTab === "transactions") {
      void loadWalletTransactions(wallet, "replace", { force: true });
    }
  }, [effectiveNetwork, effectiveRpcRequest, formData, loadWalletAssets, loadWalletTransactions, selectedForm, walletOverviewTab]);

  const clearForm = () => {
    setFormData({ network: effectiveNetwork });
  };

  const defaultFormPreset = (formId: string): FormState => {
    if (formId === "pumpfun-sell") {
      return { slippage: 1 };
    }
    if (formId === "create-nonce") {
      return { count: 1 };
    }
    return {};
  };

  const refreshCurrentWalletAssets = (wallet?: SavedWallet) => {
    const targetWallet = wallet ?? selectedSavedWallet() ?? effectiveWallet;
    if (!targetWallet) return;
    void loadWalletAssets(targetWallet, { refresh: true, background: true, force: true }).then((assets) => {
      if (!assets) return;
      const mint = String((tokenActionContext?.mint ?? formData.mint) || "").trim();
      if (!mint) return;
      const balance = aggregateTokenBalance(assets.tokens, mint);
      if (!balance) return;
      setTokenActionContext((prev) => {
        if (!prev || String(prev.mint ?? "").trim() !== mint) return prev;
        return {
          ...prev,
          amount: balance.amount,
          token_balance: balance.amount,
          token_raw_amount: balance.rawAmount,
          decimals: balance.decimals,
        };
      });
      if (selectedForm && CURRENT_WALLET_TOKEN_BALANCE_FORM_IDS.has(selectedForm)) {
        setFormData((prev) => ({
          ...prev,
          token_balance: balance.amount,
          token_raw_amount: balance.rawAmount,
          decimals: balance.decimals,
        }));
      }
    });
  };

  const handleSelectForm = (formId: string) => {
    setMobileMenuOpen(false);
    clearForm();
    setTokenActionContext(null);
    setNonceCreateOpen(false);
    if (formId !== "create-nonce") {
      setCreatedNonceAccounts([]);
    }
    openParentMenuForForm(formId);
    setSelectedForm(formId);
    setBackTarget(defaultBackTarget(formId));
    if (authFormsWithWallets.has(formId)) {
      setAuthMethod({ ...authMethod, [formId]: "keystore" });
      if (effectiveWalletId) {
        setFormData({ wallet_id: effectiveWalletId, network: effectiveNetwork, ...defaultFormPreset(formId) });
      } else {
        setFormData({ network: effectiveNetwork, ...defaultFormPreset(formId) });
      }
    } else {
      setFormData({ network: effectiveNetwork, ...defaultFormPreset(formId) });
    }
  };

  const handleOpenForm = (formId: string, preset: FormState = {}, sourceForm?: string | null) => {
    setTokenActionContext(null);
    setNonceCreateOpen(false);
    if (formId !== "create-nonce") {
      setCreatedNonceAccounts([]);
    }
    openParentMenuForForm(defaultBackTarget(formId) ?? formId);
    setBackTarget(sourceForm === undefined ? selectedForm ?? defaultBackTarget(formId) : sourceForm);
    setSelectedForm(formId);
    setFormData({
      ...(authFormsWithWallets.has(formId) && effectiveWalletId ? { wallet_id: effectiveWalletId } : {}),
      network: effectiveNetwork,
      ...defaultFormPreset(formId),
      ...preset,
    });
    if (authFormsWithWallets.has(formId)) {
      setAuthMethod((prev) => ({ ...prev, [formId]: "keystore" }));
    }
  };

  const tokenActionPreset = (token: WalletTokenAsset): FormState => ({
    wallet_id: effectiveWalletId,
    network: effectiveNetwork,
    mint: token.mint,
    amount: token.ui_amount_string,
    token_balance: token.ui_amount_string,
    token_raw_amount: token.amount,
    decimals: token.decimals,
    token_account: token.account,
    sell_percent: 10000,
  });

  const openTokenActions = (token: WalletTokenAsset) => {
    const preset = tokenActionPreset(token);
    setCreatedNonceAccounts([]);
    setTokenActionContext(preset);
    openParentMenuForForm("wallet-list");
    setBackTarget("wallet-list");
    setSelectedForm("pumpfun-sell");
    setFormData({
      network: effectiveNetwork,
      ...defaultFormPreset("pumpfun-sell"),
      ...preset,
    });
    setAuthMethod((prev) => ({
      ...prev,
      "pumpfun-sell": "keystore",
      "transfer-token": "keystore",
    }));
  };

  const switchTokenAction = (formId: "pumpfun-sell" | "transfer-token") => {
    const context: FormState = tokenActionContext ?? {};
    const tokenBalance = String(context.token_balance ?? context.amount ?? formData.token_balance ?? formData.amount ?? "");
    const currentAmount = String(formData.amount ?? "").trim();
    const next: FormState = {
      wallet_id: formData.wallet_id || context.wallet_id || effectiveWalletId,
      network: formData.network || context.network || effectiveNetwork,
      mint: formData.mint || context.mint,
      amount: currentAmount || tokenBalance,
      token_balance: context.token_balance || tokenBalance,
      token_raw_amount: context.token_raw_amount || formData.token_raw_amount,
      token_account: context.token_account || formData.token_account,
      decimals: formData.decimals ?? context.decimals,
      ...defaultFormPreset(formId),
    };

    if (formId === "pumpfun-sell") {
      if (String(next.amount ?? "") === tokenBalance) {
        next.sell_percent = 10000;
      }
      next.slippage = formData.slippage ?? context.slippage ?? next.slippage;
    } else {
      delete next.sell_percent;
      delete next.slippage;
      next.to_address = formData.to_address;
    }

    setTokenActionContext((prev) => prev || context);
    setBackTarget("wallet-list");
    setSelectedForm(formId);
    setFormData(next);
    setAuthMethod((prev) => ({
      ...prev,
      [formId]: prev[formId] || "keystore",
    }));
  };

  const openNonceCreateDialog = () => {
    setCreatedNonceAccounts([]);
    setAuthMethod((prev) => ({ ...prev, "create-nonce": "keystore" }));
    setFormData({
      wallet_id: effectiveWalletId,
      network: effectiveNetwork,
      ...defaultFormPreset("create-nonce"),
    });
    setNonceCreateOpen(true);
  };

  const closeNonceCreateDialog = () => {
    if (loading) return;
    setNonceCreateOpen(false);
  };

  const handleOpenSquadsForm = (formId: string, preset: FormState = {}) => {
    setBackTarget(selectedForm ?? "squads-workspace");
    setSelectedForm(formId);
    const method = walletAuth("squads-workspace");
    const defaultStepIndex = new Set([
      "squads-sol-transfer",
      "squads-token-transfer",
      "squads-prepare-upgrade-buffer",
      "squads-program-upgrade",
    ]).has(formId)
      ? 1
      : 0;
    setAuthMethod((prev) => ({ ...prev, "squads-workspace": method, [formId]: method }));
    setFormData((prev) => {
      const defaultMembers =
        formId === "squads-create"
          ? uniqueAddressList([preset.members, prev.members, effectiveWallet?.public_key]).join("\n")
          : undefined;
      const next: FormState = {
        wallet_id: prev.wallet_id || effectiveWalletId,
        keystoreJson: prev.keystoreJson,
        encrypted_key: prev.encrypted_key,
        private_key: prev.private_key,
        network: preset.network || prev.network || effectiveNetwork,
        stepIndex: defaultStepIndex,
        ...(formId === "squads-create"
          ? { members: defaultMembers, threshold: preset.threshold || prev.threshold || "1" }
          : {}),
        ...preset,
      };
      return next;
    });
  };

  const walletAuthFormData = (base: FormState = formData): FormState => ({
    ...base,
    wallet_id: String(base.wallet_id ?? effectiveWalletId).trim() || undefined,
    network: base.network || effectiveNetwork,
  });

  const validateBeforePasswordPrompt = (formId: string, nextFormData: FormState): boolean => {
    const fail = (message: string) => {
      toast.error(message);
      return false;
    };
    const amount = parsePositiveDecimal(nextFormData.amount);

    switch (formId) {
      case "transfer-sol":
        return nextFormData.to_address && amount !== null
          ? true
          : fail(t("features.transfer-sol.fillAllFields"));
      case "transfer-token":
        return nextFormData.to_address && nextFormData.mint && amount !== null
          ? true
          : fail(t("features.transfer-token.fillAllFields"));
      case "wrap-sol":
        return amount !== null ? true : fail(t("features.wrap-sol.fillAllFields"));
      case "program-deploy": {
        if (!nextFormData.programSoBase64) {
          return fail(t("features.program-deploy.selectFileFirst"));
        }
        const maxDataLen =
          nextFormData.max_data_len === undefined || nextFormData.max_data_len === ""
            ? undefined
            : parseInt(String(nextFormData.max_data_len), 10);
        return maxDataLen === undefined ||
          (Number.isInteger(maxDataLen) && maxDataLen > 0 && maxDataLen <= MAX_PROGRAM_SO_FILE_BYTES)
          ? true
          : fail(t("features.program-deploy.invalidMaxDataLen"));
      }
      case "squads-create": {
        const members = parseAddressList(nextFormData.members);
        const threshold = parseInt(String(nextFormData.threshold || ""), 10);
        const signer = String(effectiveWallet?.public_key ?? "").trim();
        const timeLock =
          nextFormData.time_lock === undefined || nextFormData.time_lock === ""
            ? undefined
            : parseInt(String(nextFormData.time_lock), 10);
        if (members.length === 0 || !Number.isInteger(threshold)) {
          return fail(t("features.squads-create.fillAllFields"));
        }
        if (threshold <= 0 || threshold > members.length) {
          return fail(t("features.squads-create.invalidThreshold"));
        }
        if (signer && !members.includes(signer)) {
          return fail(t("features.squads-create.signerMustBeMember"));
        }
        return timeLock === undefined || (Number.isInteger(timeLock) && timeLock >= 0)
          ? true
          : fail(t("features.squads-create.invalidTimeLock"));
      }
      case "squads-sol-transfer":
        return nextFormData.multisig && nextFormData.to_address && amount !== null
          ? true
          : fail(t("features.squads-sol-transfer.fillAllFields"));
      case "squads-token-transfer": {
        return nextFormData.multisig &&
          nextFormData.mint &&
          nextFormData.recipient &&
          amount !== null
          ? true
          : fail(t("features.squads-token-transfer.fillAllFields"));
      }
      case "squads-prepare-upgrade-buffer":
        return nextFormData.multisig && nextFormData.programSoBase64
          ? true
          : fail(t("features.squads-prepare-upgrade-buffer.fillAllFields"));
      case "squads-program-upgrade":
        return nextFormData.multisig && nextFormData.programId && nextFormData.bufferAddress
          ? true
          : fail(t("features.squads-program-upgrade.fillAllFields"));
      case "squads-set-authority":
        return nextFormData.multisig && nextFormData.programId
          ? true
          : fail(t("features.squads-set-authority.fillAllFields"));
      case "squads-approve":
      case "squads-reject":
        return nextFormData.multisig && nextFormData.proposal
          ? true
          : fail(t("features.squads-vote.fillAllFields"));
      case "squads-execute": {
        const transactionIndex = parseInt(String(nextFormData.transactionIndex || ""), 10);
        return nextFormData.multisig &&
          nextFormData.proposal &&
          Number.isInteger(transactionIndex)
          ? true
          : fail(t("features.squads-execute.fillAllFields"));
      }
      case "pumpfun-sell":
        return nextFormData.mint && hasValidSellAmount(nextFormData)
          ? true
          : fail(t("features.pumpfun-sell.fillAllFields"));
      case "pumpswap-sell":
        return nextFormData.mint && hasValidSellAmount(nextFormData)
          ? true
          : fail(t("features.pumpswap-sell.fillAllFields"));
      case "setup-2fa":
        return nextFormData.hardware_fingerprint
          ? true
          : fail(t("features.setup-2fa.fillAllFields"));
      case "create-tfa": {
        const method = walletAuth("create-tfa");
        const hasWalletMaterial =
          method === "keystore"
            ? String(nextFormData.wallet_id ?? "").trim() || String(nextFormData.keystoreJson ?? "").trim()
            : method === "encrypted"
              ? String(nextFormData.encrypted_key ?? nextFormData.encryptedKey ?? "").trim()
              : String(nextFormData.private_key ?? "").trim();
        const questionIndex =
          typeof nextFormData.question_index === "number"
            ? nextFormData.question_index
            : parseInt(String(nextFormData.question_index ?? ""), 10);
        if (
          !hasWalletMaterial ||
          !nextFormData.totp_secret ||
          !nextFormData.hardware_fingerprint ||
          nextFormData.question_index === undefined ||
          !nextFormData.security_answer
        ) {
          return fail(t("features.create-tfa.fillAllFields"));
        }
        return Number.isInteger(questionIndex) && questionIndex >= 0 && questionIndex <= 7
          ? true
          : fail(t("features.create-tfa.questionIndexError"));
      }
      case "unlock-tfa":
        return nextFormData.encrypted_wallet &&
          nextFormData.hardware_fingerprint &&
          nextFormData.security_answer &&
          nextFormData.totp_code
          ? true
          : fail(t("features.unlock-tfa.fillAllFields"));
      default:
        return true;
    }
  };

  const shouldPromptForWalletPassword = (formId: string): boolean => {
    if (!WALLET_PASSWORD_FORM_IDS.has(formId)) return false;
    const method = walletAuth(formId);
    return method === "keystore" || method === "encrypted";
  };

  const shouldPromptForMasterPassword = (formId: string): boolean =>
    MASTER_PASSWORD_FORM_IDS.has(formId);

  const requestCreatePasswordSubmit = (formId: string) => {
    if (!CREATE_PASSWORD_FORM_IDS.has(formId)) {
      void handleSubmit(formId);
      return;
    }

    const nextFormData = { ...formData };
    if (formId === "create-keystore" && !String(nextFormData.name ?? "").trim()) {
      toast.error(t("formUi.walletNameRequired"));
      return;
    }
    if (formId === "import-keystore" && !String(nextFormData.keystoreJson ?? "").trim()) {
      toast.error(t("features.import-keystore.fillAllFields"));
      return;
    }

    setPasswordPromptValue("");
    setMasterPasswordPromptValue("");
    setPasswordPrompt({ kind: "create-password", formId, formState: nextFormData });
  };

  const requestPasswordSubmit = (formId: string) => {
    const needsWalletPassword = shouldPromptForWalletPassword(formId);
    const needsMasterPassword = shouldPromptForMasterPassword(formId);

    if (!needsWalletPassword && !needsMasterPassword) {
      void handleSubmit(formId);
      return;
    }

    const nextFormData = walletAuthFormData();
    const method = walletAuth(formId);
    if (needsWalletPassword &&
      method === "keystore" &&
      !String(nextFormData.wallet_id ?? "").trim() &&
      !String(nextFormData.keystoreJson ?? "").trim()
    ) {
      toast.error(t("features.walletContext.noWallet"));
      return;
    }
    if (
      needsWalletPassword &&
      method === "encrypted" &&
      !String(nextFormData.encrypted_key ?? nextFormData.encryptedKey ?? "").trim()
    ) {
      toast.error(t("features.decrypt.fillAllFields"));
      return;
    }
    if (!validateBeforePasswordPrompt(formId, nextFormData)) {
      return;
    }

    setFormData(nextFormData);
    setPasswordPromptValue("");
    setMasterPasswordPromptValue("");
    setPasswordPrompt({
      kind: "form",
      formId,
      formState: nextFormData,
      fields: [
        ...(needsWalletPassword ? (["password"] as PasswordPromptField[]) : []),
        ...(needsMasterPassword ? (["master_password"] as PasswordPromptField[]) : []),
      ],
    });
  };

  const requestProposalPasswordSubmit = (
    proposal: WorkspaceProposal,
    action: WorkspaceProposalAction,
  ) => {
    if (walletAuth("squads-workspace") !== "keystore") {
      void handleWorkspaceProposalAction(proposal, action);
      return;
    }

    const nextFormData = walletAuthFormData();
    if (!String(nextFormData.wallet_id ?? "").trim() && !String(nextFormData.keystoreJson ?? "").trim()) {
      toast.error(t("features.walletContext.noWallet"));
      return;
    }

    setFormData(nextFormData);
    setPasswordPromptValue("");
    setMasterPasswordPromptValue("");
    setPasswordPrompt({ kind: "proposal", proposal, action, formState: nextFormData });
  };

  const closePasswordPrompt = () => {
    if (loading) return;
    setPasswordPrompt(null);
    setPasswordPromptValue("");
    setMasterPasswordPromptValue("");
  };

  const passwordPromptFields = passwordPrompt?.kind === "form"
    ? passwordPrompt.fields ?? ["password"]
    : passwordPrompt?.kind === "master-password"
      ? (["master_password"] as PasswordPromptField[])
      : (["password"] as PasswordPromptField[]);
  const showWalletPasswordPrompt = passwordPromptFields.includes("password");
  const showMasterPasswordPrompt = passwordPromptFields.includes("master_password");

  const passwordPromptTitle =
    passwordPrompt?.kind === "export-keystore"
      ? t("features.settings.exportPasswordTitle")
      : passwordPrompt?.kind === "export-private-key"
        ? t("features.settings.exportPrivateKeyTitle")
      : passwordPrompt?.kind === "create-password"
        ? t("formUi.passwordPromptTitle")
      : showWalletPasswordPrompt && showMasterPasswordPrompt
        ? t("formUi.passwordsPromptTitle")
      : showMasterPasswordPrompt
        ? t("formUi.masterPasswordPromptTitle")
      : passwordPrompt?.kind === "master-password"
        ? t("formUi.masterPasswordPromptTitle")
      : t("formUi.confirmPasswordTitle");
  const passwordPromptHint =
    passwordPrompt?.kind === "export-keystore"
      ? t("features.settings.exportPasswordHint")
      : passwordPrompt?.kind === "export-private-key"
        ? t("features.settings.exportPrivateKeyHint")
      : passwordPrompt?.kind === "create-password"
        ? t("formUi.passwordPromptHint")
      : showWalletPasswordPrompt && showMasterPasswordPrompt
        ? t("formUi.passwordsPromptHint")
      : showMasterPasswordPrompt
        ? t("formUi.masterPasswordPromptHint")
      : passwordPrompt?.kind === "master-password"
        ? t("formUi.masterPasswordPromptHint")
      : t("formUi.confirmPasswordHint");
  const passwordPromptButton =
    passwordPrompt?.kind === "export-keystore"
      ? t("features.settings.exportPasswordButton")
      : passwordPrompt?.kind === "export-private-key"
        ? t("features.settings.exportPrivateKeyButton")
      : passwordPrompt?.kind === "create-password"
        ? t("formUi.passwordPromptButton")
      : showWalletPasswordPrompt && showMasterPasswordPrompt
        ? t("formUi.passwordsPromptButton")
      : showMasterPasswordPrompt
        ? t("formUi.masterPasswordPromptButton")
      : passwordPrompt?.kind === "master-password"
        ? t("formUi.masterPasswordPromptButton")
      : t("formUi.confirmPasswordButton");

  const confirmPasswordPrompt = async () => {
    if (!passwordPrompt) return;
    const password = passwordPromptValue;
    const masterPassword = masterPasswordPromptValue;
    if (showWalletPasswordPrompt && password.length === 0) {
      toast.error(t("formUi.confirmPasswordRequired"));
      return;
    }
    if (showMasterPasswordPrompt && masterPassword.length === 0) {
      toast.error(t("formUi.masterPasswordRequired"));
      return;
    }

    if (passwordPrompt.kind === "form") {
      const nextFormData = walletAuthFormData({
        ...passwordPrompt.formState,
        ...(showWalletPasswordPrompt ? { password } : {}),
        ...(showMasterPasswordPrompt ? { master_password: masterPassword } : {}),
      });
      await handleSubmit(passwordPrompt.formId, nextFormData);
    } else if (passwordPrompt.kind === "create-password") {
      await handleSubmit(passwordPrompt.formId, { ...passwordPrompt.formState, password });
    } else if (passwordPrompt.kind === "master-password") {
      await handleSubmit(passwordPrompt.formId, {
        ...passwordPrompt.formState,
        master_password: masterPassword,
      });
    } else if (passwordPrompt.kind === "proposal") {
      const nextFormData = walletAuthFormData({ ...passwordPrompt.formState, password });
      await handleWorkspaceProposalAction(passwordPrompt.proposal, passwordPrompt.action, nextFormData);
    } else if (passwordPrompt.kind === "export-keystore") {
      await handleExportKeystore(passwordPrompt.wallet, password);
    } else {
      await handleExportPrivateKey(passwordPrompt.wallet, password);
    }

    setPasswordPrompt(null);
    setPasswordPromptValue("");
    setMasterPasswordPromptValue("");
    setFormData((prev) => {
      const next = { ...prev };
      delete next.password;
      delete next.master_password;
      return next;
    });
  };

  const handleBack = () => {
    const target = backTarget || (selectedForm ? defaultBackTarget(selectedForm) : null);
    if (!target) return;
    if (target === "wallet-list") {
      setTokenActionContext(null);
    }
    setSelectedForm(target);
    setBackTarget(defaultBackTarget(target));
    setFormData(() => ({
      network: effectiveNetwork,
      ...defaultFormPreset(target),
      ...(authFormsWithWallets.has(target) && effectiveWalletId ? { wallet_id: effectiveWalletId } : {}),
    }));
  };

  const copyToClipboard = async (text: string, id: string) => {
    if (!text) {
      toast.error(t("errors.copyFailed"));
      return;
    }
    try {
      if (navigator.clipboard?.writeText && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
      } else {
        const textarea = document.createElement("textarea");
        textarea.value = text;
        textarea.setAttribute("readonly", "true");
        textarea.style.position = "fixed";
        textarea.style.left = "-9999px";
        textarea.style.top = "0";
        document.body.appendChild(textarea);
        textarea.select();
        textarea.setSelectionRange(0, text.length);
        const copied = document.execCommand("copy");
        document.body.removeChild(textarea);
        if (!copied) {
          throw new Error("copy failed");
        }
      }
      setCopied(id);
      toast.success(t("common.copiedToClipboard"));
      setTimeout(() => setCopied(null), 2000);
    } catch {
      toast.error(t("errors.copyFailed"));
    }
  };

  const downloadFile = (content: string, filename: string, type = "application/json") => {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    toast.success(t("common.downloaded", { filename }));
  };

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      if (file.size > MAX_KEYSTORE_FILE_BYTES) {
        toast.error(t("errors.keystoreFileTooLarge"));
        event.target.value = "";
        return;
      }
      if (file.type && file.type !== "application/json" && !file.name.toLowerCase().endsWith(".json")) {
        toast.error(t("errors.invalidKeystoreFile"));
        event.target.value = "";
        return;
      }
      const reader = new FileReader();
      reader.onload = (e) => {
        const content = e.target?.result as string;
        setFormData((prev) => ({
          ...prev,
          keystoreJson: content,
          ...(selectedForm === "import-keystore" && !String(prev.name ?? "").trim()
            ? { name: keystoreMetadataName(content) }
            : {}),
        }));
        toast.success(t("features.import-keystore.fileUploaded"));
      };
      reader.readAsText(file);
    }
  };

  const handleProgramFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    if (file.size > MAX_PROGRAM_SO_FILE_BYTES) {
      toast.error(t("errors.programFileTooLarge"));
      event.target.value = "";
      return;
    }
    if (!file.name.toLowerCase().endsWith(".so")) {
      toast.error(t("errors.invalidProgramFile"));
      event.target.value = "";
      return;
    }
    const reader = new FileReader();
    reader.onload = (e) => {
      const result = e.target?.result;
      if (!(result instanceof ArrayBuffer)) {
        toast.error(t("errors.invalidProgramFile"));
        return;
      }
      const bytes = new Uint8Array(result);
      let binary = "";
      for (const byte of bytes) {
        binary += String.fromCharCode(byte);
      }
      handleFormChange("programSoBase64", btoa(binary));
      handleFormChange("programSoName", file.name);
      handleFormChange("programSoSize", file.size);
      toast.success(t("features.program-deploy.fileUploaded"));
    };
    reader.readAsArrayBuffer(file);
  };

  const handleSubmit = async (formId: string, submitFormData: FormState = formData) => {
    const formData = submitFormData;
    const submitNetwork = () => requestNetwork(formData.network);
    setLoading(true);

    try {
      switch (formId) {
        case "create-plain": {
          const response = await apiFetch("keys/create", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name: formData.name || "default" }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.create-plain.success"));
            setFormData((prev) => {
              const next: FormState = {
                ...prev,
                publicKey: data.public_key,
              };
              delete next.secretKey;
              delete next.private_key;
              return next;
            });
          } else {
            toast.error(data.error || t("errors.createFailed"));
          }
          break;
        }

        case "create-encrypted": {
          const password = String(formData.password || "");
          if (password.length < 10 || password.length > 20) {
            toast.error(t("errors.passwordLength"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("keys/create-encrypted", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ password }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.create-encrypted.success"));
            setFormData((prev) => ({
              ...prev,
              publicKey: data.public_key,
              encryptedKey: data.encrypted_key,
            }));
          } else {
            toast.error(data.error || t("features.create-encrypted.encryptFailed"));
          }
          break;
        }

        case "create-keystore": {
          const password = String(formData.password || "");
          const name = String(formData.name || "").trim();
          if (!name) {
            toast.error(t("formUi.walletNameRequired"));
            setLoading(false);
            return;
          }
          if (!password || password.length < 10) {
            toast.error(t("features.create-keystore.passwordError"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("keys/create-keystore", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              password,
              name,
            }),
          });
          const data = await response.json();

          if (response.ok) {
            let walletId: string | undefined;
            const saveResponse = await apiFetch("wallets", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({
                keystore_json: data.keystore_json,
                password,
                name,
              }),
            });
            const saveData = await saveResponse.json();
            if (saveResponse.ok) {
              walletId = saveData.wallet?.id;
              if (walletId) {
                setCurrentWallet(walletId);
              }
              await loadWallets();
              toast.success(t("features.create-keystore.success"));
            } else {
              toast.error(saveData.error || t("features.create-keystore.saveFailed"));
            }
            setFormData((prev) => ({
              ...prev,
              publicKey: data.public_key,
              keystoreJson: data.keystore_json,
              wallet_id: walletId,
              name,
            }));
          } else {
            toast.error(data.error || t("errors.createFailed"));
          }
          break;
        }

        case "import-keystore": {
          const password = String(formData.password || "");
          if (!formData.keystoreJson || !password) {
            toast.error(t("features.import-keystore.fillAllFields"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("wallets", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              keystore_json: formData.keystoreJson,
              password,
              name: formData.name || undefined,
            }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.import-keystore.success"));
            setCurrentWallet(data.wallet.id);
            await loadWallets();
            setFormData((prev) => {
              const next: FormState = {
                ...prev,
                publicKey: data.wallet.public_key,
                wallet_id: data.wallet.id,
              };
              delete next.password;
              delete next.keystoreJson;
              delete next.secretKey;
              delete next.private_key;
              delete next.name;
              return next;
            });
          } else {
            toast.error(data.error || t("features.import-keystore.error"));
          }
          break;
        }

        case "decrypt": {
          const dm = walletAuth("decrypt");
          if (!validateWalletAuth(dm, formData, "private_key")) {
            toast.error(t("features.decrypt.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {};
          applyWalletAuth(requestBody, dm, formData, "private_key");
          const response = await apiFetch("wallet/unlock", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.decrypt.success"));
            setFormData((prev) => {
              const next: FormState = {
                ...prev,
                publicKey: data.public_key,
                unlocked: data.unlocked ? "true" : undefined,
              };
              delete next.secretKey;
              delete next.private_key;
              return next;
            });
          } else {
            toast.error(data.error || t("features.decrypt.error"));
          }
          break;
        }

        case "unlock": {
          const um = walletAuth("unlock");
          if (!validateWalletAuth(um, formData, "secret_key")) {
            toast.error(t("features.unlock.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {};
          applyWalletAuth(requestBody, um, formData, "secret_key");
          const response = await apiFetch("wallet/unlock", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.unlock.success"));
            setFormData((prev) => {
              const next: FormState = {
                ...prev,
                publicKey: data.public_key,
                unlocked: data.unlocked ? "true" : undefined,
              };
              delete next.secretKey;
              delete next.private_key;
              return next;
            });
          } else {
            toast.error(data.error || t("features.unlock.error"));
          }
          break;
        }

        case "check-balance": {
          if (!formData.address) {
            toast.error(t("features.check-balance.enterAddress"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("wallet/balance", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              address: formData.address,
              network: submitNetwork(),
            }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(
              t("features.check-balance.success", {
                balance: data.balance,
                network: data.network,
              }),
            );
            setFormData((prev) => ({
              ...prev,
              balance: data.balance.toString(),
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.check-balance.error"));
          }
          break;
        }

        case "get-pubkey": {
          const method = walletAuth("get-pubkey");
          if (!validateWalletAuth(method, formData, "secret_key")) {
            toast.error(
              method === "keystore"
                ? t("features.get-pubkey.uploadKeystore")
                : method === "encrypted"
                  ? t("features.get-pubkey.enterEncrypted")
                  : t("features.get-pubkey.enterPrivateKey"),
            );
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {};
          applyWalletAuth(requestBody, method, formData, "secret_key");

          const response = await apiFetch("wallet/get-pubkey", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.get-pubkey.success"));
            setFormData((prev) => ({
              ...prev,
              publicKey: data.public_key,
            }));
          } else {
            toast.error(data.error || t("features.get-pubkey.error"));
          }
          break;
        }

        case "transfer-sol": {
          const m = walletAuth("transfer-sol");
          const amount = parsePositiveDecimal(formData.amount);
          if (!formData.to_address || amount === null) {
            toast.error(t("features.transfer-sol.fillAllFields"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.transfer-sol.fillAllFields"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            to_address: formData.to_address,
            amount,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("transfer/sol", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.transfer-sol.success", { signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
            }));
          } else {
            toast.error(data.error || t("features.transfer-sol.error"));
          }
          break;
        }

        case "transfer-token": {
          const m = walletAuth("transfer-token");
          const amount = parsePositiveDecimal(formData.amount);
          if (!formData.to_address || !formData.mint || amount === null) {
            toast.error(t("features.transfer-token.fillAllFields"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.transfer-token.fillAllFields"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            to_address: formData.to_address,
            mint: formData.mint,
            amount,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("transfer/token", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.transfer-token.success", { signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
            }));
          } else {
            toast.error(data.error || t("features.transfer-token.error"));
          }
          break;
        }

        case "create-wsol-ata": {
          const m = walletAuth("create-wsol-ata");
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(
              m === "keystore"
                ? t("features.create-wsol-ata.uploadKeystore")
                : m === "encrypted"
                  ? t("features.decrypt.fillAllFields")
                  : t("features.create-wsol-ata.enterPrivateKey"),
            );
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("wsol/create-ata", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.create-wsol-ata.success", { signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
            }));
          } else {
            toast.error(data.error || t("features.create-wsol-ata.error"));
          }
          break;
        }

        case "wrap-sol": {
          const m = walletAuth("wrap-sol");
          const amount = parsePositiveDecimal(formData.amount);
          if (amount === null) {
            toast.error(t("features.wrap-sol.fillAllFields"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.wrap-sol.fillAllFields"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            amount,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("wsol/wrap", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.wrap-sol.success", { signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
            }));
          } else {
            toast.error(data.error || t("features.wrap-sol.error"));
          }
          break;
        }

        case "unwrap-sol": {
          const m = walletAuth("unwrap-sol");
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(
              m === "keystore"
                ? t("features.unwrap-sol.uploadKeystore")
                : m === "encrypted"
                  ? t("features.decrypt.fillAllFields")
                  : t("features.unwrap-sol.enterPrivateKey"),
            );
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("wsol/unwrap", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.unwrap-sol.success", { signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
            }));
          } else {
            toast.error(data.error || t("features.unwrap-sol.error"));
          }
          break;
        }

        case "create-nonce": {
          const m = walletAuth("create-nonce");
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(
              m === "keystore"
                ? t("features.create-nonce.uploadKeystore")
                : m === "encrypted"
                  ? t("features.decrypt.fillAllFields")
                  : t("features.create-nonce.enterPrivateKey"),
            );
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            network: submitNetwork(),
            count: parseNonceCount(formData.count),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("nonce/create", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.create-nonce.success", { count: data.count ?? 1, signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              nonceAccount: data.nonce_account,
              signature: data.signature,
            }));
            setCreatedNonceAccounts(Array.isArray(data.nonce_accounts) ? data.nonce_accounts : []);
            setNonceCreateOpen(false);
            await loadNonceAccounts(savedWalletFromForm(formData) ?? effectiveWallet, { force: true });
          } else {
            toast.error(data.error || t("features.create-nonce.error"));
          }
          break;
        }

        case "program-deploy": {
          const m = walletAuth("program-deploy");
          if (!formData.programSoBase64) {
            toast.error(t("features.program-deploy.selectFileFirst"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.program-deploy.fillAllFields"));
            setLoading(false);
            return;
          }

          const maxDataLen =
            formData.max_data_len === undefined || formData.max_data_len === ""
              ? undefined
              : parseInt(String(formData.max_data_len), 10);
          if (
            maxDataLen !== undefined &&
            (!Number.isInteger(maxDataLen) || maxDataLen <= 0 || maxDataLen > MAX_PROGRAM_SO_FILE_BYTES)
          ) {
            toast.error(t("features.program-deploy.invalidMaxDataLen"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            program_so_base64: formData.programSoBase64,
            network: submitNetwork(),
            max_data_len: maxDataLen,
            squads_multisig: formData.squadsDeployMultisig,
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const actor = walletActor(m === "keystore" ? selectedSavedWallet() : undefined);

          const response = await apiFetch("program/deploy", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.program-deploy.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceProgram(data.program_id);
            if (data.vault) {
              saveWorkspaceMultisig(formData.squadsDeployMultisig, data.vault, undefined, undefined, actor);
            }
            setFormData((prev) => ({
              ...prev,
              programId: data.program_id,
              programdataAddress: data.programdata_address,
              bufferAddress: data.buffer_address,
              authority: data.authority,
              vault: data.vault,
              signature: data.deploy_signature,
              authoritySignature: data.authority_signature,
              writeCount: String(Array.isArray(data.write_signatures) ? data.write_signatures.length : 0),
              rentLamports: String(data.rent_lamports),
              programBytes: String(data.program_bytes),
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.program-deploy.error"));
          }
          break;
        }

        case "program-info": {
          if (!formData.programId) {
            toast.error(t("features.program-info.enterProgramId"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("program/info", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              program_id: formData.programId,
              network: submitNetwork(),
            }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.program-info.success"));
            if (data.exists) {
              saveWorkspaceProgram(data.program_id);
            }
            setFormData((prev) => ({
              ...prev,
              programId: data.program_id,
              programdataAddress: data.programdata_address,
              executable: data.executable ? "true" : "false",
              programExists: data.exists ? "true" : "false",
              programInfoChecked: "true",
              owner: data.owner,
              lamports: String(data.lamports),
              dataLen: String(data.data_len),
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.program-info.error"));
          }
          break;
        }

        case "squads-create": {
          const m = walletAuth("squads-create");
          const members = parseAddressList(formData.members);
          const threshold = parseInt(String(formData.threshold || ""), 10);
          const signer = String(effectiveWallet?.public_key ?? "").trim();
          const timeLock =
            formData.time_lock === undefined || formData.time_lock === ""
              ? undefined
              : parseInt(String(formData.time_lock), 10);
          if (!validateWalletAuth(m, formData, "private_key") || members.length === 0 || !Number.isInteger(threshold)) {
            toast.error(t("features.squads-create.fillAllFields"));
            setLoading(false);
            return;
          }
          if (threshold <= 0 || threshold > members.length) {
            toast.error(t("features.squads-create.invalidThreshold"));
            setLoading(false);
            return;
          }
          if (signer && !members.includes(signer)) {
            toast.error(t("features.squads-create.signerMustBeMember"));
            setLoading(false);
            return;
          }
          if (timeLock !== undefined && (!Number.isInteger(timeLock) || timeLock < 0)) {
            toast.error(t("features.squads-create.invalidTimeLock"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            members: members.join(","),
            threshold,
            time_lock: timeLock,
            memo: formData.memo,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const actor = walletActor(m === "keystore" ? selectedSavedWallet() : undefined);
          const response = await apiFetch("squads/create", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ ...requestBody, members }),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-create.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceMultisig(data.multisig, data.vault, undefined, undefined, actor);
            setFormData((prev) => ({
              ...prev,
              multisig: data.multisig,
              vault: data.vault,
              createKey: data.create_key,
              signature: data.signature,
              creationFeeLamports: String(data.creation_fee_lamports),
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-create.error"));
          }
          break;
        }

        case "squads-info": {
          if (!formData.multisig) {
            toast.error(t("features.squads-info.enterMultisig"));
            setLoading(false);
            return;
          }
          const response = await apiFetch("squads/info", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              multisig: formData.multisig,
              proposal: formData.proposal || undefined,
              network: submitNetwork(),
            }),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-info.success"));
            saveWorkspaceMultisig(data.multisig, data.vault);
            if (data.proposal?.address) {
              saveWorkspaceProposal(
                data.proposal.address,
                data.multisig,
                data.proposal.transaction_index,
                undefined,
                data.proposal.status,
              );
            }
            setFormData((prev) => ({
              ...prev,
              multisig: data.multisig,
              vault: data.vault,
              createKey: data.create_key,
              threshold: String(data.threshold),
              transactionIndex: String(data.transaction_index),
              membersText: Array.isArray(data.members)
                ? data.members.map((member: { key: string }) => member.key).join("\n")
                : "",
              proposalStatus: data.proposal?.status,
              proposalTxIndex: data.proposal ? String(data.proposal.transaction_index) : undefined,
              approvedText: Array.isArray(data.proposal?.approved) ? data.proposal.approved.join("\n") : undefined,
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-info.error"));
          }
          break;
        }

        case "squads-sol-transfer": {
          const m = walletAuth("squads-sol-transfer");
          const amount = parsePositiveDecimal(formData.amount);
          if (!validateWalletAuth(m, formData, "private_key") || !formData.multisig || !formData.to_address || amount === null) {
            toast.error(t("features.squads-sol-transfer.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            to_address: formData.to_address,
            amount,
            memo: formData.memo,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const actor = walletActor(m === "keystore" ? selectedSavedWallet() : undefined);
          const response = await apiFetch("squads/proposal/sol-transfer", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-sol-transfer.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceMultisig(formData.multisig, data.vault);
            saveWorkspaceProposal(
              data.proposal,
              formData.multisig,
              data.transaction_index,
              "sol-transfer",
              "active",
              undefined,
              actor,
            );
            setFormData((prev) => ({
              ...prev,
              vault: data.vault,
              transaction: data.transaction,
              proposal: data.proposal,
              transactionIndex: String(data.transaction_index),
              signature: data.signature,
              transactionKind: "sol-transfer",
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-sol-transfer.error"));
          }
          break;
        }

        case "squads-token-transfer": {
          const m = walletAuth("squads-token-transfer");
          const amount = parsePositiveDecimal(formData.amount);
          if (
            !validateWalletAuth(m, formData, "private_key") ||
            !formData.multisig ||
            !formData.mint ||
            !formData.recipient ||
            amount === null
          ) {
            toast.error(t("features.squads-token-transfer.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            mint: formData.mint,
            recipient: formData.recipient,
            amount,
            memo: formData.memo,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const actor = walletActor(m === "keystore" ? selectedSavedWallet() : undefined);
          const response = await apiFetch("squads/proposal/token-transfer", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-token-transfer.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceMultisig(formData.multisig, data.vault);
            saveWorkspaceProposal(
              data.proposal,
              formData.multisig,
              data.transaction_index,
              "token-transfer",
              "active",
              undefined,
              actor,
            );
            setFormData((prev) => ({
              ...prev,
              vault: data.vault,
              transaction: data.transaction,
              proposal: data.proposal,
              transactionIndex: String(data.transaction_index),
              signature: data.signature,
              transactionKind: "token-transfer",
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-token-transfer.error"));
          }
          break;
        }

        case "squads-prepare-upgrade-buffer": {
          const m = walletAuth("squads-prepare-upgrade-buffer");
          if (!validateWalletAuth(m, formData, "private_key") || !formData.multisig || !formData.programSoBase64) {
            toast.error(t("features.squads-prepare-upgrade-buffer.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            program_so_base64: formData.programSoBase64,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const response = await apiFetch("squads/program/prepare-upgrade-buffer", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-prepare-upgrade-buffer.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceMultisig(data.multisig, data.vault);
            setFormData((prev) => ({
              ...prev,
              multisig: data.multisig,
              vault: data.vault,
              bufferAddress: data.buffer_address,
              signature: undefined,
              createSignature: data.create_signature,
              authoritySignature: data.authority_signature,
              writeCount: String(Array.isArray(data.write_signatures) ? data.write_signatures.length : 0),
              rentLamports: String(data.rent_lamports),
              programBytes: String(data.program_bytes),
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-prepare-upgrade-buffer.error"));
          }
          break;
        }

        case "squads-program-upgrade": {
          const m = walletAuth("squads-program-upgrade");
          if (!validateWalletAuth(m, formData, "private_key") || !formData.multisig || !formData.programId || !formData.bufferAddress) {
            toast.error(t("features.squads-program-upgrade.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            program_id: formData.programId,
            buffer_address: formData.bufferAddress,
            spill_address: formData.spillAddress,
            memo: formData.memo,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const actor = walletActor(m === "keystore" ? selectedSavedWallet() : undefined);
          const response = await apiFetch("squads/proposal/program-upgrade", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-program-upgrade.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceMultisig(formData.multisig, data.vault);
            saveWorkspaceProgram(formData.programId);
            saveWorkspaceProposal(
              data.proposal,
              formData.multisig,
              data.transaction_index,
              "program-upgrade",
              "active",
              undefined,
              actor,
            );
            setFormData((prev) => ({
              ...prev,
              vault: data.vault,
              transaction: data.transaction,
              proposal: data.proposal,
              transactionIndex: String(data.transaction_index),
              signature: data.signature,
              transactionKind: "program-upgrade",
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-program-upgrade.error"));
          }
          break;
        }

        case "squads-set-authority": {
          const m = walletAuth("squads-set-authority");
          if (!validateWalletAuth(m, formData, "private_key") || !formData.multisig || !formData.programId) {
            toast.error(t("features.squads-set-authority.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            program_id: formData.programId,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const actor = walletActor(m === "keystore" ? selectedSavedWallet() : undefined);
          const response = await apiFetch("squads/program/set-authority", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-set-authority.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceMultisig(formData.multisig, data.new_authority, undefined, undefined, actor);
            saveWorkspaceProgram(data.program_id);
            setFormData((prev) => ({
              ...prev,
              vault: data.new_authority,
              programdataAddress: data.programdata_address,
              signature: data.signature,
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-set-authority.error"));
          }
          break;
        }

        case "squads-approve":
        case "squads-reject": {
          const m = walletAuth(formId);
          if (!validateWalletAuth(m, formData, "private_key") || !formData.multisig || !formData.proposal) {
            toast.error(t("features.squads-vote.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            proposal: formData.proposal,
            memo: formData.memo,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const response = await apiFetch(formId === "squads-approve" ? "squads/proposal/approve" : "squads/proposal/reject", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(formId === "squads-approve" ? t("features.squads-approve.success") : t("features.squads-reject.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceProposal(
              formData.proposal,
              formData.multisig,
              formData.transactionIndex,
              undefined,
              formId === "squads-approve" ? "voted" : "rejected",
              undefined,
            );
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-vote.error"));
          }
          break;
        }

        case "squads-execute": {
          const m = walletAuth("squads-execute");
          const transactionIndex = parseInt(String(formData.transactionIndex || ""), 10);
          if (!validateWalletAuth(m, formData, "private_key") || !formData.multisig || !formData.proposal || !Number.isInteger(transactionIndex)) {
            toast.error(t("features.squads-execute.fillAllFields"));
            setLoading(false);
            return;
          }
          const requestBody: ApiRequestBody = {
            multisig: formData.multisig,
            proposal: formData.proposal,
            transaction_index: transactionIndex,
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");
          const response = await apiFetch("squads/proposal/execute", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();
          if (response.ok) {
            toast.success(t("features.squads-execute.success"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            saveWorkspaceProposal(
              formData.proposal,
              formData.multisig,
              transactionIndex,
              undefined,
              "executed",
              undefined,
            );
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
              network: data.network,
            }));
          } else {
            toast.error(data.error || t("features.squads-execute.error"));
          }
          break;
        }

        case "setup-2fa": {
          const masterPassword = String(formData.master_password || "");
          if (!formData.hardware_fingerprint || !masterPassword) {
            toast.error(t("features.setup-2fa.fillAllFields"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("2fa/setup", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              hardware_fingerprint: formData.hardware_fingerprint,
              master_password: masterPassword,
              account: formData.account || "sol-safekey",
              issuer: formData.issuer || "Sol SafeKey",
            }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.setup-2fa.success"));
            setFormData((prev) => ({
              ...prev,
              totp_secret: data.totp_secret,
              qr_code_url: data.qr_code_url,
            }));
          } else {
            toast.error(data.error || t("features.setup-2fa.error"));
          }
          break;
        }

        case "create-tfa": {
          const m = walletAuth("create-tfa");
          const masterPassword = String(formData.master_password || "");
          if (
            !formData.totp_secret ||
            !formData.hardware_fingerprint ||
            !masterPassword ||
            formData.question_index === undefined ||
            !formData.security_answer
          ) {
            toast.error(t("features.create-tfa.fillAllFields"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.create-tfa.fillAllFields"));
            setLoading(false);
            return;
          }
          const questionIndex =
            typeof formData.question_index === "number"
              ? formData.question_index
              : parseInt(String(formData.question_index), 10);
          if (!Number.isInteger(questionIndex) || questionIndex < 0 || questionIndex > 7) {
            toast.error(t("features.create-tfa.questionIndexError"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            totp_secret: formData.totp_secret,
            hardware_fingerprint: formData.hardware_fingerprint,
            master_password: masterPassword,
            question_index: questionIndex,
            security_answer: formData.security_answer,
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("2fa/create-tfa", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.create-tfa.success"));
            setFormData((prev) => ({
              ...prev,
              encrypted_wallet: data.encrypted_wallet,
              publicKey: data.public_key,
            }));
          } else {
            toast.error(data.error || t("features.create-tfa.error"));
          }
          break;
        }

        case "unlock-tfa": {
          const masterPassword = String(formData.master_password || "");
          if (!formData.encrypted_wallet || !formData.hardware_fingerprint || !masterPassword ||
              !formData.security_answer || !formData.totp_code) {
            toast.error(t("features.unlock-tfa.fillAllFields"));
            setLoading(false);
            return;
          }

          const response = await apiFetch("2fa/unlock-tfa", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              encrypted_wallet: formData.encrypted_wallet,
              hardware_fingerprint: formData.hardware_fingerprint,
              master_password: masterPassword,
              security_answer: formData.security_answer,
              totp_code: formData.totp_code,
            }),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.unlock-tfa.success"));
            setFormData((prev) => {
              const next: FormState = {
                ...prev,
                publicKey: data.public_key,
                unlocked: data.unlocked ? "true" : undefined,
              };
              delete next.private_key;
              delete next.secretKey;
              return next;
            });
          } else {
            toast.error(data.error || t("features.unlock-tfa.error"));
          }
          break;
        }

        case "close-wsol-ata": {
          const m = walletAuth("close-wsol-ata");
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(
              m === "keystore"
                ? t("features.close-wsol-ata.uploadKeystore")
                : m === "encrypted"
                  ? t("features.decrypt.fillAllFields")
                  : t("features.close-wsol-ata.enterPrivateKey"),
            );
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("wsol/close-ata", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(t("features.close-wsol-ata.success", { signature: data.signature }));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet);
            setFormData((prev) => ({
              ...prev,
              signature: data.signature,
            }));
          } else {
            toast.error(data.error || t("features.close-wsol-ata.error"));
          }
          break;
        }

        case "pumpfun-sell": {
          const m = walletAuth("pumpfun-sell");
          const amount = parsePositiveDecimal(formData.amount);
          const sellPercent = parseSellPercentBps(formData.sell_percent);
          if (!formData.mint || (amount === null && sellPercent === undefined)) {
            toast.error(t("features.pumpfun-sell.fillAllFields"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.pumpfun-sell.fillAllFields"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            mint: formData.mint,
            slippage: slippagePercentToBasisPoints(formData.slippage),
            network: submitNetwork(),
          };
          if (sellPercent !== undefined) requestBody.sell_percent = sellPercent;
          else if (amount !== null) requestBody.amount = amount;
          else {
            toast.error(t("features.pumpfun-sell.fillAllFields"));
            setLoading(false);
            return;
          }
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("pumpfun/sell", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(pumpSellSuccessMessage(data, "pumpfun-sell"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet, {
              tokenAdjustment: pumpSellTokenAdjustment(formData, data),
              refreshDelaysMs: POST_SELL_ASSET_REFRESH_DELAYS_MS,
              formSnapshot: formData,
            });
            setFormData((prev) => ({
              ...prev,
              status: data.status,
              signature: data.signature,
              dex: data.dex,
              market: data.market,
            }));
          } else {
            toast.error(data.error || t("features.pumpfun-sell.error"));
          }
          break;
        }

        case "pumpswap-sell": {
          const m = walletAuth("pumpswap-sell");
          const amount = parsePositiveDecimal(formData.amount);
          const sellPercent = parseSellPercentBps(formData.sell_percent);
          if (!formData.mint || (amount === null && sellPercent === undefined)) {
            toast.error(t("features.pumpswap-sell.fillAllFields"));
            setLoading(false);
            return;
          }
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(t("features.pumpswap-sell.fillAllFields"));
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            mint: formData.mint,
            slippage: slippagePercentToBasisPoints(formData.slippage),
            network: submitNetwork(),
          };
          if (sellPercent !== undefined) requestBody.sell_percent = sellPercent;
          else if (amount !== null) requestBody.amount = amount;
          else {
            toast.error(t("features.pumpswap-sell.fillAllFields"));
            setLoading(false);
            return;
          }
          applyWalletAuth(requestBody, m, formData, "private_key");

          const response = await apiFetch("pumpswap/sell", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(pumpSellSuccessMessage(data, "pumpswap-sell"));
            refreshWalletAfterMutation(savedWalletFromForm(formData) ?? effectiveWallet, {
              tokenAdjustment: pumpSellTokenAdjustment(formData, data),
              refreshDelaysMs: POST_SELL_ASSET_REFRESH_DELAYS_MS,
              formSnapshot: formData,
            });
            setFormData((prev) => ({
              ...prev,
              status: data.status,
              signature: data.signature,
              dex: data.dex,
              market: data.market,
            }));
          } else {
            toast.error(data.error || t("features.pumpswap-sell.error"));
          }
          break;
        }

        case "pumpfun-cashback":
        case "pumpswap-cashback": {
          const m = walletAuth(formId);
          if (!validateWalletAuth(m, formData, "private_key")) {
            toast.error(
              m === "keystore"
                ? t("features.pumpfun-cashback.uploadKeystore")
                : m === "encrypted"
                  ? t("features.decrypt.fillAllFields")
                  : t("features.pumpfun-cashback.enterPrivateKey"),
            );
            setLoading(false);
            return;
          }

          const requestBody: ApiRequestBody = {
            network: submitNetwork(),
          };
          applyWalletAuth(requestBody, m, formData, "private_key");

          const apiUrlPath = formId === "pumpfun-cashback" ? "/pumpfun/cashback" : "/pumpswap/cashback";
          const response = await apiFetch(apiUrlPath.replace(/^\//, ""), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(requestBody),
          });
          const data = await response.json();

          if (response.ok) {
            toast.success(
              formId === "pumpfun-cashback"
                ? t("features.pumpfun-cashback.success")
                : t("features.pumpswap-cashback.success"),
            );
            setFormData((prev) => ({
              ...prev,
              message: data.message,
              status: data.status,
              signature: data.signature,
              cashbackAmount: data.ui_amount_string,
              cashbackAsset: data.asset,
            }));
            const submittedWallet = savedWalletFromForm(formData) ?? effectiveWallet;
            refreshWalletAfterMutation(submittedWallet);
            await loadCashbackInfo(formId === "pumpfun-cashback" ? "pumpfun" : "pumpswap", submittedWallet);
          } else {
            toast.error(
              data.error ||
                (formId === "pumpfun-cashback"
                  ? t("features.pumpfun-cashback.error")
                  : t("features.pumpswap-cashback.error")),
            );
          }
          break;
        }

        default:
          toast.error(t("errors.unknownFeature"));
      }
    } catch (err) {
      const message =
        err instanceof Error ? err.message : typeof err === "string" ? err : t("errors.unknownError");
      toast.error(
        t("errors.requestFailedWithHint", {
          message,
          port: String(DEFAULT_API_PORT),
        }),
      );
    } finally {
      setLoading(false);
    }
  };

  const renderForm = (formId: string) => {
    const renderCopyRow = (id: string, label: string, value: unknown) => {
      if (!value) return null;
      return (
        <div>
          <label className="block text-sm font-medium mb-2">{label}</label>
          <div className="flex gap-2">
            <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
              {String(value)}
            </code>
            <button
              type="button"
              onClick={() => copyToClipboard(String(value), id)}
              className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
            >
              {copied === id ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
            </button>
          </div>
        </div>
      );
    };

    const openAction = (form: string, preset: FormState = {}) => () => {
      handleOpenForm(form, {
        network: requestNetwork(),
        ...preset,
      });
    };

    const renderActionGrid = (
      actions: Array<{
        id: string;
        title: string;
        icon: React.ReactNode;
        preset?: FormState;
      }>,
    ) => (
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {actions.map((action) => (
          <button
            key={action.id}
            type="button"
            onClick={openAction(action.id, action.preset)}
            className="flex items-center gap-3 p-4 bg-white/5 border border-white/10 rounded-lg text-left hover:bg-white/10 transition-colors"
          >
            <span className="shrink-0 text-gray-300">{action.icon}</span>
            <span className="min-w-0 text-sm font-medium">{action.title}</span>
          </button>
        ))}
      </div>
    );

    const renderWalletAccountManager = () => (
      <div className="space-y-3">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h3 className="text-sm font-semibold text-gray-200">{t("features.settings.walletTitle")}</h3>
            <p className="mt-1 text-xs text-gray-500">{t("features.settings.walletHint")}</p>
          </div>
          <button
            type="button"
            onClick={() => void loadWallets()}
            className="shrink-0 rounded-lg bg-white/10 px-3 py-2 text-xs hover:bg-white/20 transition-colors"
          >
            {walletsLoading ? t("common.loading") : t("formUi.refreshWallets")}
          </button>
        </div>
        {wallets.length === 0 ? (
          <div className="rounded-xl border border-white/10 bg-white/5 p-4 text-sm text-gray-400">
            {t("features.wallet-list.empty")}
          </div>
        ) : (
          <div className="space-y-3">
            {wallets.map((wallet) => {
              const isCurrent = wallet.id === effectiveWalletId;
              const walletSolBalance =
                isCurrent &&
                walletAssets?.address === wallet.public_key &&
                walletAssets.network === effectiveNetwork &&
                walletAssets.solBalance !== "--"
                  ? walletAssets.solBalance
                  : "--";
              const copyId = `settings-wallet:${wallet.id}`;
              return (
                <div
                  role="button"
                  tabIndex={0}
                  key={wallet.id}
                  onClick={() => {
                    if (!isCurrent) {
                      setCurrentWallet(wallet.id);
                    }
                  }}
                  onKeyDown={(event) => {
                    if (isCurrent) return;
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      setCurrentWallet(wallet.id);
                    }
                  }}
                  className={`block w-full rounded-xl border p-3 text-left transition-colors sm:p-4 ${
                    isCurrent ? "border-violet-300/30 bg-violet-400/10" : "border-white/10 bg-white/[0.04] hover:bg-white/[0.07]"
                  }`}
                >
                  <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center">
                    <div className="flex min-w-0 gap-3">
                      <span className="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-white/10 text-sm font-semibold text-white sm:h-11 sm:w-11">
                        {walletAvatarText(wallet)}
                      </span>
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="max-w-full truncate text-base font-semibold text-white">{wallet.name}</p>
                          {isCurrent && (
                            <span className="rounded-full bg-emerald-400/15 px-2 py-0.5 text-xs text-emerald-200">
                              {t("features.walletContext.current")}
                            </span>
                          )}
                        </div>
                        <div className="mt-2 flex min-w-0 items-center gap-2">
                          <button
                            type="button"
                            onClick={(event) => {
                              event.stopPropagation();
                              copyToClipboard(wallet.public_key, copyId);
                            }}
                            className="min-w-0 truncate text-left font-mono text-xs text-gray-400 hover:text-white sm:rounded-lg sm:border sm:border-white/10 sm:bg-black/20 sm:px-2.5 sm:py-1.5 sm:text-gray-300 sm:hover:bg-white/10"
                            title={wallet.public_key}
                          >
                            <span className="sm:hidden">{shortAddress(wallet.public_key)}</span>
                            <span className="hidden sm:inline">{wallet.public_key}</span>
                          </button>
                          <button
                            type="button"
                            onClick={(event) => {
                              event.stopPropagation();
                              copyToClipboard(wallet.public_key, copyId);
                            }}
                            className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-white/5 text-gray-300 hover:bg-white/10 hover:text-white sm:h-8 sm:w-8 sm:border sm:border-white/10"
                            title={t("features.walletContext.copyAddress")}
                            aria-label={t("features.walletContext.copyAddress")}
                          >
                            {copied === copyId ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
                          </button>
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center justify-end gap-2 lg:justify-end">
                      <div className="hidden min-w-28 rounded-lg border border-white/10 bg-black/20 px-3 py-2 text-right sm:block">
                        <p className="text-[11px] uppercase tracking-normal text-gray-500">{t("features.wallet-list.solBalance")}</p>
                        <p className="mt-0.5 truncate text-sm font-semibold text-white">{walletSolBalance} SOL</p>
                      </div>
                      <div className="relative" data-wallet-actions-menu>
                        <button
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            setWalletActionsMenuOpen((open) => (open === wallet.id ? null : wallet.id));
                          }}
                          className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5 text-gray-300 hover:bg-white/10 hover:text-white"
                          title={t("features.wallet-list.actions")}
                          aria-label={t("features.wallet-list.actions")}
                        >
                          <Menu className="h-4 w-4" />
                        </button>
                        {walletActionsMenuOpen === wallet.id && (
                          <div className="absolute right-0 z-50 mt-2 w-48 overflow-hidden rounded-xl border border-white/10 bg-zinc-950 p-1 shadow-2xl">
                            <button
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                setWalletActionsMenuOpen(null);
                                setEditingWalletId(wallet.id);
                                setFormData((prev) => ({
                                  ...prev,
                                  [`walletName:${wallet.id}`]: wallet.name,
                                }));
                              }}
                              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs text-gray-200 hover:bg-white/10"
                            >
                              <Pencil className="h-3.5 w-3.5" />
                              {t("formUi.editWalletMetadata")}
                            </button>
                            <button
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                setWalletActionsMenuOpen(null);
                                requestExportKeystore(wallet);
                              }}
                              disabled={loading}
                              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs text-gray-200 hover:bg-white/10 disabled:opacity-50"
                            >
                              <Download className="h-3.5 w-3.5" />
                              {t("features.settings.exportKeystore")}
                            </button>
                            <button
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                setWalletActionsMenuOpen(null);
                                requestExportPrivateKey(wallet);
                              }}
                              disabled={loading}
                              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs text-amber-100 hover:bg-amber-500/10 disabled:opacity-50"
                            >
                              <Key className="h-3.5 w-3.5" />
                              {t("features.settings.exportPrivateKey")}
                            </button>
                            <button
                              type="button"
                              onClick={(event) => {
                                event.stopPropagation();
                                setWalletActionsMenuOpen(null);
                                void handleDeleteWallet(wallet);
                              }}
                              disabled={loading}
                              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs text-red-200 hover:bg-red-500/10 disabled:opacity-50"
                            >
                              <Trash2 className="h-3.5 w-3.5" />
                              {t("features.settings.deleteWallet")}
                            </button>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                  {editingWalletId === wallet.id && (
                    <div
                      className="mt-4 grid grid-cols-1 gap-2 md:grid-cols-[1fr_auto_auto]"
                      onClick={(event) => event.stopPropagation()}
                    >
                      <input
                        type="text"
                        value={formData[`walletName:${wallet.id}`] ?? wallet.name}
                        onChange={(e) => handleFormChange(`walletName:${wallet.id}`, e.target.value)}
                        className="min-w-0 rounded-lg border border-white/10 bg-black/30 px-3 py-2 text-sm text-white focus:outline-none focus:ring-2 focus:ring-white/20"
                        placeholder={t("formUi.walletNamePlaceholder")}
                      />
                      <button
                        type="button"
                        onClick={() => void handleSaveWalletMetadata(wallet)}
                        disabled={loading}
                        className="inline-flex items-center justify-center gap-1 rounded-lg bg-white/10 px-3 py-2 text-xs hover:bg-white/20 disabled:opacity-50"
                      >
                        <Save className="h-3.5 w-3.5" />
                        {t("formUi.saveWalletMetadata")}
                      </button>
                      <button
                        type="button"
                        onClick={() => handleCancelWalletMetadata(wallet)}
                        disabled={loading}
                        className="rounded-lg bg-white/5 px-3 py-2 text-xs hover:bg-white/10 disabled:opacity-50"
                      >
                        {t("common.cancel")}
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    );

    const renderWalletListPanel = () => {
      const assets =
        effectiveWallet &&
        walletAssets?.address === effectiveWallet.public_key &&
        walletAssets.network === effectiveNetwork
          ? walletAssets
          : null;
      const solBalance = assets?.solBalance && assets.solBalance !== "--" ? assets.solBalance : "--";
      const tokenAssets = assets?.tokens ?? [];
      const visibleTokenAssets = tokenAssets.slice(0, visibleTokenCount);
      const hiddenTokenCount = Math.max(0, tokenAssets.length - visibleTokenAssets.length);
      const tokenSymbols = new Map(tokenAssets.map((token) => [token.mint, tokenDisplaySymbol(token)]));
      const assetsLoading = Boolean(assets?.loading || assets?.refreshing);
      const activeWalletAddress = effectiveWallet?.public_key ?? "";
      const currentTransactions =
        activeWalletAddress !== "" &&
        walletTransactions?.address === activeWalletAddress &&
        walletTransactions.network === effectiveNetwork
          ? walletTransactions
          : null;
      const transactionsLoading = Boolean(currentTransactions?.loading);

      if (!effectiveWallet) {
        return (
          <div className="space-y-6">
            <div className="rounded-2xl border border-white/10 bg-black/40 p-6 text-center">
              <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-full bg-white/10">
                <Wallet className="h-8 w-8 text-gray-300" />
              </div>
              <h3 className="mt-5 text-xl font-semibold">{t("features.wallet-list.emptyTitle")}</h3>
              <p className="mx-auto mt-2 max-w-md text-sm text-gray-400">{t("features.wallet-list.empty")}</p>
              <div className="mt-6 grid grid-cols-1 gap-3 sm:grid-cols-2">
                <button
                  type="button"
                  onClick={() => handleOpenForm("create-keystore", {}, "wallet-list")}
                  className="rounded-xl bg-white px-4 py-3 text-sm font-semibold text-black hover:bg-gray-200"
                >
                  {t("features.walletContext.createWallet")}
                </button>
                <button
                  type="button"
                  onClick={() => handleOpenForm("import-keystore", {}, "wallet-list")}
                  className="rounded-xl border border-white/10 bg-white/10 px-4 py-3 text-sm font-semibold hover:bg-white/20"
                >
                  {t("features.walletContext.importWallet")}
                </button>
              </div>
            </div>
          </div>
        );
      }

      const walletActions = [
        {
          id: "send",
          title: t("features.wallet-list.send"),
          icon: <Send className="h-5 w-5" />,
          onClick: () => handleOpenForm("transfer-sol", { wallet_id: effectiveWallet.id, network: effectiveNetwork }, "wallet-list"),
        },
        {
          id: "receive",
          title: t("features.wallet-list.receive"),
          icon: <Download className="h-5 w-5" />,
          onClick: () => copyToClipboard(effectiveWallet.public_key, "wallet-receive-address"),
        },
        {
          id: "trade",
          title: t("features.wallet-list.trade"),
          icon: <ArrowRightLeft className="h-5 w-5" />,
          onClick: () => handleOpenForm("pump-workbench", { wallet_id: effectiveWallet.id, network: effectiveNetwork }, "wallet-list"),
        },
        {
          id: "token",
          title: t("features.wallet-list.tokenSend"),
          icon: <Coins className="h-5 w-5" />,
          onClick: () => handleOpenForm("transfer-token", { wallet_id: effectiveWallet.id, network: effectiveNetwork }, "wallet-list"),
        },
        {
          id: "history",
          title: t("features.wallet-list.transactions"),
          icon: <ArrowRightLeft className="h-5 w-5" />,
          onClick: () => {
            setWalletOverviewTab("transactions");
            document.getElementById("wallet-overview-tabs")?.scrollIntoView({ behavior: "smooth", block: "start" });
          },
        },
      ];

      return (
        <div className="space-y-4">
          <section className="overflow-hidden border-white/10 bg-transparent lg:rounded-xl lg:border lg:bg-black/50">
            <div className="rounded-2xl border border-white/10 bg-gradient-to-br from-blue-500 to-violet-700 p-5 shadow-xl shadow-black/20 lg:rounded-none lg:border-0 lg:bg-none lg:p-4 lg:shadow-none">
              <div className="flex min-w-0 flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                <div className="flex min-w-0 items-center gap-3">
                  <span className="hidden h-11 w-11 shrink-0 items-center justify-center rounded-full bg-violet-300 text-sm font-bold text-black lg:flex">
                    {walletAvatarText(effectiveWallet)}
                  </span>
                  <div className="min-w-0">
                    <div className="flex flex-wrap items-center gap-2">
                      <h3 className="max-w-full truncate text-lg font-semibold lg:text-lg">{effectiveWallet.name}</h3>
                      <span className="rounded-full border border-white/15 bg-white/15 px-2 py-0.5 text-xs text-white/80 lg:border-white/10 lg:bg-white/5 lg:text-gray-300">
                        <span className="lg:hidden">{effectiveNetworkLabel}</span>
                        <span className="hidden lg:inline">{effectiveRpcLabel}</span>
                      </span>
                    </div>
                    <button
                      type="button"
                      onClick={() => copyToClipboard(effectiveWallet.public_key, "wallet-home-address")}
                      className="mt-1 block max-w-full truncate text-left font-mono text-xs text-white/75 hover:text-white lg:text-gray-400"
                    >
                      <span className="lg:hidden">{shortAddress(effectiveWallet.public_key)}</span>
                      <span className="hidden lg:inline">{effectiveWallet.public_key}</span>
                    </button>
                  </div>
                </div>
                <div className="min-w-0">
                  <div className="flex items-center gap-2">
                    <p className="text-xs text-white/70 lg:text-gray-400">{t("features.wallet-list.solBalance")}</p>
                    <button
                      type="button"
                      onClick={() => refreshCurrentWalletAssets(effectiveWallet)}
                      className="inline-flex h-6 w-6 items-center justify-center rounded-md bg-white/20 text-white hover:bg-white/30 lg:bg-white/10 lg:text-gray-300 lg:hover:bg-white/20"
                      title={t("features.wallet-list.refreshAssets")}
                      aria-label={t("features.wallet-list.refreshAssets")}
                    >
                      <RefreshCw className={`h-3.5 w-3.5 ${assetsLoading ? "animate-spin" : ""}`} />
                    </button>
                  </div>
                  <div className="mt-1 flex min-w-0 flex-wrap items-baseline gap-x-2 gap-y-1">
                    <p className="max-w-full truncate text-4xl font-semibold tracking-normal lg:text-3xl">{solBalance}</p>
                    <p className="text-sm text-white/75 lg:text-gray-400">SOL</p>
                  </div>
                  {assets?.error && (
                    <p className="mt-2 text-xs text-yellow-100 lg:text-yellow-200">
                      {assets.error || t("features.wallet-list.assetLoadFailed")}
                    </p>
                  )}
                </div>
              </div>
            </div>
            <div className="mt-3 grid grid-cols-4 gap-3 lg:mt-0 lg:grid-cols-5 lg:gap-0 lg:border-t lg:border-white/10 lg:bg-white/[0.03]">
              {walletActions.map((action, index) => (
                <button
                  key={action.id}
                  type="button"
                  onClick={action.onClick}
                  className={`${index === 4 ? "hidden lg:flex" : "flex"} h-20 min-w-0 flex-col items-center justify-center gap-2 rounded-xl bg-white/10 px-2 text-xs font-semibold text-gray-200 hover:bg-white/15 lg:h-16 lg:rounded-none lg:border-r lg:border-white/10 lg:bg-transparent lg:last:border-r-0 lg:hover:bg-white/10 sm:h-14 sm:flex-row sm:gap-2 sm:text-sm`}
                >
                  <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-black/25 text-violet-100 lg:h-8 lg:w-8 lg:bg-black/30 sm:h-7 sm:w-7">
                    {action.icon}
                  </span>
                  <span className="max-w-full truncate">{action.title}</span>
                </button>
              ))}
            </div>
          </section>

          <section id="wallet-overview-tabs" className="space-y-4 scroll-mt-6">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <div className="inline-flex rounded-lg border border-white/10 bg-white/5 p-1">
                {[
                  { id: "assets" as const, label: t("features.wallet-list.assets") },
                  { id: "transactions" as const, label: t("features.wallet-list.transactions") },
                ].map((tab) => (
                  <button
                    key={tab.id}
                    type="button"
                    onClick={() => setWalletOverviewTab(tab.id)}
                    className={`rounded-md px-4 py-2 text-sm font-semibold transition-colors ${
                      walletOverviewTab === tab.id
                        ? "bg-white text-black"
                        : "text-gray-300 hover:bg-white/10 hover:text-white"
                    }`}
                  >
                    {tab.label}
                  </button>
                ))}
              </div>
              {walletOverviewTab === "assets" ? (
                <button
                  type="button"
                  onClick={() => void loadWalletAssets(effectiveWallet, { refresh: true, force: true })}
                  className="inline-flex w-fit items-center gap-2 rounded-lg bg-white/10 px-3 py-2 text-xs hover:bg-white/20"
                >
                  <RefreshCw className={`h-3.5 w-3.5 ${assetsLoading ? "animate-spin" : ""}`} />
                  {t("features.wallet-list.refreshAssets")}
                </button>
              ) : (
                <button
                  type="button"
                  onClick={() => void loadWalletTransactions(effectiveWallet, "replace", { force: true })}
                  className="inline-flex w-fit items-center gap-2 rounded-lg bg-white/10 px-3 py-2 text-xs hover:bg-white/20"
                >
                  <RefreshCw className={`h-3.5 w-3.5 ${transactionsLoading ? "animate-spin" : ""}`} />
                  {t("features.wallet-list.refreshTransactions")}
                </button>
              )}
            </div>

            {walletOverviewTab === "assets" ? (
              <div className="space-y-2 lg:space-y-3">
                <div className="space-y-3">
                  <div className="flex items-center justify-between gap-4 border-b border-white/10 bg-transparent px-1 py-3 lg:rounded-xl lg:border lg:bg-white/10 lg:p-4">
                    <div className="flex min-w-0 items-center gap-3">
                      <span className="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-full bg-black p-2 ring-1 ring-white/10">
                        {/* eslint-disable-next-line @next/next/no-img-element */}
                        <img
                          src={SOLANA_TOKEN_LOGO_URI}
                          alt=""
                          className="h-full w-full object-contain"
                          loading="eager"
                        />
                      </span>
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="text-base font-semibold">SOL</p>
                          <span className="rounded-md bg-white/10 px-2 py-0.5 text-xs text-gray-300">
                            {t("features.wallet-list.solana")}
                          </span>
                        </div>
                        <p className="mt-1 hidden truncate text-xs text-gray-400 lg:block">{t("features.wallet-list.nativeToken")}</p>
                      </div>
                    </div>
                    <div className="text-right">
                      <p className="text-lg font-semibold">{solBalance}</p>
                      <p className="hidden text-xs text-gray-500 lg:block">{effectiveRpcLabel}</p>
                    </div>
                  </div>
                  {visibleTokenAssets.map((token) => {
                    const colors = tokenColorPair(token.mint);
                    const tokenSymbol = tokenDisplaySymbol(token);
                    const tokenName = tokenMetadataName(token);
                    const logoUri = tokenLogoUri(token);
                    const usesSolanaLogo = logoUri === SOLANA_TOKEN_LOGO_URI;
                    return (
                      <div
                        key={token.account}
                        className="flex gap-3 border-b border-white/10 bg-transparent px-1 py-3 transition-colors hover:bg-white/5 focus-within:bg-white/5 lg:flex-row lg:items-center lg:justify-between lg:gap-4 lg:rounded-xl lg:border lg:bg-white/10 lg:p-4 lg:hover:bg-white/15 lg:focus-within:bg-white/15"
                      >
                        <button
                          type="button"
                          onClick={() => openTokenActions(token)}
                          className="flex min-w-0 flex-1 items-center gap-3 text-left lg:gap-4"
                        >
                          <span
                            className={`relative flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-full text-sm font-bold text-white ring-1 ring-white/15 lg:h-14 lg:w-14 ${usesSolanaLogo ? "bg-black p-2" : ""}`}
                            style={usesSolanaLogo ? undefined : { background: `linear-gradient(135deg, ${colors.from}, ${colors.to})` }}
                          >
                            {logoUri ? (
                              // eslint-disable-next-line @next/next/no-img-element
                              <img
                                src={logoUri}
                                alt=""
                                className={`h-full w-full ${usesSolanaLogo ? "object-contain" : "object-cover"}`}
                                loading="lazy"
                                referrerPolicy="no-referrer"
                              />
                            ) : (
                              <span>{tokenAvatarText(token)}</span>
                            )}
                          </span>
                          <div className="min-w-0">
                            <div className="flex flex-wrap items-center gap-2">
                              <p className="text-base font-semibold">{tokenSymbol}</p>
                              {tokenName && (
                                <span
                                  className="max-w-[18rem] truncate rounded-md bg-white/10 px-2 py-0.5 text-xs text-gray-300"
                                  title={tokenName}
                                >
                                  {tokenName}
                                </span>
                              )}
                            </div>
                            <div className="mt-1 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1 text-xs text-gray-400">
                              <span className="hidden max-w-full truncate font-mono lg:block" title={token.mint}>
                                {token.mint}
                              </span>
                            </div>
                          </div>
                        </button>
                        <div className="flex shrink-0 items-center justify-end gap-3">
                          <div className="text-right">
                            <p className="text-lg font-semibold">{token.ui_amount_string}</p>
                          </div>
                          <button
                            type="button"
                            onClick={() => copyToClipboard(token.mint, `token-mint:${token.account}`)}
                            className="hidden h-9 w-9 items-center justify-center rounded-lg bg-white/10 text-gray-300 hover:bg-white/20 hover:text-white lg:inline-flex"
                            title={t("features.token-actions.copyMint")}
                            aria-label={t("features.token-actions.copyMint")}
                          >
                            {copied === `token-mint:${token.account}` ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                          </button>
                          <button
                            type="button"
                            onClick={() => openTokenActions(token)}
                            className="hidden h-9 items-center gap-1 rounded-lg bg-white/10 px-3 text-xs font-semibold text-gray-200 hover:bg-white/20 lg:inline-flex"
                          >
                            {t("features.wallet-list.actions")}
                            <ChevronRight className="h-4 w-4" />
                          </button>
                        </div>
                      </div>
                    );
                  })}
                  {hiddenTokenCount > 0 && (
                    <button
                      type="button"
                      onClick={() => setVisibleTokenCount((count) => count + TOKEN_ASSET_PAGE_SIZE)}
                      className="flex w-full items-center justify-center rounded-xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-semibold text-gray-200 hover:bg-white/10"
                    >
                      {t("features.wallet-list.loadMoreTokens", {
                        count: Math.min(TOKEN_ASSET_PAGE_SIZE, hiddenTokenCount),
                      })}
                    </button>
                  )}
                  {!assetsLoading && tokenAssets.length === 0 && (
                    <div className="rounded-lg border border-dashed border-white/10 bg-white/[0.03] p-6 text-center text-sm text-gray-400">
                      {t("features.wallet-list.noTokens")}
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                {renderTransactionList(
                  currentTransactions,
                  () => void loadWalletTransactions(effectiveWallet, "append"),
                  t("features.wallet-list.noTransactions"),
                  t("features.wallet-list.transactionsLoadFailed"),
                  tokenSymbols,
                )}
              </div>
            )}
          </section>
        </div>
      );
    };

    const renderProgramWorkspacePanel = (showWorkspaceHeader = false) => (
      <div className="space-y-4">
        {showWorkspaceHeader && renderSquadsWorkspaceHeader()}
        <div className="space-y-3">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-sm font-semibold text-gray-300">{t("features.workspace.savedPrograms")}</h3>
            <details data-close-on-outside className="relative">
              <summary className="cursor-pointer list-none px-3 py-2 bg-white/10 rounded-lg text-sm hover:bg-white/20">
                {t("features.workspace.actions")}
              </summary>
              <div className="absolute right-0 z-20 mt-2 w-64 overflow-hidden rounded-lg border border-white/10 bg-zinc-950 shadow-xl">
                {[
                  { id: "program-info", label: t("features.program-info.title"), onClick: () => handleOpenForm("program-info") },
                  { id: "program-deploy", label: t("features.program-deploy.title"), onClick: () => handleOpenForm("program-deploy") },
                ].map((action) => (
                  <button
                    key={action.id}
                    type="button"
                    onClick={() => {
                      action.onClick();
                      closeDropdownMenus();
                    }}
                    className="block w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-white/10"
                  >
                    {action.label}
                  </button>
                ))}
              </div>
            </details>
          </div>
          {currentPrograms.length === 0 ? (
            <p className="text-xs text-gray-500">{t("features.workspace.emptyPrograms")}</p>
          ) : currentPrograms.map((item) => (
            <div key={`${item.network}-${item.address}`} className="p-3 bg-white/5 border border-white/10 rounded-lg space-y-3">
              <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-3">
                <div className="min-w-0">
                  <p className="text-sm font-medium">{workspaceLabel(item.label, item.address)}</p>
                  <code className="block text-xs text-gray-400 break-all">{item.address}</code>
                </div>
                <details data-close-on-outside className="relative shrink-0">
                  <summary className="cursor-pointer list-none px-3 py-1.5 bg-white/10 rounded text-xs hover:bg-white/20">
                    {t("features.workspace.actions")}
                  </summary>
                  <div className="absolute right-0 z-20 mt-2 w-56 overflow-hidden rounded-lg border border-white/10 bg-zinc-950 shadow-xl">
                    {[
                      { id: "info", label: t("features.program-info.queryButton"), onClick: () => handleOpenForm("program-info", { programId: item.address, network: item.network }) },
                      { id: "upgrade", label: t("features.squads-program-upgrade.createButton"), onClick: () => handleOpenSquadsForm("squads-program-upgrade", { programId: item.address, network: item.network }) },
                      { id: "authority", label: t("features.squads-set-authority.setButton"), onClick: () => handleOpenSquadsForm("squads-set-authority", { programId: item.address, network: item.network }) },
                      { id: "remove", label: t("features.workspace.remove"), onClick: () => removeWorkspaceItem("programs", item.address, item.network) },
                    ].map((action) => (
                      <button
                        key={action.id}
                        type="button"
                        onClick={() => {
                          action.onClick();
                          closeDropdownMenus();
                        }}
                        className="block w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-white/10"
                      >
                        {action.label}
                      </button>
                    ))}
                  </div>
                </details>
              </div>
            </div>
          ))}
        </div>
      </div>
    );

    const renderNonceCreateForm = () => {
      const nonceAuth = walletAuth("create-nonce");
      return (
        <div className="space-y-4">
          <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
            <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
            <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
              <button type="button"
                onClick={() => {
                  setAuthMethod({ ...authMethod, "create-nonce": "keystore" });
                  const newFormData = { ...formData };
                  delete newFormData.private_key;
                  delete newFormData.encrypted_key;
                  setFormData(newFormData);
                }}
                className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                  nonceAuth === "keystore"
                    ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                    : "bg-white/5 text-gray-400 hover:bg-white/10"
                }`}
              >
                {t("formUi.tabKeystore")}
              </button>
              <button type="button"
                onClick={() => {
                  setAuthMethod({ ...authMethod, "create-nonce": "encrypted" });
                  const newFormData = { ...formData };
                  delete newFormData.private_key;
                  delete newFormData.keystoreJson;
                  setFormData(newFormData);
                }}
                className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                  nonceAuth === "encrypted"
                    ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                    : "bg-white/5 text-gray-400 hover:bg-white/10"
                }`}
              >
                {t("formUi.tabEncrypted")}
              </button>
              <button type="button"
                onClick={() => {
                  setAuthMethod({ ...authMethod, "create-nonce": "private" });
                  const newFormData = { ...formData };
                  delete newFormData.keystoreJson;
                  delete newFormData.encrypted_key;
                  delete newFormData.password;
                  setFormData(newFormData);
                }}
                className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                  nonceAuth === "private"
                    ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                    : "bg-white/5 text-gray-400 hover:bg-white/10"
                }`}
              >
                {t("formUi.tabPrivateKey")}
              </button>
            </div>
          </div>

          {nonceAuth === "keystore" && (
            <div className={formData.wallet_id ? "hidden" : undefined}>
              <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
              <input
                key={`create-nonce-${nonceAuth}`}
                type="file"
                accept=".json,application/json"
                onChange={handleFileUpload}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
              />
              {formData.keystoreJson && (
                <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
              )}
            </div>
          )}

          {ALLOW_DIRECT_SECRET_INPUT && nonceAuth === "encrypted" && (
            <div>
              <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
              <textarea
                value={formData.encrypted_key || ""}
                onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                placeholder={t("formUi.placeholderEncryptedKey")}
              />
            </div>
          )}

          {ALLOW_DIRECT_SECRET_INPUT && nonceAuth === "private" && (
            <div>
              <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
              <input
                type="password"
                value={formData.private_key || ""}
                onChange={(e) => handleFormChange("private_key", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("formUi.placeholderPrivateKeyBase58")}
              />
              <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
            </div>
          )}

          <div>
            <label className="block text-sm font-medium mb-2">{t("features.create-nonce.count")}</label>
            <input
              type="text"
              inputMode="numeric"
              autoComplete="off"
              value={formData.count ?? 1}
              onChange={(e) => handleFormChange("count", e.target.value.replace(/[^\d]/g, ""))}
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
              placeholder={t("features.create-nonce.countPlaceholder")}
            />
            <p className="mt-1 text-xs text-gray-500">{t("features.create-nonce.countHint")}</p>
          </div>

          <button type="button"
            onClick={() => requestPasswordSubmit("create-nonce")}
            disabled={loading}
            className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
          >
            {loading ? t("features.create-nonce.creating") : t("features.create-nonce.createButton")}
          </button>
        </div>
      );
    };

    const renderNonceCreateResult = () => (
      <>
        {createdNonceAccounts.length > 0 && (
          <div className="space-y-3 rounded-lg bg-white/5 p-4">
            <label className="block text-sm font-medium">{t("features.create-nonce.createdAccounts")}</label>
            {createdNonceAccounts.map((item, index) => (
              <div key={`${item.nonce_account}-${index}`} className="rounded-lg bg-black/30 p-3">
                <div className="flex gap-2">
                  <code className="flex-1 text-xs break-all">{item.nonce_account}</code>
                  <button
                    type="button"
                    onClick={() => copyToClipboard(String(item.nonce_account ?? ""), `nonce-created:${index}`)}
                    className="rounded bg-white/10 px-2 py-1 transition-colors hover:bg-white/20"
                  >
                    {copied === `nonce-created:${index}` ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                  </button>
                </div>
                <p className="mt-2 break-all font-mono text-xs text-gray-500">{item.signature}</p>
              </div>
            ))}
          </div>
        )}
        {createdNonceAccounts.length === 0 && formData.signature && (
          <div className="space-y-3 rounded-lg bg-white/5 p-4">
            <div>
              <label className="mb-2 block text-sm font-medium">{t("features.create-nonce.nonceAccount")}</label>
              <div className="flex gap-2">
                <code className="flex-1 rounded bg-black/30 px-3 py-2 text-xs break-all">
                  {formData.nonceAccount}
                </code>
                <button
                  type="button"
                  onClick={() => copyToClipboard(formData.nonceAccount as string, "nonce-addr")}
                  className="rounded bg-white/10 px-3 py-2 transition-colors hover:bg-white/20"
                >
                  {copied === "nonce-addr" ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                </button>
              </div>
            </div>
            <div>
              <label className="mb-2 block text-sm font-medium">{t("formUi.txSignature")}</label>
              <div className="flex gap-2">
                <code className="flex-1 rounded bg-black/30 px-3 py-2 text-xs break-all">
                  {formData.signature}
                </code>
                <button
                  type="button"
                  onClick={() => copyToClipboard(formData.signature as string, "nonce-sig")}
                  className="rounded bg-white/10 px-3 py-2 transition-colors hover:bg-white/20"
                >
                  {copied === "nonce-sig" ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                </button>
              </div>
            </div>
          </div>
        )}
      </>
    );

    const renderNonceAccountList = () => {
      const currentNonceAccounts =
        effectiveWallet &&
        nonceAccounts?.owner === effectiveWallet.public_key &&
        nonceAccounts.network === effectiveNetwork
          ? nonceAccounts.nonceAccounts
          : [];
      const nonceLoading =
        effectiveWallet &&
        nonceAccounts?.owner === effectiveWallet.public_key &&
        nonceAccounts.network === effectiveNetwork &&
        nonceAccounts.loading;
      const nonceError =
        effectiveWallet &&
        nonceAccounts?.owner === effectiveWallet.public_key &&
        nonceAccounts.network === effectiveNetwork
          ? nonceAccounts.error
          : undefined;

      return (
        <div className="space-y-4">
          <div className="flex flex-wrap justify-end gap-2">
            <button
              type="button"
              onClick={() => void loadNonceAccounts(effectiveWallet, { force: true })}
              className="inline-flex items-center gap-2 rounded-lg bg-white/10 px-3 py-2 text-sm hover:bg-white/20"
            >
              <RefreshCw className={`h-4 w-4 ${nonceLoading ? "animate-spin" : ""}`} />
              {t("features.create-nonce.refreshAccounts")}
            </button>
            <button
              type="button"
              onClick={openNonceCreateDialog}
              disabled={!effectiveWallet}
              className="inline-flex items-center gap-2 rounded-lg bg-white px-3 py-2 text-sm font-semibold text-black hover:bg-gray-200 disabled:opacity-50"
            >
              <Plus className="h-4 w-4" />
              {t("features.create-nonce.createButton")}
            </button>
          </div>

          {!effectiveWallet && (
            <div className="rounded-2xl border border-dashed border-white/10 bg-white/[0.03] p-6 text-center text-sm text-gray-400">
              {t("features.walletContext.importHint")}
            </div>
          )}
          {nonceError && (
            <p className="text-sm text-yellow-200">{nonceError}</p>
          )}
          {currentNonceAccounts.length > 0 ? (
            <div className="space-y-3">
              {currentNonceAccounts.map((item) => (
                <div key={item.id} className="rounded-2xl border border-white/10 bg-white/5 p-4">
                  <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
                    <div className="min-w-0">
                      <p className="text-sm font-semibold text-gray-200">{t("features.create-nonce.nonceAccount")}</p>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(item.nonce_account, `nonce-saved:${item.id}`)}
                        className="mt-2 block max-w-full truncate text-left font-mono text-xs text-gray-300 hover:text-white"
                      >
                        {item.nonce_account}
                      </button>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(item.signature, `nonce-signature:${item.id}`)}
                        className="mt-2 block max-w-full truncate text-left font-mono text-xs text-gray-500 hover:text-white"
                      >
                        {item.signature}
                      </button>
                    </div>
                    <span className="shrink-0 rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-gray-300">
                      {effectiveRpcLabel}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          ) : effectiveWallet && !nonceLoading ? (
            <div className="rounded-2xl border border-dashed border-white/10 bg-white/[0.03] p-6 text-center text-sm text-gray-400">
              {t("features.create-nonce.noSavedAccounts")}
            </div>
          ) : null}
        </div>
      );
    };

    const currentMultisigs = workspace.multisigs.filter(
      (item) => item.network === currentNetwork(effectiveNetwork),
    );
    const currentPrograms = workspace.programs.filter(
      (item) => item.network === currentNetwork(effectiveNetwork),
    );
    const currentProposals = workspace.proposals.filter(
      (item) => item.network === currentNetwork(effectiveNetwork),
    );

    const renderSavedMultisigPicker = (field = "multisig") => (
      currentMultisigs.length > 0 ? (
        <div>
          <label className="block text-sm font-medium mb-2">{t("features.workspace.savedMultisigs")}</label>
          <select
            value={String(formData[field] ?? "")}
            onChange={(e) => handleFormChange(field, e.target.value || undefined)}
            className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
          >
            <option value="">{t("features.workspace.selectSavedMultisig")}</option>
            {currentMultisigs.map((item) => (
              <option key={`${item.network}-${item.address}`} value={item.address}>
                {workspaceLabel(item.label, item.address)}
              </option>
            ))}
          </select>
        </div>
      ) : null
    );

    const renderTokenMintInfo = () => {
      const mint = String(formData.mint ?? "").trim();
      if (!mint || !tokenMintInfo || tokenMintInfo.mint !== mint || tokenMintInfo.loading) {
        return null;
      }
      if (tokenMintInfo.error) {
        return (
          <p className="mt-2 text-xs text-red-300">
            {t("features.transfer-token.mintInfoErrorDetail", { message: tokenMintInfo.error })}
          </p>
        );
      }
      return null;
    };

    const renderMultisigInput = () => (
      <div className="space-y-2">
        {renderSavedMultisigPicker("multisig")}
        <div>
          <label className="block text-sm font-medium mb-2">{t("features.squads.multisig")}</label>
          <div className="flex gap-2">
            <input
              value={formData.multisig || ""}
              onChange={(e) => handleFormChange("multisig", e.target.value)}
              className="flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
              placeholder={t("features.squads.multisigPlaceholder")}
            />
            <button
              type="button"
              onClick={() => saveWorkspaceMultisig(formData.multisig, formData.vault)}
              disabled={!formData.multisig}
              className="px-3 py-2 bg-white/10 rounded-lg hover:bg-white/20 transition-colors disabled:opacity-40"
            >
              {t("features.workspace.save")}
            </button>
          </div>
        </div>
      </div>
    );

    const renderProposalInput = () => (
      <div className="space-y-2">
        {currentProposals.length > 0 && (
          <div>
            <label className="block text-sm font-medium mb-2">{t("features.workspace.savedProposals")}</label>
            <select
              value={String(formData.proposal ?? "")}
              onChange={(e) => {
                const selected = currentProposals.find((item) => item.address === e.target.value);
                handleFormChange("proposal", e.target.value || undefined);
                if (selected) {
                  handleFormChange("multisig", selected.multisig);
                  handleFormChange("transactionIndex", selected.transactionIndex);
                }
              }}
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
            >
              <option value="">{t("features.workspace.selectSavedProposal")}</option>
              {currentProposals.map((item) => (
                <option key={`${item.network}-${item.address}`} value={item.address}>
                  {`${item.kind || "proposal"} #${item.transactionIndex || "-"} - ${item.status || "unknown"}`}
                </option>
              ))}
            </select>
          </div>
        )}
        <div>
          <label className="block text-sm font-medium mb-2">{t("features.squads.proposal")}</label>
          <input
            value={formData.proposal || ""}
            onChange={(e) => handleFormChange("proposal", e.target.value)}
            className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
            placeholder={t("features.squads.proposalPlaceholder")}
          />
        </div>
      </div>
    );

    const renderProgramIdInput = () => (
      <div className="space-y-2">
        {currentPrograms.length > 0 && (
          <div>
            <label className="block text-sm font-medium mb-2">{t("features.workspace.savedPrograms")}</label>
            <select
              value={String(formData.programId ?? "")}
              onChange={(e) => handleFormChange("programId", e.target.value || undefined)}
              className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
            >
              <option value="">{t("features.workspace.selectSavedProgram")}</option>
              {currentPrograms.map((item) => (
                <option key={`${item.network}-${item.address}`} value={item.address}>
                  {workspaceLabel(item.label, item.address)}
                </option>
              ))}
            </select>
          </div>
        )}
        <div>
          <label className="block text-sm font-medium mb-2">{t("features.program-info.programId")}</label>
          <div className="flex gap-2">
            <input
              value={formData.programId || ""}
              onChange={(e) => handleFormChange("programId", e.target.value)}
              className="flex-1 px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
              placeholder={t("features.program-info.programIdPlaceholder")}
            />
            <button
              type="button"
              onClick={() => saveWorkspaceProgram(formData.programId)}
              disabled={!formData.programId}
              className="px-3 py-2 bg-white/10 rounded-lg hover:bg-white/20 transition-colors disabled:opacity-40"
            >
              {t("features.workspace.save")}
            </button>
          </div>
        </div>
      </div>
    );

    const renderBufferInput = () => (
      <div>
        <label className="block text-sm font-medium mb-2">{t("features.squads.buffer")}</label>
        <input
          value={formData.bufferAddress || ""}
          onChange={(e) => handleFormChange("bufferAddress", e.target.value)}
          className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
          placeholder={t("features.squads.bufferPlaceholder")}
        />
      </div>
    );

    const renderProgramFileInput = () => (
      <div>
        <label className="block text-sm font-medium mb-2">{t("features.program-deploy.programFile")}</label>
        <input
          type="file"
          accept=".so,application/octet-stream"
          onChange={handleProgramFileUpload}
          className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
        />
        {formData.programSoName && (
          <p className="mt-2 text-xs text-green-400">
            {t("features.program-deploy.fileReady", {
              filename: String(formData.programSoName),
              size: String(formData.programSoSize || 0),
            })}
          </p>
        )}
      </div>
    );

    const renderSpillInput = () => (
      <div>
        <label className="block text-sm font-medium mb-2">{t("features.squads.spill")}</label>
        <input
          value={formData.spillAddress || ""}
          onChange={(e) => handleFormChange("spillAddress", e.target.value)}
          className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
          placeholder={t("features.squads.spillPlaceholder")}
        />
      </div>
    );

    const renderMemoInput = () => (
      <div>
        <label className="block text-sm font-medium mb-2">{t("features.squads.memo")}</label>
        <input
          value={formData.memo || ""}
          onChange={(e) => handleFormChange("memo", e.target.value)}
          className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
          placeholder={t("features.squads.memoPlaceholder")}
        />
      </div>
    );

    const renderSquadsResult = () => {
      if (!formData.signature && !formData.vault && !formData.proposal) return null;
      return (
        <div className="space-y-3 p-4 bg-white/5 rounded-lg">
          {renderCopyRow("squads-multisig", t("features.squads.multisig"), formData.multisig)}
          {renderCopyRow("squads-vault", t("features.squads.vault"), formData.vault)}
          {renderCopyRow("squads-buffer", t("features.squads.buffer"), formData.bufferAddress)}
          {renderCopyRow("squads-proposal", t("features.squads.proposal"), formData.proposal)}
          {renderCopyRow("squads-transaction", t("features.squads.transaction"), formData.transaction)}
          {renderCopyRow("squads-programdata", t("features.program-info.programdataAddress"), formData.programdataAddress)}
          {renderCopyRow("squads-create-signature", t("features.squads.createSignature"), formData.createSignature)}
          {renderCopyRow("squads-authority-signature", t("features.program-deploy.authoritySignature"), formData.authoritySignature)}
          {renderCopyRow("squads-signature", t("formUi.txSignature"), formData.signature)}
          {formData.transactionIndex && (
            <p className="text-xs text-gray-400">
              {t("features.squads.transactionIndexValue", { index: String(formData.transactionIndex) })}
            </p>
          )}
          {formData.creationFeeLamports && (
            <p className="text-xs text-gray-400">
              {t("features.squads-create.creationFee", { lamports: String(formData.creationFeeLamports) })}
            </p>
          )}
          {formData.writeCount && (
            <p className="text-xs text-gray-400">
              {t("features.squads-prepare-upgrade-buffer.stats", {
                writes: String(formData.writeCount || 0),
                bytes: String(formData.programBytes || 0),
                rent: String(formData.rentLamports || 0),
              })}
            </p>
          )}
        </div>
      );
    };

    const openSquadsWorkspace = () => {
      setSelectedForm("squads-workspace");
      setBackTarget(null);
      handleFormChange("network", effectiveNetwork);
    };

    const openSquadsPrograms = () => {
      setSelectedForm("squads-programs");
      setBackTarget(null);
      handleFormChange("network", effectiveNetwork);
    };

    const openSquadsProposals = () => {
      setSelectedForm("squads-proposals");
      setBackTarget(null);
      handleFormChange("network", effectiveNetwork);
    };

    const squadsActionButtonClass = (active: boolean) =>
      `px-3 py-2 rounded-lg text-sm border transition-colors ${
        active
          ? "bg-gradient-to-r from-purple-500 to-pink-500 border-pink-300/40 text-white font-semibold shadow-sm shadow-purple-950/40"
          : "bg-white/10 border-white/10 text-gray-300 hover:bg-white/20 hover:text-white"
      }`;

    const renderTransactionList = (
      state: WalletTransactionsState | null,
      loadMore: () => void,
      emptyText: string,
      errorFallback: string,
      tokenSymbols?: Map<string, string>,
    ) => {
      const items = state?.transactions ?? [];
      const isLoading = Boolean(state?.loading);
      const canLoadMore = Boolean(state?.hasMore && !isLoading && items.length < MAX_TRANSACTION_HISTORY);
      const groupedItems = items.reduce<Array<{ date: string; transactions: WalletTransactionRecord[] }>>(
        (groups, transaction) => {
          const date = formatTransactionDate(transaction.block_time);
          const group = groups.find((item) => item.date === date);
          if (group) {
            group.transactions.push(transaction);
          } else {
            groups.push({ date, transactions: [transaction] });
          }
          return groups;
        },
        [],
      );

      return (
        <div className="space-y-4">
          {state?.error && (
            <p className="text-sm text-yellow-200">
              {state.error || errorFallback}
            </p>
          )}
          {groupedItems.map((group) => (
            <div key={group.date} className="space-y-2">
              <h4 className="px-1 text-sm font-semibold text-gray-400">{group.date}</h4>
              {group.transactions.map((transaction) => {
                const failed = Boolean(transaction.err);
                const solscanUrl = solscanTransactionUrl(transaction.signature, state?.network ?? effectiveNetwork);
                const changes = transaction.changes ?? [];
                const primaryIn = changes.find((change) => change.direction === "in");
                const primaryOut = changes.find((change) => change.direction === "out");
                const actionKey = failed ? "failed" : transaction.action || "transaction";
                const actionLabel = t(`features.wallet-list.transactionAction.${actionKey}`);
                const subtitle =
                  transaction.summary ||
                  (transaction.counterparty
                    ? t("features.wallet-list.transactionTo", { address: shortAddress(transaction.counterparty) })
                    : shortSignature(transaction.signature));
                return (
                  <div key={transaction.signature} className="rounded-xl bg-white/10 p-4 hover:bg-white/15">
                    <div className="flex items-center gap-4">
                      <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-black/40 text-sm font-bold text-violet-200 ring-1 ring-white/10">
                        {failed ? "!" : transaction.action === "swap" ? <ArrowRightLeft className="h-5 w-5" /> : transaction.action === "receive" ? <Download className="h-5 w-5" /> : <Send className="h-5 w-5" />}
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="text-base font-semibold text-gray-100">{actionLabel}</p>
                          {transaction.confirmation_status && (
                            <span className="rounded-full border border-white/10 bg-white/5 px-2 py-0.5 text-[11px] text-gray-400">
                              {transaction.confirmation_status}
                            </span>
                          )}
                        </div>
                        <p className="mt-1 truncate text-sm text-gray-400">{subtitle}</p>
                      </div>
                      <div className="shrink-0 text-right">
                        {primaryIn && (
                          <p className="text-base font-semibold text-emerald-300">{transactionAmountLabel(primaryIn, tokenSymbols)}</p>
                        )}
                        {primaryOut && (
                          <p className={primaryIn ? "mt-1 text-sm text-gray-200" : "text-base font-semibold text-gray-200"}>
                            {transactionAmountLabel(primaryOut, tokenSymbols)}
                          </p>
                        )}
                        {!primaryIn && !primaryOut && (
                          <p className="text-sm text-gray-400">{formatTransactionTime(transaction.block_time) || t("features.wallet-list.transactionUnknownAmount")}</p>
                        )}
                      </div>
                    </div>
                    <div className="mt-3 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-gray-500">
                      {transaction.counterparty && (
                        <span>{t("features.wallet-list.transactionCounterparty", { address: shortAddress(transaction.counterparty) })}</span>
                      )}
                      <span>{formatTransactionTime(transaction.block_time)}</span>
                      <span>{t("features.wallet-list.transactionSlot", { slot: transaction.slot })}</span>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(transaction.signature, `wallet-tx:${transaction.signature}`)}
                        className="font-mono hover:text-white"
                        title={transaction.signature}
                      >
                        {copied === `wallet-tx:${transaction.signature}` ? t("common.copied") : shortSignature(transaction.signature)}
                      </button>
                      <button
                        type="button"
                        onClick={() => window.open(solscanUrl, "_blank", "noopener,noreferrer")}
                        className="inline-flex h-7 w-7 items-center justify-center rounded-lg bg-white/10 text-gray-300 transition-colors hover:bg-white/20 hover:text-white"
                        title="Solscan"
                        aria-label="Open transaction in Solscan"
                      >
                        <ExternalLink className="h-3.5 w-3.5" />
                      </button>
                    </div>
                    {transaction.memo && (
                      <p className="mt-2 text-xs text-gray-500">{transaction.memo}</p>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
          {!isLoading && items.length === 0 && (
            <div className="rounded-lg border border-dashed border-white/10 bg-white/[0.03] p-6 text-center text-sm text-gray-400">
              {emptyText}
            </div>
          )}
          {(isLoading || canLoadMore) && (
            <button
              type="button"
              onClick={loadMore}
              disabled={isLoading}
              className="w-full rounded-lg bg-white/10 px-4 py-3 text-sm font-semibold text-gray-200 hover:bg-white/20 disabled:opacity-50"
            >
              {isLoading
                ? t("common.loading")
                : t("features.wallet-list.loadMoreTransactions", {
                    count: Math.min(TRANSACTION_PAGE_SIZE, MAX_TRANSACTION_HISTORY - items.length),
                  })}
            </button>
          )}
        </div>
      );
    };

    const renderSquadsWorkspaceHeader = () => (
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={openSquadsWorkspace}
          className={squadsActionButtonClass(formId === "squads-workspace")}
        >
          {t("features.workspace.savedMultisigs")}
        </button>
        <button
          type="button"
          onClick={openSquadsProposals}
          className={squadsActionButtonClass(formId === "squads-proposals")}
        >
          {t("features.workspace.savedProposals")}
        </button>
        <button
          type="button"
          onClick={openSquadsPrograms}
          className={squadsActionButtonClass(formId === "squads-programs")}
        >
          {t("features.workspace.savedPrograms")}
        </button>
        <details data-close-on-outside className="relative ml-auto">
          <summary className="cursor-pointer list-none px-3 py-2 bg-white/10 rounded-lg text-sm hover:bg-white/20">
            {t("features.workspace.actions")}
          </summary>
          <div className="absolute right-0 z-20 mt-2 w-72 overflow-hidden rounded-lg border border-white/10 bg-zinc-950 shadow-xl">
            {[
              { id: "squads-create", label: t("features.squads-create.title"), onClick: () => handleOpenSquadsForm("squads-create") },
              { id: "squads-info", label: t("features.squads-info.title"), onClick: () => handleOpenSquadsForm("squads-info") },
              { id: "squads-sol-transfer", label: t("features.squads-sol-transfer.title"), onClick: () => handleOpenSquadsForm("squads-sol-transfer") },
              { id: "squads-token-transfer", label: t("features.squads-token-transfer.title"), onClick: () => handleOpenSquadsForm("squads-token-transfer") },
              { id: "squads-prepare-upgrade-buffer", label: t("features.squads-prepare-upgrade-buffer.title"), onClick: () => handleOpenSquadsForm("squads-prepare-upgrade-buffer") },
              { id: "squads-program-upgrade", label: t("features.squads-program-upgrade.title"), onClick: () => handleOpenSquadsForm("squads-program-upgrade") },
              { id: "squads-set-authority", label: t("features.squads-set-authority.title"), onClick: () => handleOpenSquadsForm("squads-set-authority") },
            ].map((action) => (
              <button
                key={action.id}
                type="button"
                onClick={() => {
                  action.onClick();
                  closeDropdownMenus();
                }}
                className="block w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-white/10"
              >
                {action.label}
              </button>
            ))}
          </div>
        </details>
      </div>
    );

    const terminalProposalStatuses = new Set(["executed", "rejected", "cancelled"]);
    const activeProposals = currentProposals.filter(
      (item) => !terminalProposalStatuses.has(String(item.status || "").toLowerCase()),
    );
    const historicalProposals = currentProposals.filter(
      (item) => terminalProposalStatuses.has(String(item.status || "").toLowerCase()),
    );

    const renderStatusBadge = (status: string | undefined) => {
      const value = status || "unknown";
      const normalized = value.toLowerCase();
      const tone = normalized === "executed"
        ? "bg-green-500/15 text-green-300 border-green-400/20"
        : normalized === "rejected" || normalized === "cancelled"
          ? "bg-red-500/15 text-red-300 border-red-400/20"
          : "bg-yellow-500/15 text-yellow-200 border-yellow-400/20";
      return (
        <span className={`px-2 py-0.5 rounded border text-xs ${tone}`}>
          {value}
        </span>
      );
    };

    const renderProposalRecord = (item: WorkspaceProposal, showActions: boolean) => (
      <div key={`${item.network}-${item.address}`} className="p-3 bg-white/5 rounded-lg space-y-3">
        <div className="flex flex-col md:flex-row md:items-start md:justify-between gap-3">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <p className="text-sm font-medium">{`${item.kind || "proposal"} #${item.transactionIndex || "-"}`}</p>
              {renderStatusBadge(item.status)}
            </div>
            <p className="text-xs text-gray-500">{t("features.workspace.proposalMultisig", { multisig: shortAddress(item.multisig) })}</p>
            {actorLabel(item) && (
              <p className="text-xs text-gray-500">{t("features.workspace.createdBy", { wallet: actorLabel(item) })}</p>
            )}
            <code className="block text-xs text-gray-400 break-all">{item.address}</code>
          </div>
          <div className="flex flex-wrap gap-2 shrink-0">
            {showActions && (
              <>
                <button type="button" onClick={() => requestProposalPasswordSubmit(item, "approve")} disabled={loading} className="px-2 py-1 bg-white/10 rounded text-xs hover:bg-white/20 disabled:opacity-40">
                  {t("features.squads-approve.approveButton")}
                </button>
                <button type="button" onClick={() => requestProposalPasswordSubmit(item, "reject")} disabled={loading} className="px-2 py-1 bg-white/10 rounded text-xs hover:bg-white/20 disabled:opacity-40">
                  {t("features.squads-reject.rejectButton")}
                </button>
                <button type="button" onClick={() => requestProposalPasswordSubmit(item, "execute")} disabled={loading} className="px-2 py-1 bg-white/10 rounded text-xs hover:bg-white/20 disabled:opacity-40">
                  {t("features.squads-execute.executeButton")}
                </button>
              </>
            )}
            <button type="button" onClick={() => removeWorkspaceItem("proposals", item.address, item.network)} className="px-2 py-1 bg-white/10 rounded text-xs hover:bg-white/20">
              {t("features.workspace.remove")}
            </button>
          </div>
        </div>
      </div>
    );

    const renderWorkspacePanel = () => (
      <div className="space-y-4">
        {renderSquadsWorkspaceHeader()}
        <div className="space-y-3">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-sm font-semibold text-gray-300">{t("features.workspace.savedMultisigs")}</h3>
            <span className="text-xs text-gray-500">{currentMultisigs.length}</span>
          </div>
          {currentMultisigs.length === 0 ? (
            <p className="text-xs text-gray-500">{t("features.workspace.emptyMultisigs")}</p>
          ) : currentMultisigs.map((item) => (
            <div key={`${item.network}-${item.address}`} className="p-3 bg-white/5 rounded-lg space-y-2">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <p className="text-sm font-medium">{workspaceLabel(item.label, item.address)}</p>
                  {actorLabel(item) && (
                    <p className="text-xs text-gray-500">{t("features.workspace.createdBy", { wallet: actorLabel(item) })}</p>
                  )}
                  <code className="block text-xs text-gray-400 break-all">{item.address}</code>
                </div>
                <details data-close-on-outside className="relative shrink-0">
                  <summary className="cursor-pointer list-none px-3 py-1.5 bg-white/10 rounded text-xs hover:bg-white/20">
                    {t("features.workspace.actions")}
                  </summary>
                  <div className="absolute right-0 z-20 mt-2 w-64 overflow-hidden rounded-lg border border-white/10 bg-zinc-950 shadow-xl">
                    {[
                      { id: "info", label: t("features.squads-info.queryButton"), onClick: () => handleOpenSquadsForm("squads-info", { multisig: item.address, network: item.network }) },
                      { id: "refresh", label: t("features.workspace.refresh"), onClick: () => void refreshWorkspaceProposals(item) },
                      { id: "sol", label: t("features.squads-sol-transfer.createButton"), onClick: () => handleOpenSquadsForm("squads-sol-transfer", { multisig: item.address, network: item.network }) },
                      { id: "token", label: t("features.squads-token-transfer.createButton"), onClick: () => handleOpenSquadsForm("squads-token-transfer", { multisig: item.address, network: item.network }) },
                      { id: "upgrade", label: t("features.squads-program-upgrade.createButton"), onClick: () => handleOpenSquadsForm("squads-program-upgrade", { multisig: item.address, network: item.network }) },
                      { id: "authority", label: t("features.squads-set-authority.setButton"), onClick: () => handleOpenSquadsForm("squads-set-authority", { multisig: item.address, network: item.network }) },
                      { id: "remove", label: t("features.workspace.remove"), onClick: () => removeWorkspaceItem("multisigs", item.address, item.network) },
                    ].map((action) => (
                      <button
                        key={action.id}
                        type="button"
                        onClick={() => {
                          action.onClick();
                          closeDropdownMenus();
                        }}
                        className="block w-full px-3 py-2 text-left text-sm text-gray-200 hover:bg-white/10"
                      >
                        {action.label}
                      </button>
                    ))}
                  </div>
                </details>
              </div>
            </div>
          ))}
        </div>
        {renderSquadsResult()}
      </div>
    );

    const renderProposalWorkspacePanel = () => (
      <div className="space-y-4">
        {renderSquadsWorkspaceHeader()}
        <div>
          <label className="block text-sm font-medium mb-2">{t("features.squads.memo")}</label>
          <input
            value={formData.memo || ""}
            onChange={(e) => handleFormChange("memo", e.target.value)}
            className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
            placeholder={t("features.workspace.voteMemoPlaceholder")}
          />
        </div>
        <button
          type="button"
          onClick={() => void refreshWorkspaceProposals()}
          className="px-3 py-2 bg-white/10 rounded-lg hover:bg-white/20 transition-colors"
        >
          {t("features.workspace.refreshProposals")}
        </button>
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-gray-300">{t("features.workspace.activeProposals")}</h3>
          {activeProposals.length === 0 ? (
            <p className="text-xs text-gray-500">{t("features.workspace.emptyActiveProposals")}</p>
          ) : activeProposals.map((item) => renderProposalRecord(item, true))}
        </div>
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-gray-300">{t("features.workspace.proposalHistory")}</h3>
          {historicalProposals.length === 0 ? (
            <p className="text-xs text-gray-500">{t("features.workspace.emptyProposalHistory")}</p>
          ) : historicalProposals.map((item) => renderProposalRecord(item, false))}
        </div>
        {renderSquadsResult()}
      </div>
    );

    const renderStepForm = (steps: Array<{ title: string; content: React.ReactNode }>) => {
      const rawStep = Number(formData.stepIndex || 0);
      const stepIndex = Number.isInteger(rawStep)
        ? Math.min(Math.max(rawStep, 0), steps.length - 1)
        : 0;

      return (
        <div className="space-y-4">
          <div className="flex gap-2 overflow-x-auto pb-1">
            {steps.map((step, index) => (
              <button
                key={step.title}
                type="button"
                onClick={() => handleFormChange("stepIndex", index)}
                className={`min-w-[7rem] flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-colors whitespace-nowrap ${
                  index === stepIndex
                    ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                    : "bg-white/5 text-gray-400 hover:bg-white/10"
                }`}
              >
                {index + 1}. {step.title}
              </button>
            ))}
          </div>
          <div className="space-y-4">{steps[stepIndex]?.content}</div>
          <div className="grid grid-cols-2 gap-2">
            <button
              type="button"
              onClick={() => handleFormChange("stepIndex", stepIndex - 1)}
              disabled={stepIndex === 0}
              className="px-3 py-2 bg-white/10 rounded-lg hover:bg-white/20 transition-colors disabled:opacity-40"
            >
              {t("features.workspace.prevStep")}
            </button>
            <button
              type="button"
              onClick={() => handleFormChange("stepIndex", stepIndex + 1)}
              disabled={stepIndex >= steps.length - 1}
              className="px-3 py-2 bg-white/10 rounded-lg hover:bg-white/20 transition-colors disabled:opacity-40"
            >
              {t("features.workspace.nextStep")}
            </button>
          </div>
        </div>
      );
    };

    const renderFormBody = () => {
      switch (formId) {
      case "wallet-list":
        return renderWalletListPanel();

      case "settings":
        return (
          <div className="space-y-6">
            <section className="space-y-3 rounded-2xl border border-white/10 bg-white/5 p-4">
              <div>
                <h3 className="text-sm font-semibold text-gray-200">{t("features.settings.networkTitle")}</h3>
                <p className="mt-1 text-xs text-gray-500">{t("features.settings.networkHint")}</p>
              </div>
              <div className="grid grid-cols-3 gap-2 rounded-xl border border-white/10 bg-black/20 p-1">
                {(["mainnet", "devnet", "testnet"] as AppNetwork[]).map((network) => {
                  const isNetworkSelected = settingsNetwork === network;
                  return (
                    <button
                      key={network}
                      type="button"
                      onClick={() => setAppNetwork(network)}
                      className={`h-10 rounded-lg text-sm font-medium transition-colors ${
                        isNetworkSelected
                          ? "bg-white text-black shadow-sm"
                          : "text-gray-300 hover:bg-white/10 hover:text-white"
                      }`}
                    >
                      {networkLabel(t, network)}
                    </button>
                  );
                })}
              </div>
              <div className="space-y-2">
                {visibleRpcProfiles.map((profile) => {
                  const isSelected = selectedRpcId === profile.id;
                  return (
                    <div
                      key={profile.id}
                      className={`rounded-lg border p-3 transition-colors ${
                        isSelected
                          ? "border-violet-300/50 bg-violet-400/15"
                          : "border-white/10 bg-white/5"
                      }`}
                    >
                      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                        <button
                          type="button"
                          onClick={() => setAppRpc(profile.id)}
                          className="min-w-0 flex-1 text-left"
                        >
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="text-sm font-semibold text-gray-100">{profile.name}</span>
                            <span className="rounded-full border border-white/10 bg-black/20 px-2 py-0.5 text-[11px] text-gray-400">
                              {profile.builtin ? t("features.settings.rpcBuiltin") : t("features.settings.rpcCustom")}
                            </span>
                            {isSelected && (
                              <span className="rounded-full bg-emerald-400/15 px-2 py-0.5 text-[11px] font-medium text-emerald-200">
                                {t("features.settings.rpcActive")}
                              </span>
                            )}
                          </div>
                          <p className="mt-1 truncate font-mono text-xs text-gray-500">{profile.url}</p>
                        </button>
                        {!profile.builtin && (
                          <button
                            type="button"
                            onClick={() => handleRemoveRpcProfile(profile.id)}
                            className="inline-flex h-8 items-center justify-center gap-1 rounded-lg border border-white/10 bg-white/5 px-2 text-xs text-gray-300 hover:bg-red-500/15 hover:text-red-100"
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                            {t("features.settings.rpcRemove")}
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>

              <div className="grid gap-2 border-t border-white/10 pt-3 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)_auto]">
                <input
                  type="text"
                  value={newRpcName}
                  onChange={(event) => setNewRpcName(event.target.value)}
                  className="h-10 min-w-0 rounded-lg border border-white/10 bg-black/20 px-3 text-sm text-white outline-none focus:ring-2 focus:ring-white/20"
                  placeholder={t("features.settings.rpcNamePlaceholder")}
                />
                <input
                  type="url"
                  value={newRpcUrl}
                  onChange={(event) => setNewRpcUrl(event.target.value)}
                  className="h-10 min-w-0 rounded-lg border border-white/10 bg-black/20 px-3 text-sm text-white outline-none focus:ring-2 focus:ring-white/20"
                  placeholder={t("features.settings.rpcUrlPlaceholder")}
                />
                <button
                  type="button"
                  onClick={handleAddRpcProfile}
                  className="inline-flex h-10 items-center justify-center gap-2 rounded-lg bg-white px-3 text-sm font-semibold text-black hover:bg-gray-200"
                >
                  <Plus className="h-4 w-4" />
                  {t("features.settings.rpcAdd")}
                </button>
              </div>
              <p className="text-xs text-gray-500">
                {t("features.settings.rpcAddHint", { network: networkLabel(t, settingsNetwork) })}
              </p>
            </section>

            <section className="rounded-2xl border border-white/10 bg-white/5 p-4">
              {renderWalletAccountManager()}
            </section>

            <section className="space-y-3 rounded-2xl border border-white/10 bg-white/5 p-4">
              <div>
                <h3 className="text-sm font-semibold text-gray-200">{t("features.settings.walletToolsTitle")}</h3>
                <p className="mt-1 text-xs text-gray-500">{t("features.settings.walletToolsHint")}</p>
              </div>
              {renderActionGrid([
                { id: "create-keystore", title: t("features.create-keystore.title"), icon: <Key className="w-4 h-4" /> },
                { id: "import-keystore", title: t("features.import-keystore.title"), icon: <Upload className="w-4 h-4" /> },
              ])}
            </section>

            <section className="space-y-3 rounded-2xl border border-white/10 bg-white/5 p-4">
              <div>
                <h3 className="text-sm font-semibold text-gray-200">{t("features.settings.securityTitle")}</h3>
                <p className="mt-1 text-xs text-gray-500">{t("features.settings.securityHint")}</p>
              </div>
              {renderActionGrid([
                { id: "setup-2fa", title: t("features.setup-2fa.title"), icon: <Lock className="w-4 h-4" /> },
                { id: "create-tfa", title: t("features.create-tfa.title"), icon: <ShieldCheck className="w-4 h-4" /> },
                { id: "unlock-tfa", title: t("features.unlock-tfa.title"), icon: <Unlock className="w-4 h-4" /> },
              ])}
            </section>
          </div>
        );

      case "wsol-workbench":
        return (
          <div className="space-y-4">
            {renderActionGrid([
              { id: "create-wsol-ata", title: t("features.create-wsol-ata.title"), icon: <Hash className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
              { id: "wrap-sol", title: t("features.wrap-sol.title"), icon: <RefreshCw className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
              { id: "unwrap-sol", title: t("features.unwrap-sol.title"), icon: <ArrowRightLeft className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
              { id: "close-wsol-ata", title: t("features.close-wsol-ata.title"), icon: <X className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
            ])}
          </div>
        );

      case "pump-workbench":
        return (
          <div className="space-y-4">
            {renderActionGrid([
              { id: "pumpfun-sell", title: t("features.pumpfun-sell.title"), icon: <Coins className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
              { id: "pumpswap-sell", title: t("features.pumpswap-sell.title"), icon: <Coins className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
              { id: "pumpfun-cashback", title: t("features.pumpfun-cashback.title"), icon: <Download className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
              { id: "pumpswap-cashback", title: t("features.pumpswap-cashback.title"), icon: <Download className="w-4 h-4" />, preset: { wallet_id: effectiveWalletId, network: effectiveNetwork } },
            ])}
          </div>
        );

      case "program-workbench":
        return renderProgramWorkspacePanel();

      case "create-nonce":
      case "nonce-workbench":
        return (
          <>
            {renderNonceAccountList()}
            {nonceCreateOpen &&
              typeof document !== "undefined" &&
              createPortal(
                <div
                  className="fixed inset-0 z-[190] flex items-end bg-black/60"
                  onClick={closeNonceCreateDialog}
                >
                  <div
                    className="relative max-h-[90vh] w-full overflow-y-auto border-t border-white/10 bg-zinc-950 px-4 py-5 shadow-2xl"
                    onClick={(event) => event.stopPropagation()}
                  >
                    <div className="mx-auto max-w-xl space-y-4">
                      <div className="flex items-start justify-between gap-4">
                        <div className="min-w-0">
                          <h3 className="text-lg font-semibold">{t("features.create-nonce.createButton")}</h3>
                        </div>
                        <button
                          type="button"
                          onClick={closeNonceCreateDialog}
                          disabled={loading}
                          className="rounded-lg bg-white/10 p-2 text-gray-300 hover:bg-white/20 disabled:opacity-50"
                        >
                          <X className="h-4 w-4" />
                        </button>
                      </div>
                      {renderNonceCreateForm()}
                      {renderNonceCreateResult()}
                    </div>
                  </div>
                </div>,
                document.body,
              )}
          </>
        );

      case "squads-workspace":
        return renderWorkspacePanel();

      case "squads-proposals":
        return renderProposalWorkspacePanel();

      case "squads-programs":
        return renderProgramWorkspacePanel(true);

      case "create-plain":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.create-plain.name")}</label>
              <input
                type="text"
                value={formData.name || ""}
                onChange={(e) => handleFormChange("name", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.create-plain.namePlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => handleSubmit("create-plain")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.create-plain.creating") : t("features.create-plain.createButton")}
            </button>
            {formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.publicKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.publicKey as string, "pubkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "pubkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.secretKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.secretKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.secretKey as string, "privkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "privkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "create-encrypted":
        return (
          <div className="space-y-4">
            <button type="button"
              onClick={() => requestCreatePasswordSubmit("create-encrypted")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.create-encrypted.creating") : t("features.create-encrypted.createButton")}
            </button>
            {formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.publicKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.publicKey as string, "enc-pubkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "enc-pubkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-encrypted.encryptedKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.encryptedKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.encryptedKey as string, "enc-privkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "enc-privkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "create-keystore":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("formUi.walletNameRequiredLabel")}</label>
              <input
                type="text"
                value={formData.name || ""}
                onChange={(e) => handleFormChange("name", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("formUi.walletNamePlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestCreatePasswordSubmit("create-keystore")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.create-keystore.creating") : t("features.create-keystore.createButton")}
            </button>
            {formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.publicKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.publicKey as string, "ks-pubkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "ks-pubkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <p className="text-sm text-green-400">{t("formUi.walletSaved")}</p>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-keystore.keystoreJson")}</label>
                  <div className="flex gap-2">
                    <textarea
                      readOnly
                      className="flex-1 px-3 py-2 bg-black/30 rounded text-xs h-24 resize-none"
                      value={formData.keystoreJson ?? ""}
                    />
                    <button type="button"
                      onClick={() => copyToClipboard(formData.keystoreJson as string, "ks-json")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "ks-json" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <button type="button"
                  onClick={() => downloadFile(formData.keystoreJson as string, "keystore.json")}
                  className="w-full py-2 bg-white/10 rounded-lg hover:bg-white/20 transition-colors flex items-center justify-center gap-2"
                >
                  <Download className="w-4 h-4" />
                  {t("features.create-keystore.downloadFile")}
                </button>
              </div>
            )}
          </div>
        );

      case "import-keystore":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("formUi.walletName")}</label>
              <input
                type="text"
                value={formData.name || ""}
                onChange={(e) => handleFormChange("name", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("formUi.walletNamePlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.import-keystore.selectFile")}</label>
              <input
                type="file"
                accept=".json"
                onChange={handleFileUpload}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
              />
            </div>
            <div className="text-center text-gray-400">{t("features.import-keystore.or")}</div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.import-keystore.pasteJson")}</label>
              <textarea
                value={formData.keystoreJson || ""}
                onChange={(e) => {
                  const content = e.target.value;
                  setFormData((prev) => ({
                    ...prev,
                    keystoreJson: content,
                    ...(!String(prev.name ?? "").trim() ? { name: keystoreMetadataName(content) } : {}),
                  }));
                }}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                placeholder={t("features.import-keystore.jsonPlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestCreatePasswordSubmit("import-keystore")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.import-keystore.importing") : t("features.import-keystore.importButton")}
            </button>
            {formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <code className="block px-3 py-2 bg-black/30 rounded text-xs break-all">
                    {formData.publicKey}
                  </code>
                </div>
                <p className="text-sm text-green-400">{t("formUi.walletSaved")}</p>
              </div>
            )}
            {wallets.length > 0 && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div className="flex items-center justify-between">
                  <label className="block text-sm font-medium">{t("formUi.walletList")}</label>
                  <button
                    type="button"
                    onClick={() => void loadWallets()}
                    className="px-3 py-1.5 bg-white/10 rounded text-xs hover:bg-white/20 transition-colors"
                  >
                    {walletsLoading ? t("common.loading") : t("formUi.refreshWallets")}
                  </button>
                </div>
                <div className="space-y-2">
                  {wallets.map((wallet) => (
                    <div key={wallet.id} className="flex items-center gap-2 p-3 bg-black/30 rounded-lg">
                      <div className="min-w-0 flex-1">
                        <p className="font-medium truncate">{wallet.name}</p>
                        <p className="text-xs text-gray-400 break-all">{wallet.public_key}</p>
                      </div>
                      <button
                        type="button"
                        onClick={() => copyToClipboard(wallet.public_key, `wallet-${wallet.id}`)}
                        className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                      >
                        {copied === `wallet-${wallet.id}` ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        );

      case "decrypt": {
        const decryptAuth = walletAuth("decrypt");
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "decrypt": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.secretKey;
                    delete newFormData.encryptedKey;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    decryptAuth === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "decrypt": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.secretKey;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    decryptAuth === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "decrypt": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encryptedKey;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    decryptAuth === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {decryptAuth === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && decryptAuth === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && decryptAuth === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.secretKey || ""}
                  onChange={(e) => handleFormChange("secretKey", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintext")}</p>
              </div>
            )}

            <button type="button"
              onClick={() => requestPasswordSubmit("decrypt")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.decrypt.decrypting") : t("features.decrypt.decryptButton")}
            </button>
            {formData.unlocked && formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <code className="block px-3 py-2 bg-black/30 rounded text-xs break-all">
                    {formData.publicKey}
                  </code>
                </div>
                <p className="text-sm text-green-400">{t("formUi.walletUnlockedNoSecret")}</p>
              </div>
            )}
          </div>
        );
      }

      case "unlock": {
        const unlockAuth = walletAuth("unlock");
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "unlock": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.secretKey;
                    delete newFormData.encryptedKey;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    unlockAuth === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "unlock": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.secretKey;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    unlockAuth === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "unlock": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encryptedKey;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    unlockAuth === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {unlockAuth === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && unlockAuth === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && unlockAuth === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.secretKey || ""}
                  onChange={(e) => handleFormChange("secretKey", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintext")}</p>
              </div>
            )}

            <button type="button"
              onClick={() => requestPasswordSubmit("unlock")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.unlock.unlocking") : t("features.unlock.unlockButton")}
            </button>
            {formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <code className="block px-3 py-2 bg-black/30 rounded text-xs break-all">
                    {formData.publicKey}
                  </code>
                </div>
                <p className="text-sm text-green-400">{t("formUi.walletUnlockedNoSecret")}</p>
              </div>
            )}
          </div>
        );
      }

      case "check-balance":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.check-balance.address")}</label>
              <input
                type="text"
                value={formData.address || ""}
                onChange={(e) => handleFormChange("address", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.check-balance.addressPlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => handleSubmit("check-balance")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.check-balance.checking") : t("features.check-balance.checkButton")}
            </button>
            {formData.balance && (
              <div className="p-4 bg-white/5 rounded-lg">
                <p className="text-2xl font-bold text-center">
                  {formData.balance} SOL
                  {formData.network && ` (${formData.network})`}
                </p>
              </div>
            )}
          </div>
        );

      case "get-pubkey": {
        const getPubkeyAuth = walletAuth("get-pubkey");
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "get-pubkey": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.secretKey;
                    delete newFormData.encryptedKey;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    getPubkeyAuth === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "get-pubkey": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.secretKey;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    getPubkeyAuth === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "get-pubkey": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encryptedKey;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    getPubkeyAuth === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {getPubkeyAuth === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && getPubkeyAuth === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[100px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && getPubkeyAuth === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.secretKey || ""}
                  onChange={(e) => handleFormChange("secretKey", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextSuggestEnc")}</p>
              </div>
            )}

            <button type="button"
              onClick={() => requestPasswordSubmit("get-pubkey")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.get-pubkey.getting") : t("features.get-pubkey.getButton")}
            </button>
            {formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.publicKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.publicKey as string, "get-pubkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "get-pubkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );
      }

      case "transfer-sol":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "transfer-sol": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("transfer-sol") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "transfer-sol": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("transfer-sol") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "transfer-sol": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("transfer-sol") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {/* Keystore Input */}
            {walletAuth("transfer-sol") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("transfer-sol") === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("transfer-sol") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.senderPrivateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium mb-2">{t("features.transfer-sol.toAddress")}</label>
              <input
                type="text"
                value={formData.to_address || ""}
                onChange={(e) => handleFormChange("to_address", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.transfer-sol.addressPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.transfer-sol.amount")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={formData.amount || ""}
                onChange={(e) => handleFormChange("amount", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.transfer-sol.amountPlaceholder")}
              />
              <div className="mt-2 rounded-lg border border-white/10 bg-black/20 px-3 py-2">
                <div className="flex items-center justify-between gap-2">
                  <p className="text-xs text-gray-500">{t("features.transfer-sol.walletBalance")}</p>
                  <button
                    type="button"
                    onClick={() => refreshCurrentWalletAssets(transferSolWallet || undefined)}
                    className="inline-flex h-6 w-6 items-center justify-center rounded-md bg-white/10 text-gray-300 hover:bg-white/20 hover:text-white"
                    title={t("features.wallet-list.refreshAssets")}
                    aria-label={t("features.wallet-list.refreshAssets")}
                  >
                    <RefreshCw className={`h-3.5 w-3.5 ${transferSolAssets?.refreshing ? "animate-spin" : ""}`} />
                  </button>
                </div>
                <p className="mt-1 font-mono text-sm text-gray-100">
                  {transferSolBalance} SOL
                </p>
                {transferSolAssets?.refreshing && (
                  <p className="mt-1 text-xs text-gray-500">{t("features.transfer-sol.balanceRefreshing")}</p>
                )}
              </div>
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("transfer-sol")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.transfer-sol.transferring") : t("features.transfer-sol.transferButton")}
            </button>
            {formData.signature && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.txSignature")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.signature}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.signature as string, "txsig")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "txsig" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "transfer-token":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "transfer-token": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("transfer-token") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "transfer-token": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("transfer-token") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "transfer-token": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("transfer-token") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("transfer-token") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("transfer-token") === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("transfer-token") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.senderPrivateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium mb-2">{t("features.transfer-token.toAddress")}</label>
              <input
                type="text"
                value={formData.to_address || ""}
                onChange={(e) => handleFormChange("to_address", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.transfer-token.addressPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.transfer-token.mintAddress")}</label>
              <input
                type="text"
                value={formData.mint || ""}
                onChange={(e) => handleFormChange("mint", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.transfer-token.mintPlaceholder")}
              />
              {renderTokenMintInfo()}
              {transferTokenMint && (
                <div className="mt-2 rounded-lg border border-white/10 bg-black/20 px-3 py-2">
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-xs text-gray-500">{t("features.transfer-token.walletBalance")}</p>
                    <button
                      type="button"
                      onClick={() => refreshCurrentWalletAssets(transferTokenWallet || undefined)}
                      className="inline-flex h-6 w-6 items-center justify-center rounded-md bg-white/10 text-gray-300 hover:bg-white/20 hover:text-white"
                      title={t("features.wallet-list.refreshAssets")}
                      aria-label={t("features.wallet-list.refreshAssets")}
                    >
                      <RefreshCw className={`h-3.5 w-3.5 ${transferTokenAssets?.refreshing ? "animate-spin" : ""}`} />
                    </button>
                  </div>
                  <p className="mt-1 font-mono text-sm text-gray-100">
                    {transferTokenBalance?.amount ?? "0"}
                  </p>
                  {transferTokenAssets?.refreshing && (
                    <p className="mt-1 text-xs text-gray-500">{t("features.transfer-token.balanceRefreshing")}</p>
                  )}
                </div>
              )}
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.transfer-token.amount")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={formData.amount || ""}
                onChange={(e) => handleFormChange("amount", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.transfer-token.amountPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.transfer-token.decimals")}</label>
              <input
                value={formData.decimals ?? ""}
                readOnly
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg text-white/80"
                placeholder={t("features.transfer-token.decimalsPlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("transfer-token")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.transfer-token.transferring") : t("features.transfer-token.transferButton")}
            </button>
            {formData.signature && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.txSignature")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.signature}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.signature as string, "token-txsig")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "token-txsig" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "create-wsol-ata":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "create-wsol-ata": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("create-wsol-ata") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "create-wsol-ata": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("create-wsol-ata") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "create-wsol-ata": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("create-wsol-ata") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("create-wsol-ata") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("create-wsol-ata") === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("create-wsol-ata") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}
            <button type="button"
              onClick={() => requestPasswordSubmit("create-wsol-ata")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.create-wsol-ata.creating") : t("features.create-wsol-ata.createButton")}
            </button>
            {formData.signature && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.txSignature")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.signature}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.signature as string, "wsol-ata-sig")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "wsol-ata-sig" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "wrap-sol":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "wrap-sol": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("wrap-sol") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "wrap-sol": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("wrap-sol") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "wrap-sol": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("wrap-sol") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("wrap-sol") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("wrap-sol") === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("wrap-sol") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium mb-2">{t("features.wrap-sol.amount")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={formData.amount || ""}
                onChange={(e) => handleFormChange("amount", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.wrap-sol.amountPlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("wrap-sol")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.wrap-sol.wrapping") : t("features.wrap-sol.wrapButton")}
            </button>
            {formData.signature && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.txSignature")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.signature}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.signature as string, "wrap-sig")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "wrap-sig" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "unwrap-sol":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "unwrap-sol": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("unwrap-sol") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "unwrap-sol": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("unwrap-sol") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "unwrap-sol": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("unwrap-sol") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("unwrap-sol") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("unwrap-sol") === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("unwrap-sol") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}
            <button type="button"
              onClick={() => requestPasswordSubmit("unwrap-sol")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.unwrap-sol.unwrapping") : t("features.unwrap-sol.unwrapButton")}
            </button>
            {formData.signature && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.txSignature")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.signature}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.signature as string, "unwrap-sig")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "unwrap-sig" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "program-deploy": {
        const deployAuth = walletAuth("program-deploy");
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "program-deploy": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    deployAuth === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "program-deploy": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    deployAuth === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "program-deploy": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    deployAuth === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {deployAuth === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json,application/json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && deployAuth === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && deployAuth === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}

            {renderProgramFileInput()}
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.program-deploy.maxDataLen")}</label>
              <input
                type="text"
                inputMode="numeric"
                autoComplete="off"
                value={formData.max_data_len || ""}
                onChange={(e) => handleFormChange("max_data_len", e.target.value.replace(/[^\d]/g, ""))}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.program-deploy.maxDataLenPlaceholder")}
              />
            </div>
            {renderSavedMultisigPicker("squadsDeployMultisig")}
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.program-deploy.squadsMultisig")}</label>
              <input
                value={formData.squadsDeployMultisig || ""}
                onChange={(e) => handleFormChange("squadsDeployMultisig", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.program-deploy.squadsMultisigPlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("program-deploy")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.program-deploy.deploying") : t("features.program-deploy.deployButton")}
            </button>
            {formData.programId && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                {[
                  ["program-id", t("features.program-deploy.programId"), formData.programId],
                  ["programdata", t("features.program-deploy.programdataAddress"), formData.programdataAddress],
                  ["program-authority", t("features.program-deploy.authority"), formData.authority],
                  ["program-vault", t("features.squads.vault"), formData.vault],
                  ["deploy-sig", t("formUi.txSignature"), formData.signature],
                  ["authority-sig", t("features.program-deploy.authoritySignature"), formData.authoritySignature],
                ].map(([id, label, value]) => (
                  value ? (
                    <div key={id as string}>
                      <label className="block text-sm font-medium mb-2">{label}</label>
                      <div className="flex gap-2">
                        <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                          {value}
                        </code>
                        <button type="button"
                          onClick={() => copyToClipboard(String(value), String(id))}
                          className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                        >
                          {copied === id ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                        </button>
                      </div>
                    </div>
                  ) : null
                ))}
                <p className="text-xs text-gray-400">
                  {t("features.program-deploy.deployStats", {
                    writes: String(formData.writeCount || 0),
                    bytes: String(formData.programBytes || 0),
                    rent: String(formData.rentLamports || 0),
                  })}
                </p>
              </div>
            )}
          </div>
        );
      }

      case "program-info":
        return (
          <div className="space-y-4">
            {renderProgramIdInput()}
            <button type="button"
              onClick={() => handleSubmit("program-info")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.program-info.querying") : t("features.program-info.queryButton")}
            </button>
            {formData.programInfoChecked && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                {[
                  ["info-program-id", t("features.program-info.programId"), formData.programId],
                  ["info-programdata", t("features.program-info.programdataAddress"), formData.programdataAddress],
                  ["info-owner", t("features.program-info.owner"), formData.owner],
                ].map(([id, label, value]) => (
                  value ? (
                    <div key={id as string}>
                      <label className="block text-sm font-medium mb-2">{label}</label>
                      <div className="flex gap-2">
                        <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                          {value}
                        </code>
                        <button type="button"
                          onClick={() => copyToClipboard(String(value), String(id))}
                          className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                        >
                          {copied === id ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                        </button>
                      </div>
                    </div>
                  ) : null
                ))}
                <p className="text-xs text-gray-400">
                  {t("features.program-info.stats", {
                    exists: String(formData.programExists || "false"),
                    executable: String(formData.executable || "false"),
                    lamports: String(formData.lamports || 0),
                    dataLen: String(formData.dataLen || 0),
                  })}
                </p>
              </div>
            )}
          </div>
        );

      case "squads-create":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.squads-create.members")}</label>
              <textarea
                value={formData.members || ""}
                onChange={(e) => handleFormChange("members", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                placeholder={t("features.squads-create.membersPlaceholder")}
              />
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium mb-2">{t("features.squads-create.threshold")}</label>
                <input
                  type="text"
                  inputMode="numeric"
                  autoComplete="off"
                  value={formData.threshold || ""}
                  onChange={(e) => handleFormChange("threshold", e.target.value.replace(/[^\d]/g, ""))}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("features.squads-create.thresholdPlaceholder")}
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">{t("features.squads-create.timeLock")}</label>
                <input
                  type="text"
                  inputMode="numeric"
                  autoComplete="off"
                  value={formData.time_lock || ""}
                  onChange={(e) => handleFormChange("time_lock", e.target.value.replace(/[^\d]/g, ""))}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("features.squads-create.timeLockPlaceholder")}
                />
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.squads.memo")}</label>
              <input
                value={formData.memo || ""}
                onChange={(e) => handleFormChange("memo", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.squads.memoPlaceholder")}
              />
            </div>
            <button type="button" onClick={() => requestPasswordSubmit("squads-create")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
              {loading ? t("features.squads-create.creating") : t("features.squads-create.createButton")}
            </button>
            {renderSquadsResult()}
          </div>
        );

      case "squads-info":
        return (
          <div className="space-y-4">
            {renderMultisigInput()}
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.squads.proposalOptional")}</label>
              <input
                value={formData.proposal || ""}
                onChange={(e) => handleFormChange("proposal", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.squads.proposalPlaceholder")}
              />
            </div>
            <button type="button" onClick={() => handleSubmit("squads-info")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
              {loading ? t("features.squads-info.querying") : t("features.squads-info.queryButton")}
            </button>
            {(formData.vault || formData.membersText) && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                {renderCopyRow("squads-info-vault", t("features.squads.vault"), formData.vault)}
                {renderCopyRow("squads-info-create-key", t("features.squads.createKey"), formData.createKey)}
                <p className="text-xs text-gray-400">
                  {t("features.squads-info.stats", {
                    threshold: String(formData.threshold || ""),
                    transactionIndex: String(formData.transactionIndex || ""),
                  })}
                </p>
                {formData.membersText && (
                  <pre className="px-3 py-2 bg-black/30 rounded text-xs whitespace-pre-wrap break-all">{formData.membersText}</pre>
                )}
                {formData.proposalStatus && (
                  <p className="text-xs text-gray-300">
                    {t("features.squads-info.proposalStats", {
                      status: String(formData.proposalStatus),
                      transactionIndex: String(formData.proposalTxIndex || ""),
                    })}
                  </p>
                )}
                {formData.approvedText && (
                  <pre className="px-3 py-2 bg-black/30 rounded text-xs whitespace-pre-wrap break-all">{formData.approvedText}</pre>
                )}
              </div>
            )}
          </div>
        );

      case "squads-sol-transfer":
        return (
          renderStepForm([
            {
              title: t("features.workspace.stepTarget"),
              content: renderMultisigInput(),
            },
            {
              title: t("features.workspace.stepDetails"),
              content: (
                <>
                  {renderMultisigInput()}
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("formUi.recipientAddress")}</label>
                    <input
                      value={formData.to_address || ""}
                      onChange={(e) => handleFormChange("to_address", e.target.value)}
                      className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                      placeholder={t("formUi.placeholderRecipient")}
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("formUi.amountSol")}</label>
                    <input
                      type="text"
                      inputMode="decimal"
                      autoComplete="off"
                      value={formData.amount || ""}
                      onChange={(e) => handleFormChange("amount", e.target.value)}
                      className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                      placeholder={t("formUi.placeholderAmountSol")}
                    />
                  </div>
                  {renderMemoInput()}
                </>
              ),
            },
            {
              title: t("features.workspace.stepSubmit"),
              content: (
                <>
                  <button type="button" onClick={() => requestPasswordSubmit("squads-sol-transfer")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
                    {loading ? t("features.squads-sol-transfer.creating") : t("features.squads-sol-transfer.createButton")}
                  </button>
                  {renderSquadsResult()}
                </>
              ),
            },
          ])
        );

      case "squads-token-transfer":
        return (
          renderStepForm([
            {
              title: t("features.workspace.stepTarget"),
              content: (
                <>
                  {renderMultisigInput()}
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("features.transfer-token.mintAddress")}</label>
                    <input
                      value={formData.mint || ""}
                      onChange={(e) => handleFormChange("mint", e.target.value)}
                      className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                      placeholder={t("features.transfer-token.mintPlaceholder")}
                    />
                    {renderTokenMintInfo()}
                  </div>
                </>
              ),
            },
            {
              title: t("features.workspace.stepDetails"),
              content: (
                <>
                  {renderMultisigInput()}
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("features.transfer-token.mintAddress")}</label>
                    <input
                      value={formData.mint || ""}
                      onChange={(e) => handleFormChange("mint", e.target.value)}
                      className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                      placeholder={t("features.transfer-token.mintPlaceholder")}
                    />
                    {renderTokenMintInfo()}
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("formUi.recipientAddress")}</label>
                    <input
                      value={formData.recipient || ""}
                      onChange={(e) => handleFormChange("recipient", e.target.value)}
                      className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                      placeholder={t("formUi.placeholderRecipient")}
                    />
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <div>
                      <label className="block text-sm font-medium mb-2">{t("features.transfer-token.amount")}</label>
                      <input
                        type="text"
                        inputMode="decimal"
                        autoComplete="off"
                        value={formData.amount || ""}
                        onChange={(e) => handleFormChange("amount", e.target.value)}
                        className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                        placeholder={t("features.transfer-token.amountPlaceholder")}
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium mb-2">{t("features.transfer-token.decimals")}</label>
                      <input
                        value={formData.decimals ?? ""}
                        readOnly
                        className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg text-white/80"
                        placeholder={t("features.transfer-token.decimalsPlaceholder")}
                      />
                    </div>
                  </div>
                  {renderMemoInput()}
                </>
              ),
            },
            {
              title: t("features.workspace.stepSubmit"),
              content: (
                <>
                  <button type="button" onClick={() => requestPasswordSubmit("squads-token-transfer")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
                    {loading ? t("features.squads-token-transfer.creating") : t("features.squads-token-transfer.createButton")}
                  </button>
                  {renderSquadsResult()}
                </>
              ),
            },
          ])
        );

      case "squads-prepare-upgrade-buffer":
        return (
          renderStepForm([
            {
              title: t("features.workspace.stepTarget"),
              content: renderMultisigInput(),
            },
            {
              title: t("features.workspace.stepDetails"),
              content: (
                <>
                  {renderMultisigInput()}
                  {renderProgramFileInput()}
                </>
              ),
            },
            {
              title: t("features.workspace.stepSubmit"),
              content: (
                <>
                  <button type="button" onClick={() => requestPasswordSubmit("squads-prepare-upgrade-buffer")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
                    {loading ? t("features.squads-prepare-upgrade-buffer.preparing") : t("features.squads-prepare-upgrade-buffer.prepareButton")}
                  </button>
                  {renderSquadsResult()}
                </>
              ),
            },
          ])
        );

      case "squads-program-upgrade":
        return (
          renderStepForm([
            {
              title: t("features.workspace.stepTarget"),
              content: (
                <>
                  {renderMultisigInput()}
                  {renderProgramIdInput()}
                </>
              ),
            },
            {
              title: t("features.workspace.stepDetails"),
              content: (
                <>
                  {renderMultisigInput()}
                  {renderProgramIdInput()}
                  {renderBufferInput()}
                  {renderSpillInput()}
                  {renderMemoInput()}
                </>
              ),
            },
            {
              title: t("features.workspace.stepSubmit"),
              content: (
                <>
                  <button type="button" onClick={() => requestPasswordSubmit("squads-program-upgrade")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
                    {loading ? t("features.squads-program-upgrade.creating") : t("features.squads-program-upgrade.createButton")}
                  </button>
                  {renderSquadsResult()}
                </>
              ),
            },
          ])
        );

      case "squads-set-authority":
        return (
          <div className="space-y-4">
            {renderMultisigInput()}
            {renderProgramIdInput()}
            <button type="button" onClick={() => requestPasswordSubmit("squads-set-authority")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
              {loading ? t("features.squads-set-authority.setting") : t("features.squads-set-authority.setButton")}
            </button>
            {renderSquadsResult()}
          </div>
        );

      case "squads-approve":
      case "squads-reject":
        return (
          <div className="space-y-4">
            {renderMultisigInput()}
            {renderProposalInput()}
            {renderMemoInput()}
            <button type="button" onClick={() => requestPasswordSubmit(formId)} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
              {loading
                ? formId === "squads-approve"
                  ? t("features.squads-approve.approving")
                  : t("features.squads-reject.rejecting")
                : formId === "squads-approve"
                  ? t("features.squads-approve.approveButton")
                  : t("features.squads-reject.rejectButton")}
            </button>
            {renderSquadsResult()}
          </div>
        );

      case "squads-execute":
        return (
          <div className="space-y-4">
            {renderMultisigInput()}
            {renderProposalInput()}
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.squads.transactionIndex")}</label>
              <input
                type="text"
                inputMode="numeric"
                autoComplete="off"
                value={formData.transactionIndex || ""}
                onChange={(e) => handleFormChange("transactionIndex", e.target.value.replace(/[^\d]/g, ""))}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.squads.transactionIndexPlaceholder")}
              />
            </div>
            <button type="button" onClick={() => requestPasswordSubmit("squads-execute")} disabled={loading} className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50">
              {loading ? t("features.squads-execute.executing") : t("features.squads-execute.executeButton")}
            </button>
            {renderSquadsResult()}
          </div>
        );

      case "setup-2fa":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.setup-2fa.hardwareFingerprint")}</label>
              <input
                type="text"
                value={formData.hardware_fingerprint || ""}
                onChange={(e) => handleFormChange("hardware_fingerprint", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.setup-2fa.fingerprintPlaceholder")}
              />
              <p className="mt-1 text-xs text-gray-400">{t("features.setup-2fa.fingerprintHint")}</p>
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.setup-2fa.accountName")}</label>
              <input
                type="text"
                value={formData.account || ""}
                onChange={(e) => handleFormChange("account", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.setup-2fa.accountPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.setup-2fa.issuer")}</label>
              <input
                type="text"
                value={formData.issuer || ""}
                onChange={(e) => handleFormChange("issuer", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.setup-2fa.issuerPlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("setup-2fa")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.setup-2fa.generating") : t("features.setup-2fa.generateButton")}
            </button>
            {formData.totp_secret && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.setup-2fa.totpSecret")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.totp_secret}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.totp_secret as string, "totp")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "totp" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.setup-2fa.qrCodeUrl")}</label>
                  <code className="block px-3 py-2 bg-black/30 rounded text-xs break-all">
                    {formData.qr_code_url}
                  </code>
                </div>
              </div>
            )}
          </div>
        );

      case "create-tfa": {
        const tfaAuth = authMethod["create-tfa"] ?? "keystore";
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "create-tfa": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    tfaAuth === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "create-tfa": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    tfaAuth === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "create-tfa": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    tfaAuth === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {tfaAuth === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("features.create-tfa.uploadFile")}</label>
                  <input
                    type="file"
                    accept=".json,application/json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && tfaAuth === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && tfaAuth === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("features.create-tfa.privateKeyLabel")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("features.create-tfa.privateKeyPlaceholder")}
                />
              </div>
            )}
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.create-tfa.totpSecret")}</label>
              <input
                type="text"
                value={formData.totp_secret || ""}
                onChange={(e) => handleFormChange("totp_secret", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.create-tfa.totpSecretPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.create-tfa.hardwareFingerprint")}</label>
              <input
                type="text"
                value={formData.hardware_fingerprint || ""}
                onChange={(e) => handleFormChange("hardware_fingerprint", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.create-tfa.fingerprintPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.create-tfa.questionIndex")}</label>
              <input
                type="text"
                inputMode="numeric"
                autoComplete="off"
                value={formData.question_index === undefined ? "" : formData.question_index}
                onChange={(e) => {
                  const raw = e.target.value.replace(/[^\d]/g, "");
                  if (raw === "") {
                    handleFormChange("question_index", undefined);
                    return;
                  }
                  const n = parseInt(raw, 10);
                  handleFormChange("question_index", Number.isNaN(n) ? undefined : n);
                }}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.create-tfa.questionIndexPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.create-tfa.securityAnswer")}</label>
              <input
                type="password"
                value={formData.security_answer || ""}
                onChange={(e) => handleFormChange("security_answer", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.create-tfa.answerPlaceholder")}
                autoComplete="off"
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("create-tfa")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.create-tfa.creating") : t("features.create-tfa.createButton")}
            </button>
            {formData.encrypted_wallet && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.publicKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.publicKey as string, "tfa-pubkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "tfa-pubkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-tfa.encryptedWallet")}</label>
                  <p className="text-xs text-gray-400">{t("features.create-tfa.saveHint")}</p>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all max-h-32 overflow-y-auto">
                      {formData.encrypted_wallet}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.encrypted_wallet as string, "tfa-enc")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "tfa-enc" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );
      }

      case "unlock-tfa":
        return (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.unlock-tfa.encryptedWallet")}</label>
              <textarea
                value={formData.encrypted_wallet || ""}
                onChange={(e) => handleFormChange("encrypted_wallet", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[100px]"
                placeholder={t("features.unlock-tfa.walletPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.unlock-tfa.hardwareFingerprint")}</label>
              <input
                type="text"
                value={formData.hardware_fingerprint || ""}
                onChange={(e) => handleFormChange("hardware_fingerprint", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.unlock-tfa.fingerprintPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.unlock-tfa.securityAnswer")}</label>
              <input
                type="password"
                value={formData.security_answer || ""}
                onChange={(e) => handleFormChange("security_answer", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.unlock-tfa.answerPlaceholder")}
                autoComplete="off"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.unlock-tfa.totpCode")}</label>
              <input
                type="password"
                value={formData.totp_code || ""}
                onChange={(e) => handleFormChange("totp_code", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.unlock-tfa.totpPlaceholder")}
                maxLength={6}
                inputMode="numeric"
                autoComplete="one-time-code"
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("unlock-tfa")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.unlock-tfa.unlocking") : t("features.unlock-tfa.unlockButton")}
            </button>
            {formData.unlocked && formData.publicKey && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("features.create-plain.publicKey")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.publicKey}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.publicKey as string, "unlock-pubkey")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "unlock-pubkey" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
                <p className="text-sm text-green-400">{t("formUi.walletUnlockedNoSecret")}</p>
              </div>
            )}
          </div>
        );

      case "close-wsol-ata":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "close-wsol-ata": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("close-wsol-ata") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "close-wsol-ata": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("close-wsol-ata") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "close-wsol-ata": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("close-wsol-ata") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("close-wsol-ata") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("close-wsol-ata") === "encrypted" && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                  <textarea
                    value={formData.encrypted_key || ""}
                    onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                    placeholder={t("formUi.placeholderEncryptedKey")}
                  />
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("close-wsol-ata") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}
            <button type="button"
              onClick={() => requestPasswordSubmit("close-wsol-ata")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.close-wsol-ata.closing") : t("features.close-wsol-ata.closeButton")}
            </button>
            {formData.signature && (
              <div className="space-y-3 p-4 bg-white/5 rounded-lg">
                <div>
                  <label className="block text-sm font-medium mb-2">{t("formUi.txSignature")}</label>
                  <div className="flex gap-2">
                    <code className="flex-1 px-3 py-2 bg-black/30 rounded text-xs break-all">
                      {formData.signature}
                    </code>
                    <button type="button"
                      onClick={() => copyToClipboard(formData.signature as string, "close-ata")}
                      className="px-3 py-2 bg-white/10 rounded hover:bg-white/20 transition-colors"
                    >
                      {copied === "close-ata" ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        );

      case "pumpfun-sell":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "pumpfun-sell": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("pumpfun-sell") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "pumpfun-sell": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("pumpfun-sell") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "pumpfun-sell": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("pumpfun-sell") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("pumpfun-sell") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("pumpfun-sell") === "encrypted" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                <textarea
                  value={formData.encrypted_key || ""}
                  onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                  placeholder={t("formUi.placeholderEncryptedKey")}
                />
              </div>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("pumpfun-sell") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("features.pumpfun-sell.privateKeyLabel")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("features.pumpfun-sell.privateKeyPlaceholder")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium mb-2">{t("features.pumpfun-sell.mintAddress")}</label>
              <input
                type="text"
                value={formData.mint || ""}
                onChange={(e) => handleFormChange("mint", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.pumpfun-sell.mintPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.pumpfun-sell.amount")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={formData.amount ?? ""}
                onChange={(e) => handleFormChange("amount", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.pumpfun-sell.amountPlaceholder")}
              />
              <p className="mt-2 text-xs text-gray-400">{t("features.pumpfun-sell.sellPercent")}</p>
              <div className="mt-2 grid grid-cols-4 gap-2">
                {[25, 50, 75, 100].map((percent) => (
                  <button
                    key={percent}
                    type="button"
                    onClick={() => void handleSellPercentShortcut("pumpfun-sell", percent)}
                    className="rounded-lg bg-white/10 px-3 py-2 text-sm font-medium text-gray-200 transition-colors hover:bg-white/20"
                  >
                    {percent}%
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.pumpfun-sell.slippage")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={slippageInputDisplay(formData.slippage)}
                onChange={(e) => {
                  const raw = e.target.value;
                  if (raw === "") {
                    handleFormChange("slippage", undefined);
                    return;
                  }
                  const n = parseFloat(raw);
                  handleFormChange("slippage", Number.isNaN(n) ? undefined : n);
                }}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.pumpfun-sell.slippagePlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("pumpfun-sell")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.pumpfun-sell.selling") : t("features.pumpfun-sell.sellButton")}
            </button>
            {formData.status === "success" && (
              <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <p className="text-green-400">{t("formUi.sellSuccess")}</p>
              </div>
            )}
          </div>
        );

      case "pumpswap-sell":
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "pumpswap-sell": "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("pumpswap-sell") === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "pumpswap-sell": "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("pumpswap-sell") === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, "pumpswap-sell": "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    walletAuth("pumpswap-sell") === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {walletAuth("pumpswap-sell") === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("pumpswap-sell") === "encrypted" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                <textarea
                  value={formData.encrypted_key || ""}
                  onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                  placeholder={t("formUi.placeholderEncryptedKey")}
                />
              </div>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && walletAuth("pumpswap-sell") === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("features.pumpswap-sell.privateKeyLabel")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("features.pumpswap-sell.privateKeyPlaceholder")}
                />
                <p className="mt-1 text-xs text-yellow-400">{t("formUi.warnPlaintextStrong")}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium mb-2">{t("features.pumpswap-sell.mintAddress")}</label>
              <input
                type="text"
                value={formData.mint || ""}
                onChange={(e) => handleFormChange("mint", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.pumpswap-sell.mintPlaceholder")}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.pumpswap-sell.amount")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={formData.amount ?? ""}
                onChange={(e) => handleFormChange("amount", e.target.value)}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.pumpswap-sell.amountPlaceholder")}
              />
              <p className="mt-2 text-xs text-gray-400">{t("features.pumpswap-sell.sellPercent")}</p>
              <div className="mt-2 grid grid-cols-4 gap-2">
                {[25, 50, 75, 100].map((percent) => (
                  <button
                    key={percent}
                    type="button"
                    onClick={() => void handleSellPercentShortcut("pumpswap-sell", percent)}
                    className="rounded-lg bg-white/10 px-3 py-2 text-sm font-medium text-gray-200 transition-colors hover:bg-white/20"
                  >
                    {percent}%
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium mb-2">{t("features.pumpswap-sell.slippage")}</label>
              <input
                type="text"
                inputMode="decimal"
                autoComplete="off"
                value={slippageInputDisplay(formData.slippage)}
                onChange={(e) => {
                  const raw = e.target.value;
                  if (raw === "") {
                    handleFormChange("slippage", undefined);
                    return;
                  }
                  const n = parseFloat(raw);
                  handleFormChange("slippage", Number.isNaN(n) ? undefined : n);
                }}
                className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                placeholder={t("features.pumpswap-sell.slippagePlaceholder")}
              />
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit("pumpswap-sell")}
              disabled={loading}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading ? t("features.pumpswap-sell.selling") : t("features.pumpswap-sell.sellButton")}
            </button>
            {formData.status === "success" && (
              <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                <p className="text-green-400">{t("formUi.sellSuccess")}</p>
              </div>
            )}
          </div>
        );

      case "pumpfun-cashback":
      case "pumpswap-cashback": {
        const cashbackAuth = walletAuth(selectedForm || "pumpfun-cashback");
        const cashbackDex = selectedForm === "pumpswap-cashback" ? "pumpswap" : "pumpfun";
        const currentCashbackInfo = cashbackInfo
          ? cashbackInfo.owner === effectiveWallet?.public_key &&
            cashbackInfo.network === effectiveNetwork &&
            cashbackInfo.dex === cashbackDex
            ? cashbackInfo
            : null
          : null;
        return (
          <div className="space-y-4">
            <div className={ALLOW_DIRECT_SECRET_INPUT ? undefined : "hidden"}>
              <label className="block text-sm font-medium mb-2">{t("formUi.authMethod")}</label>
              <div className={ALLOW_DIRECT_SECRET_INPUT ? "grid grid-cols-3 gap-2" : "grid grid-cols-1 gap-2 [&>button:not(:first-child)]:hidden"}>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, [selectedForm || "pumpfun-cashback"]: "keystore" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.encrypted_key;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    cashbackAuth === "keystore"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabKeystore")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, [selectedForm || "pumpfun-cashback"]: "encrypted" });
                    const newFormData = { ...formData };
                    delete newFormData.private_key;
                    delete newFormData.keystoreJson;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    cashbackAuth === "encrypted"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabEncrypted")}
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setAuthMethod({ ...authMethod, [selectedForm || "pumpfun-cashback"]: "private" });
                    const newFormData = { ...formData };
                    delete newFormData.keystoreJson;
                    delete newFormData.encrypted_key;
                    delete newFormData.password;
                    setFormData(newFormData);
                  }}
                  className={`py-2 px-3 rounded-lg font-medium transition-colors text-sm ${
                    cashbackAuth === "private"
                      ? "bg-gradient-to-r from-purple-500 to-pink-500 text-white"
                      : "bg-white/5 text-gray-400 hover:bg-white/10"
                  }`}
                >
                  {t("formUi.tabPrivateKey")}
                </button>
              </div>
            </div>

            {cashbackAuth === "keystore" && (
              <>
                <div className={formData.wallet_id ? "hidden" : undefined}>
                  <label className="block text-sm font-medium mb-2">{t("formUi.uploadKeystore")}</label>
                  <input
                    type="file"
                    accept=".json,application/json"
                    onChange={handleFileUpload}
                    className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:bg-white/10 file:text-white hover:file:bg-white/20"
                  />
                  {formData.keystoreJson && (
                    <p className="mt-2 text-xs text-green-400">{t("formUi.fileUploadedOk")}</p>
                  )}
                </div>
              </>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && cashbackAuth === "encrypted" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.encryptedKey")}</label>
                <textarea
                  value={formData.encrypted_key || ""}
                  onChange={(e) => handleFormChange("encrypted_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white min-h-[120px]"
                  placeholder={t("formUi.placeholderEncryptedKey")}
                />
              </div>
            )}

            {ALLOW_DIRECT_SECRET_INPUT && cashbackAuth === "private" && (
              <div>
                <label className="block text-sm font-medium mb-2">{t("formUi.privateKey")}</label>
                <input
                  type="password"
                  value={formData.private_key || ""}
                  onChange={(e) => handleFormChange("private_key", e.target.value)}
                  className="w-full px-4 py-2 bg-white/5 border border-white/10 rounded-lg focus:outline-none focus:ring-2 focus:ring-white/20 text-white"
                  placeholder={t("formUi.placeholderPrivateKeyBase58")}
                />
              </div>
            )}
            <div className="rounded-xl border border-white/10 bg-white/5 p-4">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-sm text-gray-400">
                    {selectedForm === "pumpswap-cashback"
                      ? t("features.pumpswap-cashback.availableAmount")
                      : t("features.pumpfun-cashback.availableAmount")}
                  </p>
                  <p className="mt-2 text-3xl font-semibold">
                    {currentCashbackInfo?.loading
                      ? "--"
                      : `${currentCashbackInfo?.uiAmountString ?? "0"} ${currentCashbackInfo?.asset ?? (selectedForm === "pumpswap-cashback" ? "WSOL" : "SOL")}`}
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => void loadCashbackInfo(cashbackDex, effectiveWallet)}
                  className="inline-flex items-center gap-2 rounded-lg bg-white/10 px-3 py-2 text-xs hover:bg-white/20"
                >
                  <RefreshCw className={`h-3.5 w-3.5 ${currentCashbackInfo?.loading ? "animate-spin" : ""}`} />
                  {selectedForm === "pumpswap-cashback"
                    ? t("features.pumpswap-cashback.refreshAmount")
                    : t("features.pumpfun-cashback.refreshAmount")}
                </button>
              </div>
              {currentCashbackInfo?.accumulator && (
                <button
                  type="button"
                  onClick={() => copyToClipboard(currentCashbackInfo.accumulator || "", `cashback-accumulator:${cashbackDex}`)}
                  className="mt-3 block max-w-full truncate text-left font-mono text-xs text-gray-500 hover:text-white"
                >
                  {currentCashbackInfo.accumulator}
                </button>
              )}
              {currentCashbackInfo?.error && (
                <p className="mt-3 text-xs text-yellow-200">{currentCashbackInfo.error}</p>
              )}
            </div>
            <button type="button"
              onClick={() => requestPasswordSubmit(selectedForm || "")}
              disabled={loading || currentCashbackInfo?.loading || currentCashbackInfo?.available !== true}
              className="w-full py-3 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg font-semibold hover:from-purple-600 hover:to-pink-600 transition-all disabled:opacity-50"
            >
              {loading
                ? selectedForm === "pumpswap-cashback"
                  ? t("features.pumpswap-cashback.claiming")
                  : t("features.pumpfun-cashback.claiming")
                : selectedForm === "pumpswap-cashback"
                  ? t("features.pumpswap-cashback.claimButton")
                  : t("features.pumpfun-cashback.claimButton")}
            </button>
            {formData.message && (
              <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg">
                <p className="text-blue-400 text-sm">{formData.message}</p>
                {formData.signature && (
                  <button
                    type="button"
                    onClick={() => copyToClipboard(String(formData.signature), "cashback-signature")}
                    className="mt-2 block max-w-full truncate text-left font-mono text-xs text-blue-200 hover:text-white"
                  >
                    {formData.signature}
                  </button>
                )}
              </div>
            )}
          </div>
        );
      }

      default:
        return <div className="text-center text-gray-400">{t("formUi.pickFeature")}</div>;
      }
    };

    const internalSquadsForms = new Set([
      "squads-create",
      "squads-info",
      "squads-sol-transfer",
      "squads-token-transfer",
      "squads-prepare-upgrade-buffer",
      "squads-program-upgrade",
      "squads-set-authority",
    ]);
    const showSquadsWalletPicker = ALLOW_DIRECT_SECRET_INPUT && internalSquadsForms.has(formId);
    const showTokenActionHeader = tokenActionContext && TOKEN_ACTION_FORM_IDS.has(formId);
    const tokenActionMint = String((tokenActionContext?.mint ?? formData.mint) || "");
    const tokenActionBalance = String((tokenActionContext?.token_balance ?? formData.token_balance ?? formData.amount) || "0");
    const tokenActionDecimals = tokenActionContext?.decimals ?? formData.decimals;
    const transferSolWallet = selectedSavedWallet() ?? effectiveWallet;
    const transferSolAssets =
      transferSolWallet &&
      walletAssets?.address === transferSolWallet.public_key &&
      walletAssets.network === effectiveNetwork
        ? walletAssets
        : null;
    const transferSolBalance =
      transferSolAssets?.solBalance && transferSolAssets.solBalance !== "--"
        ? transferSolAssets.solBalance
        : "--";
    const transferTokenMint = String(formData.mint ?? "").trim();
    const transferTokenWallet = selectedSavedWallet() ?? effectiveWallet;
    const transferTokenAssets =
      transferTokenWallet &&
      walletAssets?.address === transferTokenWallet.public_key &&
      walletAssets.network === effectiveNetwork
        ? walletAssets
        : null;
    const transferTokenBalance =
      transferTokenMint && transferTokenAssets
        ? aggregateTokenBalance(transferTokenAssets.tokens, transferTokenMint)
        : null;

    return (
      <div className="space-y-4">
        {showTokenActionHeader && (
          <div className="space-y-4 rounded-2xl border border-white/10 bg-white/5 p-4">
            <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <div className="min-w-0">
                <p className="text-xs font-semibold uppercase tracking-normal text-gray-500">
                  {t("features.token-actions.token")}
                </p>
                <button
                  type="button"
                  onClick={() => copyToClipboard(tokenActionMint, "token-actions-mint")}
                  className="mt-1 block max-w-full truncate text-left font-mono text-sm text-gray-200 hover:text-white"
                >
                  {tokenActionMint}
                </button>
              </div>
              <div className="grid grid-cols-2 gap-2 sm:min-w-56">
                <div className="rounded-lg bg-black/20 px-3 py-2">
                  <div className="flex items-center justify-between gap-2">
                    <p className="text-xs text-gray-500">{t("features.token-actions.balance")}</p>
                    <button
                      type="button"
                      onClick={() => refreshCurrentWalletAssets()}
                      className="inline-flex h-6 w-6 items-center justify-center rounded-md bg-white/10 text-gray-300 hover:bg-white/20 hover:text-white"
                      title={t("features.wallet-list.refreshAssets")}
                      aria-label={t("features.wallet-list.refreshAssets")}
                    >
                      <RefreshCw className={`h-3.5 w-3.5 ${walletAssets?.refreshing ? "animate-spin" : ""}`} />
                    </button>
                  </div>
                  <p className="mt-1 truncate text-sm font-semibold">{tokenActionBalance}</p>
                </div>
                <div className="rounded-lg bg-black/20 px-3 py-2">
                  <p className="text-xs text-gray-500">{t("features.token-actions.decimals")}</p>
                  <p className="mt-1 text-sm font-semibold">{String(tokenActionDecimals ?? "")}</p>
                </div>
              </div>
            </div>
            <div className="grid grid-cols-2 gap-2 rounded-xl border border-white/10 bg-black/20 p-1">
              {[
                { id: "pumpfun-sell" as const, label: t("features.token-actions.sell") },
                { id: "transfer-token" as const, label: t("features.token-actions.transfer") },
              ].map((tab) => (
                <button
                  key={tab.id}
                  type="button"
                  onClick={() => switchTokenAction(tab.id)}
                  className={`h-10 rounded-lg text-sm font-semibold transition-colors ${
                    formId === tab.id
                      ? "bg-white text-black"
                      : "text-gray-300 hover:bg-white/10 hover:text-white"
                  }`}
                >
                  {tab.label}
                </button>
              ))}
            </div>
          </div>
        )}
        {showSquadsWalletPicker && (
          <SavedWalletPicker
            copied={copied}
            formData={formData}
            formId={formId}
            loading={walletsLoading}
            t={t}
            walletAuth={walletAuth(formId)}
            wallets={wallets}
            showTemporaryKeystore
            onCopy={copyToClipboard}
            onFieldChange={handleFormChange}
            onKeystoreUpload={handleFileUpload}
            onRefresh={() => void loadWallets()}
          />
        )}
        {renderFormBody()}
      </div>
    );
  };

  const selectedFormTitle = selectedForm
    ? tokenActionContext && TOKEN_ACTION_FORM_IDS.has(selectedForm)
      ? t("features.token-actions.title")
      : menuItems
        .flatMap((m) => m.children || [])
        .find((c) => c?.id === selectedForm)?.label ||
      ({
        "wallet-list": t("features.wallet-list.title"),
        "wsol-workbench": t("features.wsol-workbench.title"),
        "pump-workbench": t("features.pump-workbench.title"),
        "program-workbench": t("features.program-workbench.title"),
        "nonce-workbench": t("features.nonce-workbench.title"),
        "settings": t("features.settings.title"),
        "create-encrypted": t("features.create-encrypted.title"),
        "create-keystore": t("features.create-keystore.title"),
        "import-keystore": t("features.import-keystore.title"),
        "decrypt": t("features.decrypt.title"),
        "setup-2fa": t("features.setup-2fa.title"),
        "create-tfa": t("features.create-tfa.title"),
        "unlock-tfa": t("features.unlock-tfa.title"),
        "unlock": t("features.unlock.title"),
        "check-balance": t("features.check-balance.title"),
        "get-pubkey": t("features.get-pubkey.title"),
        "transfer-sol": t("features.transfer-sol.title"),
        "transfer-token": t("features.transfer-token.title"),
        "create-wsol-ata": t("features.create-wsol-ata.title"),
        "wrap-sol": t("features.wrap-sol.title"),
        "unwrap-sol": t("features.unwrap-sol.title"),
        "close-wsol-ata": t("features.close-wsol-ata.title"),
        "pumpfun-sell": t("features.pumpfun-sell.title"),
        "pumpswap-sell": t("features.pumpswap-sell.title"),
        "pumpfun-cashback": t("features.pumpfun-cashback.title"),
        "pumpswap-cashback": t("features.pumpswap-cashback.title"),
        "create-nonce": t("features.create-nonce.title"),
        "program-deploy": t("features.program-deploy.title"),
        "program-info": t("features.program-info.title"),
        "squads-workspace": t("features.workspace.title"),
        "squads-proposals": t("features.workspace.savedProposals"),
        "squads-programs": t("features.workspace.savedPrograms"),
        "squads-create": t("features.squads-create.title"),
        "squads-info": t("features.squads-info.title"),
        "squads-sol-transfer": t("features.squads-sol-transfer.title"),
        "squads-token-transfer": t("features.squads-token-transfer.title"),
        "squads-prepare-upgrade-buffer": t("features.squads-prepare-upgrade-buffer.title"),
        "squads-program-upgrade": t("features.squads-program-upgrade.title"),
        "squads-set-authority": t("features.squads-set-authority.title"),
      } as Record<string, string>)[selectedForm] ||
      t("formUi.pickFeature")
    : t("app.welcome");
  const showFormHeader = selectedForm !== "wallet-list";

  return (
    <div className="flex min-h-screen flex-col bg-black text-white lg:h-screen lg:flex-row">
      {/* Left Sidebar */}
      <div className="w-full bg-black/40 backdrop-blur-xl border-b border-white/10 flex flex-col lg:h-screen lg:w-80 lg:border-b-0 lg:border-r">
        <div className="p-3 border-b border-white/10 flex items-center justify-between gap-3 lg:p-6">
          <div className="min-w-0">
            <h1 className="text-xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent lg:text-2xl">
              {t("app.title")}
            </h1>
            <p className="hidden text-sm text-gray-400 mt-1 sm:block">{t("app.subtitle")}</p>
            <p className="mt-1 truncate text-xs text-gray-400 lg:hidden">{selectedFormTitle}</p>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <LanguageSwitcher />
            <button
              type="button"
              onClick={() => setMobileMenuOpen((open) => !open)}
              className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-white/10 bg-white/5 text-gray-200 hover:bg-white/10 lg:hidden"
              aria-label={t("formUi.pickFeature")}
            >
              {mobileMenuOpen ? <X className="h-4 w-4" /> : <Menu className="h-4 w-4" />}
            </button>
          </div>
        </div>

        <nav className="hidden flex-1 space-y-2 overflow-y-auto p-4 scrollbar-thin lg:flex lg:flex-col">
          {menuItems.map((item) => (
            <div
              key={item.id}
              className="min-w-0"
            >
              <button type="button"
                onClick={() => item.children ? toggleMenu(item.id) : handleSelectForm(item.id)}
                className={`w-full flex items-center justify-between px-4 py-3 rounded-lg transition-colors ${
                  selectedForm === item.id
                    ? "bg-gradient-to-r from-purple-500/20 to-pink-500/20 text-white"
                    : "hover:bg-white/5 text-gray-300"
                }`}
              >
                <div className="flex min-w-0 items-center gap-3">
                  {item.icon}
                  <span className="truncate font-medium">{item.label}</span>
                </div>
                {item.children && (
                  activeMenu === item.id ? (
                    <ChevronDown className="w-4 h-4" />
                  ) : (
                    <ChevronRight className="w-4 h-4" />
                  )
                )}
              </button>

              {item.children && activeMenu === item.id && (
                <div className="ml-4 mt-2 space-y-1">
                  {item.children.map((child) => (
                    <button type="button"
                      key={child.id}
                      onClick={() => handleSelectForm(child.id)}
                      className={`w-full flex items-center gap-3 px-4 py-2 rounded-lg transition-colors ${
                        selectedForm === child.id
                          ? "bg-gradient-to-r from-purple-500/20 to-pink-500/20 text-white"
                          : "hover:bg-white/5 text-gray-400"
                      }`}
                    >
                      {child.icon}
                      <span className="truncate text-sm">{child.label}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          ))}
        </nav>
      </div>

      {mobileMenuOpen && (
        <button
          type="button"
          aria-label={t("common.cancel")}
          onClick={() => setMobileMenuOpen(false)}
          className="fixed inset-0 z-[110] bg-black/60 backdrop-blur-sm lg:hidden"
        />
      )}

      <aside
        className={`fixed inset-y-0 left-0 z-[120] flex w-80 max-w-[86vw] flex-col border-r border-white/10 bg-zinc-950 shadow-2xl transition-transform duration-200 lg:hidden ${
          mobileMenuOpen ? "translate-x-0" : "-translate-x-full"
        }`}
      >
        <div className="flex items-center justify-between gap-3 border-b border-white/10 px-4 py-4">
          <div className="min-w-0">
            <p className="truncate text-lg font-semibold">{t("app.title")}</p>
            <p className="mt-1 truncate text-sm text-gray-400">{selectedFormTitle}</p>
          </div>
          <button
            type="button"
            onClick={() => setMobileMenuOpen(false)}
            className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white/10 text-gray-200 hover:bg-white/20"
            aria-label={t("common.cancel")}
          >
            <X className="h-5 w-5" />
          </button>
        </div>
        <nav className="flex-1 space-y-2 overflow-y-auto p-3 scrollbar-thin">
          {menuItems.map((item) => (
            <div key={item.id} className="min-w-0">
              <button
                type="button"
                onClick={() => item.children ? toggleMenu(item.id) : handleSelectForm(item.id)}
                className={`flex w-full items-center justify-between rounded-xl px-4 py-3 text-left transition-colors ${
                  selectedForm === item.id
                    ? "bg-gradient-to-r from-purple-500/20 to-pink-500/20 text-white"
                    : "text-gray-300 hover:bg-white/5"
                }`}
              >
                <span className="flex min-w-0 items-center gap-3">
                  {item.icon}
                  <span className="truncate text-base font-medium">{item.label}</span>
                </span>
                {item.children && (
                  activeMenu === item.id ? (
                    <ChevronDown className="h-4 w-4 shrink-0" />
                  ) : (
                    <ChevronRight className="h-4 w-4 shrink-0" />
                  )
                )}
              </button>
              {item.children && activeMenu === item.id && (
                <div className="ml-4 mt-2 space-y-1 border-l border-white/10 pl-2">
                  {item.children.map((child) => (
                    <button
                      type="button"
                      key={child.id}
                      onClick={() => handleSelectForm(child.id)}
                      className={`flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-colors ${
                        selectedForm === child.id
                          ? "bg-gradient-to-r from-purple-500/20 to-pink-500/20 text-white"
                          : "text-gray-400 hover:bg-white/5 hover:text-gray-200"
                      }`}
                    >
                      {child.icon}
                      <span className="truncate text-sm">{child.label}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          ))}
        </nav>
      </aside>

      {/* Main Content */}
      <div className="min-w-0 flex-1 overflow-y-auto bg-gradient-to-br from-black via-purple-950 to-black">
        <div className="min-h-full overflow-y-auto">
          <div className="max-w-5xl mx-auto p-3 space-y-3 sm:p-4 lg:p-8 lg:space-y-4">
            <div className="bg-black/40 backdrop-blur-xl rounded-xl border border-white/10 p-3 sm:p-4 lg:rounded-2xl lg:p-6">
              {showFormHeader && (
                <div className="mb-5 flex flex-col gap-3 sm:flex-row sm:items-center">
                  {selectedForm && (backTarget || defaultBackTarget(selectedForm)) && (
                    <button
                      type="button"
                      onClick={handleBack}
                      className="inline-flex w-fit items-center gap-2 rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-gray-300 hover:bg-white/10 hover:text-white"
                    >
                      <ArrowLeft className="h-4 w-4" />
                      {t("common.back")}
                    </button>
                  )}
                  <h2 className="text-2xl font-bold">
                    {selectedFormTitle}
                  </h2>
                </div>
              )}
              {selectedForm ? renderForm(selectedForm) : (
                <div className="text-center py-12 text-gray-400">
                  <Key className="w-16 h-16 mx-auto mb-4 opacity-50" />
                  <p className="text-lg">{t("app.selectFeature")}</p>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
      {passwordPrompt && (
        <div className="fixed inset-0 z-[200] flex items-end bg-black/60">
          <button
            type="button"
            aria-label={t("common.cancel")}
            className="absolute inset-0 cursor-default"
            onClick={closePasswordPrompt}
          />
          <div className="relative w-full border-t border-white/10 bg-zinc-950 px-4 py-5 shadow-2xl">
            <div className="mx-auto max-w-xl space-y-4">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <h3 className="text-lg font-semibold">{passwordPromptTitle}</h3>
                  <p className="mt-1 text-sm text-gray-400">{passwordPromptHint}</p>
                </div>
                <button
                  type="button"
                  onClick={closePasswordPrompt}
                  disabled={loading}
                  className="rounded-lg bg-white/10 p-2 text-gray-300 hover:bg-white/20 disabled:opacity-50"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
              <div className="space-y-3">
                {showWalletPasswordPrompt && (
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("formUi.walletPassword")}</label>
                    <input
                      autoFocus
                      type="password"
                      value={passwordPromptValue}
                      onChange={(e) => setPasswordPromptValue(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          e.preventDefault();
                          void confirmPasswordPrompt();
                        }
                      }}
                      className="w-full rounded-lg border border-white/10 bg-black/40 px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-white/20"
                      placeholder={t("formUi.placeholderKeystorePassword")}
                    />
                  </div>
                )}
                {showMasterPasswordPrompt && (
                  <div>
                    <label className="block text-sm font-medium mb-2">{t("formUi.masterPassword")}</label>
                    <input
                      autoFocus={!showWalletPasswordPrompt}
                      type="password"
                      value={masterPasswordPromptValue}
                      onChange={(e) => setMasterPasswordPromptValue(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          e.preventDefault();
                          void confirmPasswordPrompt();
                        }
                      }}
                      className="w-full rounded-lg border border-white/10 bg-black/40 px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-white/20"
                      placeholder={t("formUi.masterPasswordPlaceholder")}
                    />
                  </div>
                )}
              </div>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={closePasswordPrompt}
                  disabled={loading}
                  className="rounded-lg bg-white/10 px-4 py-3 text-sm font-semibold hover:bg-white/20 disabled:opacity-50"
                >
                  {t("common.cancel")}
                </button>
                <button
                  type="button"
                  onClick={() => void confirmPasswordPrompt()}
                  disabled={loading}
                  className="rounded-lg bg-gradient-to-r from-purple-500 to-pink-500 px-4 py-3 text-sm font-semibold hover:from-purple-600 hover:to-pink-600 disabled:opacity-50"
                >
                  {loading ? t("common.processing") : passwordPromptButton}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
