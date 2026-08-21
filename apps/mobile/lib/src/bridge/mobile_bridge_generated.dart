import 'generated/api.dart' as gen;
import 'generated/frb_generated.dart' as frb;
import 'mobile_bridge_backend.dart';
import 'mobile_models.dart';

class GeneratedMobileBridgeBackend implements MobileBridgeBackend {
  GeneratedMobileBridgeBackend();

  Future<void>? _initFuture;

  Future<void> _ensureInitialized() {
    return _initFuture ??= frb.RustLib.init();
  }

  @override
  Future<String> health() async {
    await _ensureInitialized();
    final health = await gen.health();
    return '${health.service}:${health.version}';
  }

  @override
  Future<MobileCapabilities> capabilities() async {
    await _ensureInitialized();
    final capabilities = await gen.getMobileCapabilities();
    return MobileCapabilities(
      enabled: capabilities.enabled,
      excluded: capabilities.excluded,
    );
  }

  @override
  Future<WalletKeystore> createWallet({
    required String name,
    required String password,
  }) async {
    return _guard(() async {
      await _ensureInitialized();
      final created = await gen.walletCreate(
        req: gen.CreateWalletRequest(name: name, password: password),
      );
      return _walletKeystore(created);
    });
  }

  @override
  Future<WalletKeystore> importKeystore({
    required String name,
    required String keystoreJson,
    required String password,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final imported = await gen.walletImportKeystore(
        req: gen.ImportKeystoreRequest(
          name: name,
          keystoreJson: keystoreJson,
          password: password,
        ),
      );
      return _walletKeystore(imported);
    });
  }

  @override
  Future<WalletKeystore> importPrivateKey({
    required String name,
    required String privateKeyBase58,
    required String password,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final imported = await gen.walletImportPrivateKey(
        req: gen.ImportPrivateKeyRequest(
          name: name,
          privateKeyBase58: privateKeyBase58,
          password: password,
        ),
      );
      return _walletKeystore(imported);
    });
  }

  @override
  Future<WalletKeystore> importMnemonic({
    required String name,
    required String mnemonic,
    required String password,
    String? derivationPath,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final imported = await gen.walletImportMnemonic(
        req: gen.ImportMnemonicRequest(
          name: name,
          mnemonic: mnemonic,
          derivationPath: derivationPath,
          password: password,
        ),
      );
      return _walletKeystore(imported);
    });
  }

  @override
  Future<WalletSummary> unlockWallet({
    required String keystoreJson,
    required String password,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final unlocked = await gen.walletUnlock(
        req: gen.UnlockWalletRequest(
          keystoreJson: keystoreJson,
          password: password,
        ),
      );
      return _walletSummary(unlocked.wallet);
    });
  }

  @override
  Future<ExportPrivateKeyResponse> exportPrivateKey({
    required String keystoreJson,
    required String password,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final exported = await gen.walletExportPrivateKey(
        req: gen.ExportPrivateKeyRequest(
          keystoreJson: keystoreJson,
          password: password,
        ),
      );
      return ExportPrivateKeyResponse(
        publicKey: exported.publicKey,
        privateKeyBase58: exported.privateKeyBase58,
      );
    });
  }

  @override
  Future<AssetSnapshot> loadAssets({
    required AppNetwork network,
    required String walletPublicKey,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final snapshot = await gen.assetsSnapshot(
        req: gen.AssetQueryRequest(
          network: _networkToGenerated(network),
          walletPublicKey: walletPublicKey,
        ),
      );
      return _assetSnapshot(snapshot);
    });
  }

  @override
  Future<SigningPreview> previewPayment({
    required AppNetwork network,
    required String walletPublicKey,
    required String recipient,
    required String amount,
    String? mint,
    String? memo,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final preview = await gen.paymentPreview(
        req: gen.PaymentPreviewRequest(
          network: _networkToGenerated(network),
          walletPublicKey: walletPublicKey,
          recipient: recipient,
          mint: mint,
          amount: amount,
          memo: memo,
        ),
      );
      return _signingPreview(preview);
    });
  }

  @override
  Future<TransactionSubmitResult> confirmPayment({
    required SigningPreview preview,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String recipient,
    required int amountBaseUnits,
    required PaymentOperation operation,
    String? mint,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.paymentConfirm(
        req: gen.PaymentSubmitRequest(
          previewId: preview.id,
          approved: approved,
          network: _networkToGenerated(preview.network),
          walletPublicKey: preview.walletPublicKey,
          keystoreJson: keystoreJson,
          password: password,
          operation: _paymentOperationToGenerated(operation),
          recipient: recipient,
          mint: mint,
          amountBaseUnits: BigInt.from(amountBaseUnits),
        ),
      );
      return TransactionSubmitResult(
        signature: result.signature,
        slot: result.slot?.toInt(),
        network: _networkFromGenerated(result.network),
        submittedAt: result.submittedAt,
        status: result.status,
      );
    });
  }

  @override
  Future<TotpSetup> setupTotp(String account) {
    return _guard(() async {
      await _ensureInitialized();
      final setup = await gen.securitySetupTotp(account: account);
      return TotpSetup(
          secret: setup.secret, issuer: setup.issuer, account: setup.account);
    });
  }

  @override
  Future<bool> verifyTotp({
    required String secret,
    required String code,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      return gen.securityVerifyTotp(
          req: gen.TotpVerifyRequest(secret: secret, code: code));
    });
  }

  @override
  Future<BiometricPolicy> biometricPolicy() async {
    await _ensureInitialized();
    final policy = await gen.securityBiometricPolicy();
    return BiometricPolicy(
      supported: policy.supported,
      configured: policy.configured,
      reason: policy.reason,
    );
  }

  @override
  Future<SigningPreview> previewPumpSell({
    required AppNetwork network,
    required String walletPublicKey,
    required String mint,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final preview = await gen.pumpPreview(
        req: gen.PumpPreviewRequest(
          network: _networkToGenerated(network),
          walletPublicKey: walletPublicKey,
          mint: mint,
          sellPercentBps: 10000,
          slippageBps: 100,
          venue: 'PumpFun',
        ),
      );
      return _signingPreview(preview);
    });
  }

  @override
  Future<SigningPreview> previewDappSign({
    required AppNetwork network,
    required String walletPublicKey,
    required String appName,
    required String appUrl,
    required String method,
    required String payloadBase64,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final preview = await gen.dappSignPreview(
        req: gen.DappSignPreviewRequest(
          network: _networkToGenerated(network),
          walletPublicKey: walletPublicKey,
          appName: appName,
          appUrl: appUrl,
          method: method,
          payloadBase64: payloadBase64,
        ),
      );
      return _signingPreview(preview);
    });
  }

  @override
  Future<DappSignSubmitResult> confirmDappSign({
    required SigningPreview preview,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String method,
    required String payloadBase64,
    String? transactionFormat,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.dappSignConfirm(
        req: gen.DappSignSubmitRequest(
          previewId: preview.id,
          approved: approved,
          network: _networkToGenerated(preview.network),
          walletPublicKey: preview.walletPublicKey,
          keystoreJson: keystoreJson,
          password: password,
          method: method,
          payloadBase64: payloadBase64,
          transactionFormat: transactionFormat,
        ),
      );
      return DappSignSubmitResult(
        status: result.status,
        signature: result.signature,
        signatureBase64: result.signatureBase64,
        signedPayloadBase64: result.signedPayloadBase64,
        signedPayloadsBase64: result.signedPayloadsBase64,
        transaction: result.transaction == null
            ? null
            : TransactionSubmitResult(
                signature: result.transaction!.signature,
                slot: result.transaction!.slot?.toInt(),
                network: _networkFromGenerated(result.transaction!.network),
                submittedAt: result.transaction!.submittedAt,
                status: result.transaction!.status,
              ),
      );
    });
  }

  @override
  Future<SigningPreview> previewSquadsAction({
    required AppNetwork network,
    required String walletPublicKey,
    required String multisig,
    required String action,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final preview = await gen.squadsPreview(
        req: gen.SquadsPreviewRequest(
          network: _networkToGenerated(network),
          walletPublicKey: walletPublicKey,
          multisig: multisig,
          action: action,
        ),
      );
      return _signingPreview(preview);
    });
  }

  @override
  Future<SquadsInfo> loadSquadsInfo({
    required AppNetwork network,
    required String multisig,
    String? proposal,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final info = await gen.squadsInfoQuery(
        req: gen.SquadsInfoRequest(
          network: _networkToGenerated(network),
          multisig: multisig,
          proposal: proposal,
        ),
      );
      return _squadsInfo(info);
    });
  }

  @override
  Future<SquadsProposals> loadSquadsProposals({
    required AppNetwork network,
    required String multisig,
    int? limit,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final proposals = await gen.squadsProposalsQuery(
        req: gen.SquadsProposalsRequest(
          network: _networkToGenerated(network),
          multisig: multisig,
          limit: limit == null ? null : BigInt.from(limit),
        ),
      );
      return SquadsProposals(
        multisig: proposals.multisig,
        vault: proposals.vault,
        proposals: [
          for (final proposal in proposals.proposals) _squadsProposal(proposal)
        ],
        latestTransactionIndex: proposals.latestTransactionIndex.toInt(),
        network: _networkFromGenerated(proposals.network),
      );
    });
  }

  @override
  Future<SquadsCreateResult> confirmSquadsCreate({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required List<String> members,
    required int threshold,
    int? timeLock,
    String? memo,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.squadsCreateConfirm(
        req: gen.SquadsCreateSubmitRequest(
          approved: approved,
          network: _networkToGenerated(network),
          keystoreJson: keystoreJson,
          password: password,
          members: members,
          threshold: threshold,
          timeLock: timeLock,
          memo: memo,
        ),
      );
      return SquadsCreateResult(
        multisig: result.multisig,
        vault: result.vault,
        createKey: result.createKey,
        signature: result.signature,
        threshold: result.threshold,
        members: [for (final member in result.members) _squadsMember(member)],
        creationFeeLamports: result.creationFeeLamports.toInt(),
        network: _networkFromGenerated(result.network),
        status: result.status,
      );
    });
  }

  @override
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
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.squadsTransferProposalConfirm(
        req: gen.SquadsTransferProposalSubmitRequest(
          approved: approved,
          network: _networkToGenerated(network),
          keystoreJson: keystoreJson,
          password: password,
          multisig: multisig,
          kind: _squadsTransferKindToGenerated(kind),
          recipient: recipient,
          destinationTokenAccount: destinationTokenAccount,
          sourceTokenAccount: sourceTokenAccount,
          mint: mint,
          amountBaseUnits: BigInt.from(amountBaseUnits),
          decimals: decimals,
          memo: memo,
        ),
      );
      return _squadsProposalCreateResult(result);
    });
  }

  @override
  Future<TransactionSubmitResult> confirmSquadsApprove({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required String proposal,
    String? memo,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.squadsApproveConfirm(
        req: gen.SquadsVoteSubmitRequest(
          approved: approved,
          network: _networkToGenerated(network),
          keystoreJson: keystoreJson,
          password: password,
          multisig: multisig,
          proposal: proposal,
          memo: memo,
        ),
      );
      return _transactionSubmitResult(result);
    });
  }

  @override
  Future<TransactionSubmitResult> confirmSquadsReject({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required String proposal,
    String? memo,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.squadsRejectConfirm(
        req: gen.SquadsVoteSubmitRequest(
          approved: approved,
          network: _networkToGenerated(network),
          keystoreJson: keystoreJson,
          password: password,
          multisig: multisig,
          proposal: proposal,
          memo: memo,
        ),
      );
      return _transactionSubmitResult(result);
    });
  }

  @override
  Future<TransactionSubmitResult> confirmSquadsExecute({
    required AppNetwork network,
    required bool approved,
    required String keystoreJson,
    required String password,
    required String multisig,
    required String proposal,
    required int transactionIndex,
  }) {
    return _guard(() async {
      await _ensureInitialized();
      final result = await gen.squadsExecuteConfirm(
        req: gen.SquadsExecuteSubmitRequest(
          approved: approved,
          network: _networkToGenerated(network),
          keystoreJson: keystoreJson,
          password: password,
          multisig: multisig,
          proposal: proposal,
          transactionIndex: BigInt.from(transactionIndex),
        ),
      );
      return _transactionSubmitResult(result);
    });
  }
}

