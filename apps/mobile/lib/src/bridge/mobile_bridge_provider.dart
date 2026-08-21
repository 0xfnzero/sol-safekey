import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../storage/mobile_wallet_store.dart';
import '../security/biometric_gate.dart';
import 'mobile_bridge.dart';
import 'mobile_models.dart';

final mobileBridgeProvider = Provider<MobileBridge>((ref) => MobileBridge());

final mobileCapabilitiesProvider = FutureProvider<MobileCapabilities>((ref) {
  return ref.watch(mobileBridgeProvider).capabilities();
});

final mobileWalletStoreProvider = Provider<MobileWalletStore>((ref) {
  return MobileWalletStore();
});

final biometricGateProvider = Provider<BiometricGate>((ref) {
  return BiometricGate(ref.watch(mobileWalletStoreProvider));
});

final storedWalletsProvider = FutureProvider<List<WalletSummary>>((ref) {
  return ref.watch(mobileWalletStoreProvider).loadWallets();
});

final storedActiveWalletProvider = FutureProvider<WalletSummary?>((ref) {
  return ref.watch(mobileWalletStoreProvider).loadActiveWallet();
});

final activeNetworkProvider =
    StateProvider<AppNetwork>((ref) => AppNetwork.devnet);

final activeWalletProvider = StateProvider<WalletSummary?>((ref) => null);

final signingPreviewProvider = StateProvider<SigningPreview?>((ref) => null);

final paymentSigningDraftProvider =
    StateProvider<PaymentSigningDraft?>((ref) => null);

final dappSigningDraftProvider =
    StateProvider<DappSigningDraft?>((ref) => null);

final dappSignResponseProvider =
    StateProvider<DappSignResponse?>((ref) => null);

final squadsSigningDraftProvider =
    StateProvider<SquadsSigningDraft?>((ref) => null);

final scannedValueProvider = StateProvider<String?>((ref) => null);
