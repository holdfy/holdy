import 'package:app_banco/src/core/banco_api_client.dart';
import 'package:app_banco/src/data/local_accounts_store.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Aba Link: pagamento `POST /payments/link` para a conta **selecionada** na aba Contas.
class LinkTabScreen extends StatefulWidget {
  const LinkTabScreen({super.key, required this.account, required this.api});

  final SavedAccount? account;
  final BancoApiClient api;

  @override
  State<LinkTabScreen> createState() => _LinkTabScreenState();
}

class _LinkTabScreenState extends State<LinkTabScreen> {
  final _referenceController = TextEditingController();
  bool _paying = false;

  static const _sandboxSimulation = 'APPROVED';

  @override
  void dispose() {
    _referenceController.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(LinkTabScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.account?.id != widget.account?.id) {
      _referenceController.clear();
    }
  }

  Future<void> _pasteFromClipboard() async {
    if (_paying) return;
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final text = data?.text?.trim();
    if (!mounted) return;
    if (text == null || text.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Área de transferência vazia.')),
      );
      return;
    }
    setState(() => _referenceController.text = text);
    HapticFeedback.lightImpact();
    ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Texto colado no campo.')));
  }

  Future<void> _pay() async {
    if (widget.account == null) return;
    final ref = _referenceController.text.trim();
    if (ref.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Informe o link ou a referência da cobrança.')),
      );
      return;
    }
    setState(() => _paying = true);
    try {
      await widget.api.pay(method: 'link', reference: ref, simulationState: _sandboxSimulation);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Pagamento enviado.')));
      _referenceController.clear();
      setState(() {});
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
    } finally {
      if (mounted) setState(() => _paying = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final muted = theme.colorScheme.onSurfaceVariant;
    final acc = widget.account;

    if (acc == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            'Selecione uma conta na aba Contas para pagar com link.',
            textAlign: TextAlign.center,
            style: theme.textTheme.titleMedium?.copyWith(color: muted),
          ),
        ),
      );
    }

    final person = acc.personName.trim().isEmpty ? 'Sem nome' : acc.personName.trim();

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text('Conta selecionada', style: theme.textTheme.labelLarge?.copyWith(color: muted)),
        const SizedBox(height: 4),
        Text(person, style: theme.textTheme.titleLarge),
        const SizedBox(height: 4),
        Text('${acc.bankName} · Ag. ${acc.agency} · CC ${acc.accountNumber}', style: theme.textTheme.bodyMedium),
        const SizedBox(height: 20),
        Text(
          'Cole a URL, o identificador ou qualquer texto copiado (área de transferência) no campo abaixo.',
          style: theme.textTheme.bodySmall?.copyWith(color: muted),
        ),
        const SizedBox(height: 12),
        TextField(
          controller: _referenceController,
          maxLines: 5,
          keyboardType: TextInputType.url,
          decoration: const InputDecoration(
            labelText: 'Link ou referência',
            hintText: 'https://… ou id da cobrança',
            border: OutlineInputBorder(),
            alignLabelWithHint: true,
          ),
        ),
        const SizedBox(height: 10),
        Align(
          alignment: Alignment.centerLeft,
          child: OutlinedButton.icon(
            onPressed: _paying ? null : _pasteFromClipboard,
            icon: const Icon(Icons.content_paste),
            label: const Text('Colar da área de transferência'),
          ),
        ),
        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: _paying ? null : _pay,
          icon: _paying
              ? const SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                )
              : const Icon(Icons.link_rounded),
          label: Text(_paying ? 'Processando...' : 'Pagar com este link'),
        ),
      ],
    );
  }
}
