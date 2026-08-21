enum AppNetwork { mainnet, devnet, testnet }

extension AppNetworkLabel on AppNetwork {
  String get label => switch (this) {
        AppNetwork.mainnet => 'Mainnet',
        AppNetwork.devnet => 'Devnet',
        AppNetwork.testnet => 'Testnet',
      };
}

class WalletSummary {
  const WalletSummary({
    required this.id,
    required this.name,
    required this.publicKey,
  });

  factory WalletSummary.fromJson(Map<String, Object?> json) {
    return WalletSummary(
      id: json['id'] as String,
      name: json['name'] as String,
      publicKey: json['publicKey'] as String,
    );
  }

  final String id;
  final String name;
  final String publicKey;

  Map<String, Object?> toJson() => {
        'id': id,
        'name': name,
        'publicKey': publicKey,
      };
}

class WalletKeystore {
  const WalletKeystore({
    required this.wallet,
    required this.keystoreJson,
  });

  final WalletSummary wallet;
  final String keystoreJson;
}

class AssetSnapshot {
  const AssetSnapshot({
    required this.network,
    required this.walletPublicKey,
    required this.solBalanceLamports,
    required this.tokens,
    this.recentTransactions = const [],
    this.refreshedAtMs,
  });

  final AppNetwork network;
  final String walletPublicKey;
  final int solBalanceLamports;
  final List<TokenAsset> tokens;
  final List<TransactionHistoryEntry> recentTransactions;
  final int? refreshedAtMs;
}

class TokenAsset {
  const TokenAsset({
    required this.tokenAccount,
    required this.mint,
    required this.symbol,
    required this.name,
    required this.amount,
    required this.rawAmount,
    required this.decimals,
    this.logoUri,
  });

  final String tokenAccount;
  final String mint;
  final String symbol;
  final String name;
  final String amount;
  final String rawAmount;
  final int decimals;
  final String? logoUri;
}

class TransactionHistoryEntry {
  const TransactionHistoryEntry({
    required this.signature,
    required this.slot,
    required this.status,
    this.blockTime,
  });

  final String signature;
  final int slot;
  final int? blockTime;
  final String status;
}

class ExportPrivateKeyResponse {
  const ExportPrivateKeyResponse({
    required this.publicKey,
    required this.privateKeyBase58,
  });

  final String publicKey;
  final String privateKeyBase58;
}

class SigningPreview {
  const SigningPreview({
    required this.id,
    required this.title,
    required this.network,
    required this.walletPublicKey,
    required this.summary,
    required this.warnings,
    this.requiresUserConfirmation = true,
  });

  final String id;
  final String title;
  final AppNetwork network;
  final String walletPublicKey;
  final String summary;
  final List<String> warnings;
  final bool requiresUserConfirmation;
}

class PaymentSigningDraft {
  const PaymentSigningDraft({
    required this.preview,
    required this.recipient,
    required this.amountBaseUnits,
    this.operation = PaymentOperation.solTransfer,
    this.mint,
  });

  final SigningPreview preview;
  final String recipient;
  final int amountBaseUnits;
  final PaymentOperation operation;
  final String? mint;
}

class DappSigningDraft {
  const DappSigningDraft({
    required this.preview,
    required this.method,
    required this.payloadBase64,
    this.requestId,
    this.transactionFormat,
  });

  final SigningPreview preview;
  final String method;
  final String payloadBase64;
  final String? requestId;
  final String? transactionFormat;
}

class DappSignResponse {
  const DappSignResponse({
    required this.requestId,
    required this.approved,
    this.signature,
    this.signatureBase64,
    this.signedPayloadBase64,
    this.signedPayloadsBase64 = const [],
    this.transactionSignature,
    this.error,
  });

  final String requestId;
  final bool approved;
  final String? signature;
  final String? signatureBase64;
  final String? signedPayloadBase64;
  final List<String> signedPayloadsBase64;
  final String? transactionSignature;
  final String? error;
}

enum SquadsDraftKind {
  create,
  solTransferProposal,
  tokenTransferProposal,
  approve,
  reject,
  execute,
}

class SquadsSigningDraft {
  const SquadsSigningDraft({
    required this.preview,
    required this.kind,
    this.members = const [],
    this.threshold = 1,
    this.timeLock,
    this.multisig,
    this.proposal,
    this.transactionIndex,
    this.recipient,
    this.destinationTokenAccount,
    this.sourceTokenAccount,
    this.mint,
    this.amountBaseUnits = 0,
    this.decimals,
    this.memo,
  });

