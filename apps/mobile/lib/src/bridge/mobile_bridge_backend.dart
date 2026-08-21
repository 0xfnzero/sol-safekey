import 'mobile_models.dart';

abstract interface class MobileBridgeBackend {
  Future<String> health();

  Future<MobileCapabilities> capabilities();

  Future<WalletKeystore> createWallet({
    required String name,
    required String password,
  });

  Future<WalletKeystore> importKeystore({
    required String name,
    required String keystoreJson,
    required String password,
  });

  Future<WalletKeystore> importPrivateKey({
    required String name,
    required String privateKeyBase58,
    required String password,
  });

  Future<WalletKeystore> importMnemonic({
    required String name,
    required String mnemonic,
    required String password,
    String? derivationPath,
  });

  Future<WalletSummary> unlockWallet({
    required String keystoreJson,
    required String password,
  });

  Future<ExportPrivateKeyResponse> exportPrivateKey({
    required String keystoreJson,
    required String password,
  });

  Future<AssetSnapshot> loadAssets({
    required AppNetwork network,
    required String walletPublicKey,
  });

  Future<SigningPreview> previewPayment({
    required AppNetwork network,
    required String walletPublicKey,
    required String recipient,
    required String amount,
    String? mint,
    String? memo,
  });

  Future<TransactionSubmitResult> confirmPayment({
    required SigningPreview preview,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String recipient,
    required int amountBaseUnits,
    required PaymentOperation operation,
    String? mint,
  });

  Future<TotpSetup> setupTotp(String account);

  Future<bool> verifyTotp({
    required String secret,
    required String code,
  });

  Future<BiometricPolicy> biometricPolicy();

  Future<SigningPreview> previewPumpSell({
    required AppNetwork network,
    required String walletPublicKey,
    required String mint,
  });

  Future<SigningPreview> previewDappSign({
    required AppNetwork network,
    required String walletPublicKey,
    required String appName,
    required String appUrl,
    required String method,
    required String payloadBase64,
  });

  Future<DappSignSubmitResult> confirmDappSign({
    required SigningPreview preview,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String method,
    required String payloadBase64,
    String? transactionFormat,
  });

  Future<SigningPreview> previewSquadsAction({
    required AppNetwork network,
    required String walletPublicKey,
    required String multisig,
    required String action,
  });

  Future<SquadsInfo> loadSquadsInfo({
    required AppNetwork network,
    required String multisig,
    String? proposal,
  });

  Future<SquadsProposals> loadSquadsProposals({
    required AppNetwork network,
    required String multisig,
    int? limit,
  });

  Future<SquadsCreateResult> confirmSquadsCreate({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required List<String> members,
    required int threshold,
    int? timeLock,
    String? memo,
  });

  Future<SquadsProposalCreateResult> confirmSquadsTransferProposal({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required SquadsTransferKind kind,
    String? recipient,
    String? destinationTokenAccount,
    String? sourceTokenAccount,
    String? mint,
    required int amountBaseUnits,
    int? decimals,
    String? memo,
  });

  Future<TransactionSubmitResult> confirmSquadsApprove({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required String proposal,
    String? memo,
  });

  Future<TransactionSubmitResult> confirmSquadsReject({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required String proposal,
    String? memo,
  });

  Future<TransactionSubmitResult> confirmSquadsExecute({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required String proposal,
    required int transactionIndex,
  });
}