Future<T> _guard<T>(Future<T> Function() run) async {
  try {
    return await run();
  } on gen.MobileError catch (error) {
    throw MobileBridgeException(error.code.name, error.message);
  }
}

WalletKeystore _walletKeystore(gen.WalletKeystore value) {
  return WalletKeystore(
    wallet: _walletSummary(value.wallet),
    keystoreJson: value.keystoreJson,
  );
}

WalletSummary _walletSummary(gen.WalletSummary value) {
  return WalletSummary(
    id: value.id,
    name: value.name,
    publicKey: value.publicKey,
  );
}

AssetSnapshot _assetSnapshot(gen.AssetSnapshot value) {
  return AssetSnapshot(
    network: _networkFromGenerated(value.network),
    walletPublicKey: value.walletPublicKey,
    solBalanceLamports: value.solBalanceLamports.toInt(),
    tokens: [for (final token in value.tokens) _tokenAsset(token)],
    recentTransactions: [
      for (final entry in value.recentTransactions) _historyEntry(entry)
    ],
    refreshedAtMs: value.refreshedAtMs.toInt(),
  );
}

TokenAsset _tokenAsset(gen.AssetSummary value) {
  return TokenAsset(
    tokenAccount: value.tokenAccount,
    mint: value.mint,
    symbol: value.symbol,
    name: value.name,
    amount: value.amount,
    rawAmount: value.rawAmount,
    decimals: value.decimals,
    logoUri: value.logoUri,
  );
}

