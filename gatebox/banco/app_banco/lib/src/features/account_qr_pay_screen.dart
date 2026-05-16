import 'dart:async' show unawaited;

import 'package:app_banco/src/core/banco_api_client.dart';
import 'package:app_banco/src/data/local_accounts_store.dart';
import 'package:flutter/foundation.dart' show defaultTargetPlatform, kIsWeb;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:intl/intl.dart';
import 'package:mobile_scanner/mobile_scanner.dart';

/// Aba PIX: câmera ativa enquanto a aba está visível; pagamento automático ao ler o QR.
class PixTabScreen extends StatefulWidget {
  const PixTabScreen({
    super.key,
    required this.account,
    required this.api,
    required this.pixTabActive,
    this.onPaymentSettled,
  });

  final SavedAccount? account;
  final BancoApiClient api;

  /// Quando falso (outra aba do [IndexedStack]), a câmera é libertada.
  final bool pixTabActive;

  /// Após pagamento aprovado: saldo vindo do backend (conta local actualizada).
  final void Function(SavedAccount updated)? onPaymentSettled;

  @override
  State<PixTabScreen> createState() => _PixTabScreenState();
}

class _PixTabScreenState extends State<PixTabScreen> {
  final _referenceController = TextEditingController();
  bool _paying = false;

  /// Evita múltiplos disparos do mesmo frame / mesmo código enquanto processa.
  bool _scanBusy = false;
  bool _autoPayEnabled = true;
  String? _lastPayRefKey;
  DateTime? _lastPayAt;
  MobileScannerController? _scannerController;

  /// Último pagamento concluído (para mostrar estado Gatebox / backend).
  Map<String, dynamic>? _lastPaymentJson;

  static const _sandboxSimulation = 'APPROVED';

  static bool get _cameraTarget {
    if (kIsWeb) return false;
    return defaultTargetPlatform == TargetPlatform.android ||
        defaultTargetPlatform == TargetPlatform.iOS;
  }

