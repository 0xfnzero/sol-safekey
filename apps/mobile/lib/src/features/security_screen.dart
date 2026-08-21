import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:local_auth/local_auth.dart';

import '../bridge/mobile_bridge_provider.dart';

class SecurityScreen extends ConsumerStatefulWidget {
  const SecurityScreen({super.key});

  @override
  ConsumerState<SecurityScreen> createState() => _SecurityScreenState();
}

class _SecurityScreenState extends ConsumerState<SecurityScreen> {
  final _totpSecretController = TextEditingController();
  final _totpCodeController = TextEditingController();
  final _localAuth = LocalAuthentication();
  bool _busy = false;

  @override
  void dispose() {
    _totpSecretController.dispose();
    _totpCodeController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final wallet = ref.watch(activeWalletProvider);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          tooltip: 'Back',
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/'),
        ),
        title: const Text('Security'),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: ListTile(
              leading: const Icon(Icons.verified_user_outlined),
              title: const Text('Active wallet'),
              subtitle: Text(wallet?.publicKey ?? 'No active wallet'),
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _totpSecretController,
            obscureText: true,
            enableSuggestions: false,
            autocorrect: false,
            decoration: const InputDecoration(
                labelText: 'TOTP secret', border: OutlineInputBorder()),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _totpCodeController,
            keyboardType: TextInputType.number,
            decoration: const InputDecoration(
                labelText: 'TOTP code', border: OutlineInputBorder()),
          ),
          const SizedBox(height: 12),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              FilledButton.icon(
                onPressed: _busy ? null : _setupTotp,
                icon: const Icon(Icons.password),
                label: const Text('Setup TOTP'),
              ),
              OutlinedButton.icon(
                onPressed: _busy ? null : _verifyTotp,
                icon: const Icon(Icons.check_circle_outline),
                label: const Text('Verify TOTP'),
              ),
              OutlinedButton.icon(
                onPressed: _busy ? null : _authenticateBiometric,
                icon: const Icon(Icons.fingerprint),
                label: const Text('Biometric check'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Future<void> _setupTotp() async {
    await _run(() async {
      final account =
          ref.read(activeWalletProvider)?.publicKey ?? 'mobile-wallet';
      final setup = await ref.read(mobileBridgeProvider).setupTotp(account);
      _totpSecretController.text = setup.secret;
      _show('TOTP setup generated for ${setup.account}');
    });
  }

  Future<void> _verifyTotp() async {
    await _run(() async {
      await ref.read(mobileBridgeProvider).verifyTotp(
            secret: _totpSecretController.text,
            code: _totpCodeController.text,
          );
      _totpCodeController.clear();
      _show('TOTP verified');
    });
  }

  Future<void> _authenticateBiometric() async {
    await _run(() async {
      final supported = await _localAuth.isDeviceSupported();
      final canCheck = await _localAuth.canCheckBiometrics;
      if (!supported || !canCheck) {
        _show('Biometric authentication is not available on this device');
        return;
      }
      final authenticated = await _localAuth.authenticate(
        localizedReason: 'Confirm wallet unlock or signing action.',
        options:
            const AuthenticationOptions(biometricOnly: true, stickyAuth: true),
      );
      await ref
          .read(mobileWalletStoreProvider)
          .setBiometricUnlockEnabled(authenticated);
      _show(authenticated
          ? 'Biometric confirmation enabled'
          : 'Biometric confirmation cancelled');
    });
  }

  Future<void> _run(Future<void> Function() action) async {
    setState(() => _busy = true);
    try {
      await action();
    } catch (error) {
      _show(error.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _show(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context)
        .showSnackBar(SnackBar(content: Text(message)));
  }
}