TransactionHistoryEntry _historyEntry(gen.TransactionHistoryEntry value) {
  return TransactionHistoryEntry(
    signature: value.signature,
    slot: value.slot.toInt(),
    blockTime: value.blockTime?.toInt(),
    status: value.status,
  );
}

TransactionSubmitResult _transactionSubmitResult(
    gen.TransactionSubmitResult value) {
  return TransactionSubmitResult(
    signature: value.signature,
    slot: value.slot?.toInt(),
    network: _networkFromGenerated(value.network),
    submittedAt: value.submittedAt,
    status: value.status,
  );
}

SquadsMemberSummary _squadsMember(gen.SquadsMemberSummary value) {
  return SquadsMemberSummary(
    key: value.key,
    permissions: value.permissions,
  );
}

SquadsProposalSummary _squadsProposal(gen.SquadsProposalSummary value) {
  return SquadsProposalSummary(
    address: value.address,
    transactionIndex: value.transactionIndex.toInt(),
    status: value.status,
    approved: value.approved,
    rejected: value.rejected,
    cancelled: value.cancelled,
  );
}

SquadsInfo _squadsInfo(gen.SquadsInfoResponse value) {
  return SquadsInfo(
    multisig: value.multisig,
    vault: value.vault,
    createKey: value.createKey,
    threshold: value.threshold,
    timeLock: value.timeLock,
    transactionIndex: value.transactionIndex.toInt(),
    staleTransactionIndex: value.staleTransactionIndex.toInt(),
    members: [for (final member in value.members) _squadsMember(member)],
    proposal: value.proposal == null ? null : _squadsProposal(value.proposal!),
    network: _networkFromGenerated(value.network),
  );
}