  bool get _shouldRunCamera =>
      widget.account != null &&
      widget.pixTabActive &&
      _cameraTarget &&
      _autoPayEnabled;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _syncCamera());
  }

  @override
  void dispose() {
    _scannerController?.dispose();
    _referenceController.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(PixTabScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.account?.id != widget.account?.id) {
      _referenceController.clear();
    }
    if (oldWidget.pixTabActive != widget.pixTabActive ||
        oldWidget.account?.id != widget.account?.id) {
      WidgetsBinding.instance.addPostFrameCallback((_) => _syncCamera());
    }
  }

  void _syncCamera() {
    if (!mounted) return;
    final want = _shouldRunCamera;
    if (want && _scannerController == null) {
      setState(() => _scannerController = MobileScannerController());
    } else if (!want && _scannerController != null) {
      _scannerController!.dispose();
      setState(() => _scannerController = null);
    }
  }

  String _methodForReference(String raw) {
    final t = raw.trim();
    if (t.startsWith('000201')) return 'pix';
    return 'qrcode';
  }

  double _scannerSide(double maxWidth) {
    final w = (maxWidth * 0.88).clamp(220.0, 340.0);
    return w;
  }

  void _clearPixPayload() {
    setState(() => _referenceController.clear());
  }

  static String _pixPayloadPreview(String t) {
    if (t.length <= 120) return t;
    return '${t.substring(0, 55)} … ${t.substring(t.length - 55)}';
  }

  String _refKey(String ref) {
    final t = ref.trim();
    if (t.length <= 120) return t;
    return '${t.substring(0, 60)}…${t.substring(t.length - 60)}';
  }

  bool _shouldThrottle(String ref) {
    final key = _refKey(ref);
    final lastKey = _lastPayRefKey;
    final lastAt = _lastPayAt;
    if (lastKey == null || lastAt == null) return false;
    if (key != lastKey) return false;
    return DateTime.now().difference(lastAt) < const Duration(seconds: 12);
  }

  static Map<String, String> _readInfoRows(String raw) {
    final t = raw.trim();
    final rows = <String, String>{'Caracteres': '${t.length}'};
    if (t.startsWith('000201')) {
      rows['Tipo'] = 'BR Code (PIX / copia e cola típico)';
    } else if (t.startsWith('GATEBOXRUST')) {
      rows['Tipo'] = 'QR Gatebox (sandbox HoldFy)';
    } else if (t.startsWith('http')) {
      rows['Tipo'] = 'URL';
    } else {
      rows['Tipo'] = 'Texto / QR genérico';
    }
    return rows;
  }

  Future<void> _autoPayFromScan(String ref) async {
    final acc = widget.account;
    if (acc == null || !mounted) {
      if (mounted) setState(() => _scanBusy = false);
      return;
    }
    final trimmed = ref.trim();
    if (trimmed.isEmpty) {
      if (mounted) setState(() => _scanBusy = false);
      return;
    }
    if (_shouldThrottle(trimmed)) {
      if (mounted) setState(() => _scanBusy = false);
      return;
    }

    setState(() {
      _referenceController.text = trimmed;
      _paying = true;
    });
    _syncCamera();

    Map<String, dynamic>? okPayment;
    try {
      _lastPayRefKey = _refKey(trimmed);
      _lastPayAt = DateTime.now();
      final method = _methodForReference(trimmed);
      final result = await widget.api.pay(
        method: method,
        reference: trimmed,
        simulationState: _sandboxSimulation,
      );
      if (!mounted) return;
      okPayment = result;

      final summary = _paymentSummaryLine(result);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(summary),
          duration: const Duration(seconds: 8),
          backgroundColor: Colors.green.shade800,
        ),
      );
      setState(() {
        _lastPaymentJson = Map<String, dynamic>.from(result);
        _referenceController.clear();
        // Evita pagar repetidamente o mesmo QR enquanto ele ainda está na câmera.
        _autoPayEnabled = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _lastPaymentJson = null);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('$e', style: const TextStyle(fontSize: 12)),
          duration: const Duration(seconds: 14),
          behavior: SnackBarBehavior.floating,
        ),
      );
    } finally {
      if (mounted) {
        setState(() {
          _paying = false;
          _scanBusy = false;
        });
        _syncCamera();
      }
    }

    // Saldo: em segundo plano — antes `getBalance` podia pendurar sem timeout em `_get` e o loader nunca parava.
    if (okPayment != null && mounted) {
      unawaited(_syncBalanceAfterPayment(okPayment));
    }
  }

  void _onDetect(BarcodeCapture capture) {
    if (!_cameraTarget ||
        !_autoPayEnabled ||
        !mounted ||
        _paying ||
        _scanBusy) {
      return;
    }
    if (capture.barcodes.isEmpty) return;
    final raw = capture.barcodes.first.rawValue;
    if (raw == null || raw.isEmpty) return;
    HapticFeedback.mediumImpact();
    _scanBusy = true;
    _autoPayFromScan(raw);
  }

  Future<void> _syncBalanceAfterPayment(Map<String, dynamic> paymentJson) async {
    final acc = widget.account;
    if (acc == null) return;
    try {
      final bal = await widget.api.getBalance();
      if (!mounted) return;
      final avail = (bal['available_cents'] as num?)?.toInt() ?? acc.balanceCents;
      final updated = SavedAccount(
        id: acc.id,
        personName: acc.personName,
        bankCode: acc.bankCode,
        bankName: acc.bankName,
        agency: acc.agency,
        accountNumber: acc.accountNumber,
        balanceCents: avail,
      );
      await LocalAccountsStore.instance.updateAccount(updated);
      widget.onPaymentSettled?.call(updated);
    } catch (_) {
      // Saldo local mantém-se; o pagamento já foi persistido no backend.
    }
  }

  String _paymentSummaryLine(Map<String, dynamic> j) {
    final state = _pickStr(j, const ['payment_state', 'State']) ?? '?';
    final cents = _pickCents(j);
    final charge = _pickStr(j, const ['gatebox_charge_id', 'GateboxCharge']) ?? '';
    final money = NumberFormat.currency(locale: 'pt_BR', symbol: r'R$');
    final amt = cents != null ? money.format(cents / 100) : '?';
    return 'Pago: $state · $amt${charge.isNotEmpty ? ' · Gatebox: $charge' : ''}';
  }

  String? _pickStr(Map<String, dynamic> j, List<String> keys) {
    for (final k in keys) {
      final v = j[k];
      if (v != null && '$v'.isNotEmpty) return '$v';
    }
    return null;
  }

  int? _pickCents(Map<String, dynamic> j) {
    for (final k in const ['amount_cents', 'AmountCents']) {
      final v = j[k];
      if (v is num) return v.toInt();
    }
    return null;
  }

  Future<void> _payCurrentReference() async {
    final trimmed = _referenceController.text.trim();
    if (trimmed.isEmpty || _paying) return;
    if (_shouldThrottle(trimmed)) return;

    setState(() => _paying = true);
    _syncCamera();
    Map<String, dynamic>? okPayment;
    try {
      _lastPayRefKey = _refKey(trimmed);
      _lastPayAt = DateTime.now();
      final method = _methodForReference(trimmed);
      final result = await widget.api.pay(
        method: method,
        reference: trimmed,
        simulationState: _sandboxSimulation,
      );
      if (!mounted) return;
      okPayment = result;
      final summary = _paymentSummaryLine(result);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(summary),
          duration: const Duration(seconds: 8),
          backgroundColor: Colors.green.shade800,
        ),
      );
      setState(() {
        _lastPaymentJson = Map<String, dynamic>.from(result);
        _autoPayEnabled = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _lastPaymentJson = null);
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('$e', style: const TextStyle(fontSize: 12)),
          duration: const Duration(seconds: 14),
          behavior: SnackBarBehavior.floating,
        ),
      );
    } finally {
      if (mounted) {
        setState(() => _paying = false);
        _syncCamera();
      }
    }
    if (okPayment != null && mounted) {
      unawaited(_syncBalanceAfterPayment(okPayment));
    }
  }

  @override
  Widget build(BuildContext context) {
    final acc = widget.account;
    if (acc == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            'Selecione uma conta na aba Contas para usar o PIX.',
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      );
    }

    final money = NumberFormat.currency(locale: 'pt_BR', symbol: r'R$');
    final person = acc.personName.trim().isEmpty
        ? 'Sem nome'
        : acc.personName.trim();
    final theme = Theme.of(context);
    final muted = theme.colorScheme.onSurfaceVariant;
    final saldo = money.format(acc.balanceCents / 100);
    final refTrim = _referenceController.text.trim();
    final infoRows = refTrim.isNotEmpty ? _readInfoRows(refTrim) : null;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          'Conta selecionada',
          style: theme.textTheme.labelLarge?.copyWith(color: muted),
        ),
        const SizedBox(height: 4),
        Text(person, style: theme.textTheme.titleLarge),
        const SizedBox(height: 4),
        Text(
          '${acc.bankName} (${acc.bankCode}) · Ag. ${acc.agency} · CC ${acc.accountNumber}',
          style: theme.textTheme.bodyMedium,
        ),
        Text(
          'Saldo local: $saldo',
          style: theme.textTheme.bodySmall?.copyWith(color: muted),
        ),
        const SizedBox(height: 20),
        if (_lastPaymentJson != null) ...[
          Card(
            color: Colors.green.shade50,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Icon(Icons.check_circle, color: Colors.green.shade800),
                      const SizedBox(width: 8),
                      Text(
                        'Último pagamento (backend + Gatebox)',
                        style: theme.textTheme.titleSmall?.copyWith(
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    _paymentSummaryLine(_lastPaymentJson!),
                    style: theme.textTheme.bodyMedium,
                  ),
                  Text(
                    'Id: ${_pickStr(_lastPaymentJson!, const ['id', 'ID']) ?? '-'}',
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                      color: muted,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
        ],
        if (_cameraTarget) ...[
          Row(
            children: [
              Icon(
                _autoPayEnabled ? Icons.auto_mode : Icons.touch_app,
                color: muted,
              ),
              const SizedBox(width: 8),
              Switch.adaptive(
                value: _autoPayEnabled,
                onChanged: (v) {
                  setState(() => _autoPayEnabled = v);
                  WidgetsBinding.instance.addPostFrameCallback(
                    (_) => _syncCamera(),
                  );
                },
              ),
              const Spacer(),
              FilledButton.icon(
                onPressed: (_paying || refTrim.isEmpty)
                    ? null
                    : _payCurrentReference,
                icon: const Icon(Icons.check_circle_outline),
                label: const Text('Pagar agora'),
              ),
            ],
          ),
          const SizedBox(height: 12),
          if (_scannerController != null)
            LayoutBuilder(
              builder: (context, c) {
                final side = _scannerSide(c.maxWidth);
                return Center(
                  child: Card(
                    elevation: 2,
                    clipBehavior: Clip.antiAlias,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: SizedBox(
                      width: side,
                      height: side,
                      child: Stack(
                        fit: StackFit.expand,
                        children: [
                          MobileScanner(
                            controller: _scannerController!,
                            onDetect: _onDetect,
                            errorBuilder: (context, error, child) {
                              return Padding(
                                padding: const EdgeInsets.all(16),
                                child: Center(
                                  child: Text(
                                    'Não foi possível usar a câmera.\n\n$error',
                                    textAlign: TextAlign.center,
                                    style: theme.textTheme.bodyMedium?.copyWith(
                                      color: theme.colorScheme.error,
                                    ),
                                  ),
                                ),
                              );
                            },
                          ),
                          if (_paying)
                            ColoredBox(
                              color: Colors.black54,
                              child: Center(
                                child: Padding(
                                  padding: const EdgeInsets.all(16),
                                  child: Column(
                                    mainAxisSize: MainAxisSize.min,
                                    children: [
                                      const CircularProgressIndicator(
                                        color: Colors.white,
                                      ),
                                      const SizedBox(height: 16),
                                      Text(
                                        'A validar no Gatebox e registar pagamento…',
                                        textAlign: TextAlign.center,
                                        style: theme.textTheme.titleSmall
                                            ?.copyWith(color: Colors.white),
                                      ),
                                      const SizedBox(height: 8),
                                      Text(
                                        'Não feche a app; pode demorar até 45 s se a rede estiver lenta.',
                                        textAlign: TextAlign.center,
                                        style: theme.textTheme.bodySmall
                                            ?.copyWith(color: Colors.white70),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
                );
              },
            )
          else
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 24),
              child: Center(
                child: Text(
                  'A abrir câmera…',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: muted,
                  ),
                ),
              ),
            ),
        ] else ...[
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(
              kIsWeb
                  ? 'Câmera indisponível na web — use a aba Link para colar URL ou referência.'
                  : 'Câmera só em Android/iOS — use a aba Link para colar texto ou URL.',
              style: theme.textTheme.bodySmall?.copyWith(color: muted),
              textAlign: TextAlign.center,
            ),
          ),
        ],
        if (refTrim.isNotEmpty) ...[
          const SizedBox(height: 20),
          Text('Última leitura', style: theme.textTheme.titleMedium),
          const SizedBox(height: 8),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (infoRows != null)
                    ...infoRows.entries.map(
                      (e) => Padding(
                        padding: const EdgeInsets.only(bottom: 6),
                        child: Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            SizedBox(
                              width: 88,
                              child: Text(
                                e.key,
                                style: theme.textTheme.labelMedium?.copyWith(
                                  color: muted,
                                ),
                              ),
                            ),
                            Expanded(
                              child: Text(
                                e.value,
                                style: theme.textTheme.bodyMedium,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  const Divider(height: 20),
                  Text(
                    'Conteúdo bruto',
                    style: theme.textTheme.labelMedium?.copyWith(color: muted),
                  ),
                  const SizedBox(height: 6),
                  SelectableText(
                    refTrim,
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'Pré-visualização: ${_pixPayloadPreview(refTrim)}',
                    style: theme.textTheme.bodySmall?.copyWith(color: muted),
                  ),
                  Align(
                    alignment: Alignment.centerRight,
                    child: TextButton.icon(
                      onPressed: _paying ? null : _clearPixPayload,
                      icon: const Icon(Icons.clear),
                      label: const Text('Limpar'),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ],
    );
  }
}
