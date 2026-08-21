import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:intl/intl.dart';

import '../bridge/mobile_bridge_provider.dart';
import '../bridge/mobile_models.dart';

class AssetsScreen extends ConsumerStatefulWidget {
  const AssetsScreen({super.key});

  @override
  ConsumerState<AssetsScreen> createState() => _AssetsScreenState();
}

class _AssetsScreenState extends ConsumerState<AssetsScreen> {
  AssetSnapshot? _snapshot;
  Object? _error;
  bool _loading = false;

  @override
  Widget build(BuildContext context) {
    final wallet = ref.watch(activeWalletProvider);
    final network = ref.watch(activeNetworkProvider);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          tooltip: 'Back',
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/'),
        ),
        title: const Text('Assets'),
        actions: [
          IconButton(
            tooltip: 'Refresh',
            icon: const Icon(Icons.refresh),
            onPressed: wallet == null || _loading
                ? null
                : () => _refresh(wallet, network),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          if (wallet == null)
            const Card(
              child: ListTile(
                leading: Icon(Icons.account_balance_wallet_outlined),
                title: Text('Select or create a wallet first'),
              ),
            )
          else
            Card(
              child: ListTile(
                leading: const Icon(Icons.account_balance_wallet_outlined),
                title: Text(wallet.name),
                subtitle: Text(wallet.publicKey),
              ),
            ),
          const SizedBox(height: 16),
          if (_loading) const LinearProgressIndicator(),
          if (_error != null) ...[
            Card(
              child: ListTile(
                leading: const Icon(Icons.error_outline),
                title: const Text('Asset refresh failed'),
                subtitle: Text(_error.toString()),
              ),
            ),
            const SizedBox(height: 16),
          ],
          if (_snapshot != null) ...[
            Text('SOL', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            Card(
              child: ListTile(
                leading: const Icon(Icons.currency_exchange),
                title: Text(_formatSol(_snapshot!.solBalanceLamports)),
                subtitle: Text('${_snapshot!.network.label} balance'),
              ),
            ),
            const SizedBox(height: 20),
            Text('Tokens', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            if (_snapshot!.tokens.isEmpty)
              const Text('No SPL token accounts with non-zero balance.')
            else
              for (final token in _snapshot!.tokens)
                Card(
                  child: ListTile(
                    title: Text('${token.symbol}  ${token.amount}'),
                    subtitle: Text(
                        '${token.name}\n${token.mint}\nATA ${token.tokenAccount}'),
                    isThreeLine: true,
                  ),
                ),
            const SizedBox(height: 20),
            Text('Recent transactions',
                style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 8),
            if (_snapshot!.recentTransactions.isEmpty)
              const Text('No recent signatures found.')
            else
              for (final entry in _snapshot!.recentTransactions)
                Card(
                  child: ListTile(
                    title: Text(entry.status),
                    subtitle: Text(
                        '${entry.signature}\nslot ${entry.slot}${_blockTime(entry)}'),
                    isThreeLine: true,
                  ),
                ),
          ],
        ],
      ),
    );
  }

  Future<void> _refresh(WalletSummary wallet, AppNetwork network) async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final snapshot = await ref
          .read(mobileBridgeProvider)
          .loadAssets(network: network, walletPublicKey: wallet.publicKey);
      if (!mounted) return;
      setState(() => _snapshot = snapshot);
    } catch (error) {
      if (!mounted) return;
      setState(() => _error = error);
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  String _formatSol(int lamports) {
    final sol = lamports / 1000000000;
    return '${NumberFormat('#,##0.#########').format(sol)} SOL';
  }

  String _blockTime(TransactionHistoryEntry entry) {
    final blockTime = entry.blockTime;
    if (blockTime == null) return '';
    final time =
        DateTime.fromMillisecondsSinceEpoch(blockTime * 1000, isUtc: true);
    return ' - ${DateFormat.yMd().add_Hm().format(time.toLocal())}';
  }
}
