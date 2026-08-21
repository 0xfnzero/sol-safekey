import 'package:local_auth/local_auth.dart';

import '../bridge/mobile_models.dart';
import '../storage/mobile_wallet_store.dart';

class BiometricGate {
  BiometricGate(
    this._walletStore, {
    LocalAuthentication? localAuthentication,
  }) : _localAuthentication = localAuthentication ?? LocalAuthentication();

  final MobileWalletStore _walletStore;
  final LocalAuthentication _localAuthentication;

  Future<void> confirmSensitiveSubmit() async {
    final enabled = await _walletStore.isBiometricUnlockEnabled();
    if (!enabled) return;

    final supported = await _localAuthentication.isDeviceSupported();
    final canCheck = await _localAuthentication.canCheckBiometrics;
    if (!supported || !canCheck) {
      throw const MobileBridgeException(
        'biometric_cancelled',
        'Biometric confirmation is enabled but unavailable on this device.',
      );
    }

    final authenticated = await _localAuthentication.authenticate(
      localizedReason: 'Confirm this wallet signing action.',
      options: const AuthenticationOptions(
        biometricOnly: true,
        stickyAuth: true,
        sensitiveTransaction: true,
      ),
    );
    if (!authenticated) {
      throw const MobileBridgeException(
        'biometric_cancelled',
        'Biometric confirmation was cancelled.',
      );
    }
  }
}
