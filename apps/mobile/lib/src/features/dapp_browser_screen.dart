import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:webview_flutter/webview_flutter.dart';

import '../bridge/mobile_bridge_provider.dart';
import '../bridge/mobile_models.dart';

class DappBrowserScreen extends ConsumerStatefulWidget {
  const DappBrowserScreen({super.key});

  @override
  ConsumerState<DappBrowserScreen> createState() => _DappBrowserScreenState();
}

class _DappBrowserScreenState extends ConsumerState<DappBrowserScreen> {
  late final WebViewController _controller;
  final _urlController = TextEditingController(text: 'https://example.com');

  @override
  void initState() {
    super.initState();
    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..addJavaScriptChannel(
        'FnzeroSafeProvider',
        onMessageReceived: (message) {
          _handleProviderMessage(message.message);
        },
      )
      ..setNavigationDelegate(
        NavigationDelegate(
          onPageFinished: (_) => _injectProvider(),
        ),
      )
      ..loadRequest(Uri.parse(_urlController.text));
  }

  @override
  void dispose() {
    _urlController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final scanned = ref.watch(scannedValueProvider);
    final dappResponse = ref.watch(dappSignResponseProvider);

    if (scanned != null && scanned.startsWith(RegExp(r'https?://'))) {
      _urlController.text = scanned;
      ref.read(scannedValueProvider.notifier).state = null;
      _load();
    }
    if (dappResponse != null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _deliverDappResponse(dappResponse);
      });
    }

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          tooltip: 'Back',
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/dapps'),
        ),
        title: const Text('dApp Browser'),
        actions: [
          IconButton(
            tooltip: 'Scan URL',
            icon: const Icon(Icons.qr_code_scanner),
            onPressed: () => context.go('/scan'),
          ),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _urlController,
                    keyboardType: TextInputType.url,
                    decoration: const InputDecoration(
                      labelText: 'URL',
                      border: OutlineInputBorder(),
                    ),
                    onSubmitted: (_) => _load(),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton.filled(
                  tooltip: 'Go',
                  icon: const Icon(Icons.arrow_forward),
                  onPressed: _load,
                ),
              ],
            ),
          ),
          Expanded(child: WebViewWidget(controller: _controller)),
        ],
      ),
    );
  }

  void _load() {
    final text = _urlController.text.trim();
    if (text.isEmpty) return;
    final uri = Uri.parse(
        text.startsWith(RegExp(r'https?://')) ? text : 'https://$text');
    _controller.loadRequest(uri);
  }

  Future<void> _injectProvider() {
    final wallet = ref.read(activeWalletProvider);
    final publicKey = wallet?.publicKey;
    final script = '''
(() => {
  if (window.fnzeroSafe) return;
  const pending = new Map();
  let connected = ${publicKey == null ? 'false' : 'true'};
  let publicKeyValue = ${jsonEncode(publicKey)};
  const toSerializable = (value) => {
    if (value instanceof Uint8Array) return { __fnzeroBytes: Array.from(value) };
    if (value && value.constructor && value.constructor.name === 'Buffer') {
      return { __fnzeroBytes: Array.from(value) };
    }
    if (Array.isArray(value)) return value.map((item) => toSerializable(item));
    if (value && typeof value === 'object') {
      return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, toSerializable(item)]));
    }
    return value;
  };
  const base64ToBytes = (value) => {
    const binary = atob(value);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes;
  };
  const normalizeSuccess = (entry, payload) => {
    const method = entry.method;
    if (method === 'connect') {
      return { publicKey: provider.publicKey };
    }
    if (method === 'disconnect') {
      return undefined;
    }
    if (method === 'signMessage') {
      return {
        publicKey: provider.publicKey,
        signature: payload.signatureBase64 ? base64ToBytes(payload.signatureBase64) : payload.signature
      };
    }
    if (method === 'signAndSendTransaction') {
      return { signature: payload.transactionSignature || payload.signature };
    }
    if (method === 'signAllTransactions') {
      return {
        signedTransactions: payload.signedTransactions || [],
        signedTransactionBytes: (payload.signedTransactions || []).map((item) => base64ToBytes(item))
      };
    }
    if (method === 'signTransaction') {
      return {
        signedTransaction: payload.signedTransaction,
        signedTransactionBytes: payload.signedTransaction ? base64ToBytes(payload.signedTransaction) : undefined
      };
    }
    return payload;
  };
  const provider = {
    isFnzeroSafe: true,
    isPhantom: true,
    get isConnected() { return connected; },
    get publicKey() {
      if (!connected || !publicKeyValue) return null;
      return {
        toString: () => publicKeyValue,
        toBase58: () => publicKeyValue,
        toJSON: () => publicKeyValue
      };
    },
    connect: (opts) => provider.request({ method: 'connect', params: opts || {} }),
    disconnect: () => provider.request({ method: 'disconnect' }),
    signMessage: (message, encoding) => provider.request({
      method: 'signMessage',
      params: { message, encoding }
    }),
    signTransaction: (transaction) => provider.request({
      method: 'signTransaction',
      params: { transaction }
    }),
    signAllTransactions: (transactions) => provider.request({
      method: 'signAllTransactions',
      params: { transactions }
    }),
    signAndSendTransaction: (transaction, options) => provider.request({
      method: 'signAndSendTransaction',
      params: { transaction, options }
    }),
    request: (payload) => {
      const id = `\${Date.now()}-\${Math.random().toString(36).slice(2)}`;
      FnzeroSafeProvider.postMessage(JSON.stringify({ ...(payload || {}), __fnzeroRequestId: id }, (_, value) => toSerializable(value)));
      return new Promise((resolve, reject) => pending.set(id, {
        resolve,
        reject,
        method: payload && payload.method ? payload.method : 'request',
        params: payload && payload.params ? payload.params : undefined
      }));
    }
  };
  window.fnzeroSafeResolve = (id, ok, payload) => {
    const entry = pending.get(id);
    if (!entry) return;
    pending.delete(id);
    if (ok) {
      if (payload && payload.connected !== undefined) connected = !!payload.connected;
      if (payload && payload.publicKey !== undefined) publicKeyValue = payload.publicKey;
      entry.resolve(normalizeSuccess(entry, payload || {}));
    } else {
      const error = new Error(payload && payload.message ? payload.message : 'FnzeroSafe request rejected');
      error.code = payload && payload.code ? payload.code : 4001;
      entry.reject(error);
    }
  };
  window.fnzeroSafe = provider;
  window.solana = provider;
  window.dispatchEvent(new Event('fnzero#initialized'));
})();
''';
    return _controller.runJavaScript(script);
  }

  Future<void> _handleProviderMessage(String message) async {
    try {
      final wallet = ref.read(activeWalletProvider);
      if (wallet == null) {
        throw StateError('Select a wallet before using dApps');
      }
      final decoded = jsonDecode(message);
      final payload =
          decoded is Map<String, Object?> ? decoded : <String, Object?>{};
      final method = payload['method']?.toString() ?? 'request';
      final requestId = payload['__fnzeroRequestId']?.toString();
      if (method == 'connect') {
        await _deliverProviderResponse(
          requestId,
          true,
          {
            'publicKey': wallet.publicKey,
            'connected': true,
          },
        );
        return;
      }
      if (method == 'disconnect') {
        await _deliverProviderResponse(
          requestId,
          true,
          {
            'publicKey': null,
            'connected': false,
          },
        );
        return;
      }
      final signingPayloadBase64 =
          _signingPayloadBase64(method, payload, message);
      final transactionFormat = _transactionFormat(method);
      final preview = await ref.read(mobileBridgeProvider).previewDappSign(
            network: ref.read(activeNetworkProvider),
            walletPublicKey: wallet.publicKey,
            appName: Uri.tryParse(_urlController.text)?.host ?? 'dApp',
            appUrl: _urlController.text,
            method: method,
            payloadBase64: signingPayloadBase64,
          );
      ref.read(signingPreviewProvider.notifier).state = preview;
      ref.read(paymentSigningDraftProvider.notifier).state = null;
      ref.read(dappSigningDraftProvider.notifier).state = DappSigningDraft(
        preview: preview,
        method: method,
        payloadBase64: signingPayloadBase64,
        requestId: requestId,
        transactionFormat: transactionFormat,
      );
      if (mounted) await context.push('/confirm');
    } catch (error) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(error.toString())));
    }
  }

  String _signingPayloadBase64(
    String method,
    Map<String, Object?> payload,
    String rawMessage,
  ) {
    if (method == 'signAllTransactions') {
      final transactions = _extractTransactionList(payload, rawMessage)
          .map((value) => _valueToBase64(value, rawMessage))
          .toList(growable: false);
      return base64Encode(utf8.encode(jsonEncode(transactions)));
    }

    if (method != 'signMessage' && method != 'personal_sign') {
      final transaction = _extractPayloadValue(payload);
      return _valueToBase64(transaction, rawMessage);
    }

    final message = _extractPayloadValue(payload);

    return _valueToBase64(message, rawMessage);
  }

  String? _transactionFormat(String method) {
    return switch (method) {
      'signTransaction' || 'signAndSendTransaction' || 'signAllTransactions' =>
        'auto',
      _ => null,
    };
  }

  Object? _extractPayloadValue(Map<String, Object?> payload) {
    final params = payload['params'];
    if (params is Map<String, Object?>) {
      return params['transaction'] ??
          params['message'] ??
          params['data'] ??
          params['payload'];
    }
    if (params is List<Object?> && params.isNotEmpty) {
      return params.first;
    }
    return payload['transaction'] ?? payload['message'] ?? payload['data'];
  }

  List<Object?> _extractTransactionList(
    Map<String, Object?> payload,
    String rawMessage,
  ) {
    final params = payload['params'];
    Object? transactions;
    if (params is Map<String, Object?>) {
      transactions = params['transactions'] ?? params['transaction'];
    } else if (params is List<Object?>) {
      transactions = params;
    }
    transactions ??= payload['transactions'] ?? payload['transaction'];

    if (transactions is List<Object?> && !_looksLikeByteList(transactions)) {
      return transactions;
    }
    return [transactions ?? rawMessage];
  }

  bool _looksLikeByteList(List<Object?> value) {
    return value.isNotEmpty &&
        value.every((item) => item is int && item >= 0 && item <= 255);
  }

  String _valueToBase64(Object? value, String fallback) {
    if (value is String && value.isNotEmpty) {
      try {
        base64Decode(value);
        return value;
      } catch (_) {
        return base64Encode(utf8.encode(value));
      }
    }

    if (value is Map<String, Object?> && value['__fnzeroBytes'] is List) {
      return _valueToBase64(value['__fnzeroBytes'], fallback);
    }

    if (value is List<Object?>) {
      final bytes = <int>[];
      for (final item in value) {
        if (item is! int || item < 0 || item > 255) {
          return base64Encode(utf8.encode(fallback));
        }
        bytes.add(item);
      }
      return base64Encode(bytes);
    }

    return base64Encode(utf8.encode(fallback));
  }

  Future<void> _deliverDappResponse(DappSignResponse response) async {
    ref.read(dappSignResponseProvider.notifier).state = null;
    final payload = response.approved
        ? {
            'publicKey': ref.read(activeWalletProvider)?.publicKey,
            'connected': true,
            if (response.signature != null) 'signature': response.signature,
            if (response.signatureBase64 != null)
              'signatureBase64': response.signatureBase64,
            if (response.signedPayloadBase64 != null)
              'signedTransaction': response.signedPayloadBase64,
            if (response.signedPayloadsBase64.isNotEmpty)
              'signedTransactions': response.signedPayloadsBase64,
            if (response.transactionSignature != null)
              'transactionSignature': response.transactionSignature,
          }
        : {
            'message': response.error ?? 'FnzeroSafe request rejected',
            'code': 4001,
          };
    await _deliverProviderResponse(response.requestId, response.approved, payload);
  }

  Future<void> _deliverProviderResponse(
    String? requestId,
    bool approved,
    Map<String, Object?> payload,
  ) async {
    if (requestId == null) return;
    final script =
        'window.fnzeroSafeResolve(${jsonEncode(requestId)}, $approved, ${jsonEncode(payload)});';
    await _controller.runJavaScript(script);
  }
}