SquadsProposalCreateResult _squadsProposalCreateResult(
  gen.SquadsProposalCreateSubmitResult value,
) {
  return SquadsProposalCreateResult(
    multisig: value.multisig,
    vault: value.vault,
    transaction: value.transaction,
    proposal: value.proposal,
    transactionIndex: value.transactionIndex.toInt(),
    signature: value.signature,
    network: _networkFromGenerated(value.network),
    status: value.status,
  );
}

SigningPreview _signingPreview(gen.SigningPreview value) {
  return SigningPreview(
    id: value.id,
    title: value.title,
    network: _networkFromGenerated(value.network),
    walletPublicKey: value.walletPublicKey,
    summary: value.summary,
    warnings: value.warnings,
    requiresUserConfirmation: value.requiresUserConfirmation,
  );
}

gen.AppNetwork _networkToGenerated(AppNetwork value) => switch (value) {
      AppNetwork.mainnet => gen.AppNetwork.mainnet,
      AppNetwork.devnet => gen.AppNetwork.devnet,
      AppNetwork.testnet => gen.AppNetwork.testnet,
    };

AppNetwork _networkFromGenerated(gen.AppNetwork value) => switch (value) {
      gen.AppNetwork.mainnet => AppNetwork.mainnet,
      gen.AppNetwork.devnet => AppNetwork.devnet,
      gen.AppNetwork.testnet => AppNetwork.testnet,
    };

gen.PaymentOperation _paymentOperationToGenerated(PaymentOperation value) =>
    switch (value) {
      PaymentOperation.solTransfer => gen.PaymentOperation.solTransfer,
      PaymentOperation.splTokenTransfer =>
        gen.PaymentOperation.splTokenTransfer,
      PaymentOperation.wsolWrap => gen.PaymentOperation.wsolWrap,
      PaymentOperation.wsolUnwrap => gen.PaymentOperation.wsolUnwrap,
      PaymentOperation.wsolCloseAta => gen.PaymentOperation.wsolCloseAta,
    };

gen.SquadsTransferKind _squadsTransferKindToGenerated(
  SquadsTransferKind value,
) =>
    switch (value) {
      SquadsTransferKind.sol => gen.SquadsTransferKind.sol,
      SquadsTransferKind.splToken => gen.SquadsTransferKind.splToken,
    };