  final SigningPreview preview;
  final SquadsDraftKind kind;
  final List<String> members;
  final int threshold;
  final int? timeLock;
  final String? multisig;
  final String? proposal;
  final int? transactionIndex;
  final String? recipient;
  final String? destinationTokenAccount;
  final String? sourceTokenAccount;
  final String? mint;
  final int amountBaseUnits;
  final int? decimals;
  final String? memo;
}

enum PaymentOperation {
  solTransfer,
  splTokenTransfer,
  wsolWrap,
  wsolUnwrap,
  wsolCloseAta,
}

class TransactionSubmitResult {
  const TransactionSubmitResult({
    required this.signature,
    required this.network,
    required this.submittedAt,
    required this.status,
    this.slot,
  });

  final String signature;
  final int? slot;
  final AppNetwork network;
  final String submittedAt;
  final String status;
}

class DappSignSubmitResult {
  const DappSignSubmitResult({
    required this.status,
    this.signature,
    this.signatureBase64,
    this.signedPayloadBase64,
    this.signedPayloadsBase64 = const [],
    this.transaction,
  });

  final String status;
  final String? signature;
  final String? signatureBase64;
  final String? signedPayloadBase64;
  final List<String> signedPayloadsBase64;
  final TransactionSubmitResult? transaction;
}

class SquadsMemberSummary {
  const SquadsMemberSummary({
    required this.key,
    required this.permissions,
  });

  final String key;
  final int permissions;
}

class SquadsProposalSummary {
  const SquadsProposalSummary({
    required this.address,
    required this.transactionIndex,
    required this.status,
    required this.approved,
    required this.rejected,
    required this.cancelled,
  });

  final String address;
  final int transactionIndex;
  final String status;
  final List<String> approved;
  final List<String> rejected;
  final List<String> cancelled;
}

class SquadsInfo {
  const SquadsInfo({
    required this.multisig,
    required this.vault,
    required this.createKey,
    required this.threshold,
    required this.timeLock,
    required this.transactionIndex,
    required this.staleTransactionIndex,
    required this.members,
    required this.network,
    this.proposal,
  });

  final String multisig;
  final String vault;
  final String createKey;
  final int threshold;
  final int timeLock;
  final int transactionIndex;
  final int staleTransactionIndex;
  final List<SquadsMemberSummary> members;
  final SquadsProposalSummary? proposal;
  final AppNetwork network;
}

class SquadsProposals {
  const SquadsProposals({
    required this.multisig,
    required this.vault,
    required this.proposals,
    required this.latestTransactionIndex,
    required this.network,
  });

  final String multisig;
  final String vault;
  final List<SquadsProposalSummary> proposals;
  final int latestTransactionIndex;
  final AppNetwork network;
}

class SquadsCreateResult {
  const SquadsCreateResult({
    required this.multisig,
    required this.vault,
    required this.createKey,
    required this.signature,
    required this.threshold,
    required this.members,
    required this.creationFeeLamports,
    required this.network,
    required this.status,
  });

  final String multisig;
  final String vault;
  final String createKey;
  final String signature;
  final int threshold;
  final List<SquadsMemberSummary> members;
  final int creationFeeLamports;
  final AppNetwork network;
  final String status;
}

class SquadsProposalCreateResult {
  const SquadsProposalCreateResult({
    required this.multisig,
    required this.vault,
    required this.transaction,
    required this.proposal,
    required this.transactionIndex,
    required this.signature,
    required this.network,
    required this.status,
  });

  final String multisig;
  final String vault;
  final String transaction;
  final String proposal;
  final int transactionIndex;
  final String signature;
  final AppNetwork network;
  final String status;
}

enum SquadsTransferKind { sol, splToken }

class SigningDecision {
  const SigningDecision({
    required this.previewId,
    required this.approved,
    required this.status,
  });

  final String previewId;
  final bool approved;
  final String status;
}

class MobileCapabilities {
  const MobileCapabilities({
    required this.enabled,
    required this.excluded,
  });

  final List<String> enabled;
  final List<String> excluded;
}

class TotpSetup {
  const TotpSetup({
    required this.secret,
    required this.issuer,
    required this.account,
  });

  final String secret;
  final String issuer;
  final String account;
}

class BiometricPolicy {
  const BiometricPolicy({
    required this.supported,
    required this.configured,
    this.reason,
  });

  final bool supported;
  final bool configured;
  final String? reason;
}

class MobileBridgeException implements Exception {
  const MobileBridgeException(this.code, this.message);

  final String code;
  final String message;

  @override
  String toString() => message;
}
