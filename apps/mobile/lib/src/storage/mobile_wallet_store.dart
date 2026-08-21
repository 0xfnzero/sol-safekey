import 'dart:convert';
import 'dart:io';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:path_provider/path_provider.dart';

import '../bridge/mobile_models.dart';

class MobileWalletStore {
  MobileWalletStore({
    FlutterSecureStorage secureStorage = const FlutterSecureStorage(
      aOptions: AndroidOptions(encryptedSharedPreferences: true),
      iOptions: IOSOptions(
        accessibility: KeychainAccessibility.first_unlock_this_device,
      ),
    ),
  }) : _secureStorage = secureStorage;

  static const _walletsKey = 'fnzero.mobile.wallets.v1';
  static const _activeWalletKey = 'fnzero.mobile.active_wallet.v1';
  static const _biometricEnabledKey = 'fnzero.mobile.biometric_enabled.v1';

  final FlutterSecureStorage _secureStorage;

  Future<List<WalletSummary>> loadWallets() async {
    final encoded = await _secureStorage.read(key: _walletsKey);
    if (encoded == null || encoded.trim().isEmpty) return const [];

    final decoded = jsonDecode(encoded) as List<dynamic>;
    return [
      for (final item in decoded)
        WalletSummary.fromJson(
            Map<String, Object?>.from(item as Map<dynamic, dynamic>)),
    ];
  }

  Future<WalletSummary?> loadActiveWallet() async {
    final wallets = await loadWallets();
    final activeWalletId = await _secureStorage.read(key: _activeWalletKey);
    if (activeWalletId == null) return wallets.firstOrNull;

    for (final wallet in wallets) {
      if (wallet.id == activeWalletId) return wallet;
    }
    return wallets.firstOrNull;
  }

  Future<void> saveWalletKeystore(WalletKeystore keystore) async {
    final wallets = await loadWallets();
    final nextWallets = [
      for (final wallet in wallets)
        if (wallet.id != keystore.wallet.id) wallet,
      keystore.wallet,
    ];

    await _writeKeystoreFile(keystore.wallet.id, keystore.keystoreJson);
    await _secureStorage.write(
      key: _walletsKey,
      value: jsonEncode([for (final wallet in nextWallets) wallet.toJson()]),
    );
    await setActiveWallet(keystore.wallet.id);
  }

  Future<void> setActiveWallet(String walletId) async {
    await _secureStorage.write(key: _activeWalletKey, value: walletId);
  }

  Future<void> deleteWallet(String walletId) async {
    final wallets = await loadWallets();
    final nextWallets = [
      for (final wallet in wallets)
        if (wallet.id != walletId) wallet,
    ];
    final file = await _keystoreFile(walletId);
    if (await file.exists()) {
      await file.delete();
    }
    await _secureStorage.write(
      key: _walletsKey,
      value: jsonEncode([for (final wallet in nextWallets) wallet.toJson()]),
    );
    final activeWalletId = await _secureStorage.read(key: _activeWalletKey);
    if (activeWalletId == walletId) {
      if (nextWallets.isEmpty) {
        await _secureStorage.delete(key: _activeWalletKey);
      } else {
        await setActiveWallet(nextWallets.first.id);
      }
    }
  }

  Future<String> readKeystoreJson(String walletId) async {
    final file = await _keystoreFile(walletId);
    return file.readAsString();
  }

  Future<void> setBiometricUnlockEnabled(bool enabled) async {
    await _secureStorage.write(
        key: _biometricEnabledKey, value: enabled ? 'true' : 'false');
  }

  Future<bool> isBiometricUnlockEnabled() async {
    return await _secureStorage.read(key: _biometricEnabledKey) == 'true';
  }

  Future<File> _keystoreFile(String walletId) async {
    final directory = await _walletDirectory();
    return File('${directory.path}/$walletId.json');
  }

  Future<void> _writeKeystoreFile(String walletId, String keystoreJson) async {
    final file = await _keystoreFile(walletId);
    await file.writeAsString(keystoreJson, flush: true);
  }

  Future<Directory> _walletDirectory() async {
    final root = await getApplicationSupportDirectory();
    final directory = Directory('${root.path}/wallets');
    if (!await directory.exists()) {
      await directory.create(recursive: true);
    }
    return directory;
  }
}

extension _FirstOrNull<T> on List<T> {
  T? get firstOrNull => isEmpty ? null : first;
}
