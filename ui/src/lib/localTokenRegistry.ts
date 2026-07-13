export interface LocalTokenMetadata {
  mint: string;
  symbol: string;
  name: string;
  logoUri: string;
}

export const LOCAL_TOKEN_METADATA: LocalTokenMetadata[] = [
  {
    mint: "So11111111111111111111111111111111111111112",
    symbol: "WSOL",
    name: "Wrapped SOL",
    logoUri: "/token-icons/solana.png",
  },
  {
    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    symbol: "USDC",
    name: "USD Coin",
    logoUri: "/token-icons/usdc.png",
  },
  {
    mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
    symbol: "USDT",
    name: "USDT",
    logoUri: "/token-icons/usdt.png",
  },
  {
    mint: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN",
    symbol: "JUP",
    name: "Jupiter",
    logoUri: "/token-icons/jup.png",
  },
  {
    mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
    symbol: "JitoSOL",
    name: "Jito Staked SOL",
    logoUri: "/token-icons/jitosol.png",
  },
  {
    mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
    symbol: "mSOL",
    name: "Marinade staked SOL",
    logoUri: "/token-icons/msol.png",
  },
  {
    mint: "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
    symbol: "BONK",
    name: "Bonk",
    logoUri: "/token-icons/bonk.jpg",
  },
  {
    mint: "HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3",
    symbol: "PYTH",
    name: "Pyth Network",
    logoUri: "/token-icons/pyth.png",
  },
  {
    mint: "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
    symbol: "WIF",
    name: "dogwifhat",
    logoUri: "/token-icons/wif.jpg",
  },
  {
    mint: "jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL",
    symbol: "JTO",
    name: "Jito",
    logoUri: "/token-icons/jto.webp",
  },
];

export const LOCAL_TOKEN_METADATA_BY_MINT = new Map(
  LOCAL_TOKEN_METADATA.map((token) => [token.mint, token]),
);

export function localTokenMetadata(mint: string | null | undefined): LocalTokenMetadata | undefined {
  return mint ? LOCAL_TOKEN_METADATA_BY_MINT.get(mint.trim()) : undefined;
}
