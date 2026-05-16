import 'package:app_banco/src/core/card_expiry.dart';
import 'package:app_banco/src/core/card_number_generator.dart';
import 'package:app_banco/src/core/currency_pt_br.dart';
import 'package:app_banco/src/data/local_accounts_store.dart';
import 'package:app_banco/src/data/local_credit_cards_store.dart';
import 'package:app_banco/src/features/credit_card_visual.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:intl/intl.dart';

/// Cartões da conta selecionada: mesma ideia da lista de contas (ListView, selecionar, editar, excluir, alterar limite).
class CardTabScreen extends StatefulWidget {
  const CardTabScreen({super.key, required this.account});

  final SavedAccount? account;

  @override
  State<CardTabScreen> createState() => _CardTabScreenState();
}

class _CardTabScreenState extends State<CardTabScreen> {
  final _money = NumberFormat.currency(locale: 'pt_BR', symbol: r'R$');
  List<SavedCreditCard> _cards = [];
  bool _loading = true;
  String? _selectedCardId;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  @override
  void didUpdateWidget(CardTabScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.account?.id != widget.account?.id) {
      _selectedCardId = null;
      _reload();
    }
  }

  Future<void> _reload() async {
    final acc = widget.account;
    if (acc == null) {
      setState(() {
        _cards = [];
        _loading = false;
        _selectedCardId = null;
      });
      return;
    }
    setState(() => _loading = true);
    final list = await LocalCreditCardsStore.instance.cardsForAccount(acc.id);
    if (!mounted) return;
    if (_selectedCardId != null && !list.any((c) => c.id == _selectedCardId)) {
      _selectedCardId = null;
    }
    setState(() {
      _cards = list;
      _loading = false;
    });
  }

  Future<void> _openAddCard() async {
    final acc = widget.account;
    if (acc == null) return;
    final ok = await Navigator.of(context).push<bool>(
      MaterialPageRoute<bool>(
        builder: (context) => CreditCardFormScreen(account: acc),
      ),
    );
    if (ok == true && mounted) await _reload();
  }

  Future<void> _openEditCard(SavedCreditCard c) async {
    final acc = widget.account;
    if (acc == null) return;
    final ok = await Navigator.of(context).push<bool>(
      MaterialPageRoute<bool>(
        builder: (context) => CreditCardFormScreen(account: acc, existing: c),
      ),
    );
    if (ok == true && mounted) await _reload();
  }

  Future<void> _confirmDelete(SavedCreditCard c) async {
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Remover cartão'),
        content: Text(
          'Excluir cartão final ${c.lastFour} (${CreditCardVisual.brandLabel(c.brandCode)})?',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Excluir'),
          ),
        ],
      ),
    );
    if (ok != true || !mounted) return;
    await LocalCreditCardsStore.instance.deleteCardById(c.id);
    if (_selectedCardId == c.id) _selectedCardId = null;
    await _reload();
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(const SnackBar(content: Text('Cartão removido')));
  }

  SavedCreditCard? get _selectedCard {
    if (_selectedCardId == null) return null;
    try {
      return _cards.firstWhere((c) => c.id == _selectedCardId);
    } catch (_) {
      return null;
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
            'Selecione uma conta na aba Contas para gerenciar cartões.',
            textAlign: TextAlign.center,
            style: theme.textTheme.titleMedium?.copyWith(color: muted),
          ),
        ),
      );
    }

    final person = acc.personName.trim().isEmpty
        ? 'Sem nome'
        : acc.personName.trim();

    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    final selected = _selectedCard;

    return Scaffold(
      body: RefreshIndicator(
        onRefresh: _reload,
        child: ListView(
          physics: const AlwaysScrollableScrollPhysics(),
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 100),
          children: [
            Text(
              'Conta selecionada',
              style: theme.textTheme.labelLarge?.copyWith(color: muted),
            ),
            const SizedBox(height: 4),
            Text(person, style: theme.textTheme.titleLarge),
            const SizedBox(height: 4),
            Text(
              '${acc.bankName} · Ag. ${acc.agency}',
              style: theme.textTheme.bodyMedium,
            ),
            const SizedBox(height: 8),
            Text(
              'Mesmo fluxo das contas: toque no cartão para selecionar; menu ⋮ para editar limite e bandeira ou excluir.',
              style: theme.textTheme.bodySmall?.copyWith(color: muted),
            ),
            const SizedBox(height: 16),
            if (selected != null) ...[
              Text(
                'Cartão selecionado',
                style: theme.textTheme.labelLarge?.copyWith(color: muted),
              ),
              const SizedBox(height: 8),
              FlippableCreditCard(
                holderName: person,
                brandCode: selected.brandCode,
                creditLimitCents: selected.creditLimitCents,
                cvc: selected.cvc,
                expiryDisplay: selected.expirySlashDisplay,
                panDigits: selected.panDigits,
                maskedFallback: selected.maskedPan,
              ),
              const SizedBox(height: 6),
              Text(
                'Toque no cartão para girar e ver o CVC no verso.',
                style: theme.textTheme.bodySmall?.copyWith(color: muted),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 20),
            ],
            if (_cards.isEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 48),
                child: Center(
                  child: Text(
                    'Nenhum cartão cadastrado.\nToque em + para adicionar número, bandeira e limite.',
                    textAlign: TextAlign.center,
                    style: theme.textTheme.bodyLarge?.copyWith(color: muted),
                  ),
                ),
              )
            else
              ..._cards.map(
                (c) => _CreditCardTile(
                  card: c,
                  money: _money,
                  selected: _selectedCardId == c.id,
                  onSelect: () => setState(() => _selectedCardId = c.id),
                  onEdit: () => _openEditCard(c),
                  onDelete: () => _confirmDelete(c),
                ),
              ),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _openAddCard,
        icon: const Icon(Icons.add_card),
        label: const Text('Novo cartão'),
      ),
    );
  }
}

class _CreditCardTile extends StatelessWidget {
  const _CreditCardTile({
    required this.card,
    required this.money,
    required this.selected,
    required this.onSelect,
    required this.onEdit,
    required this.onDelete,
  });

  final SavedCreditCard card;
  final NumberFormat money;
  final bool selected;
  final VoidCallback onSelect;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final brand = CreditCardVisual.brandLabel(card.brandCode);
    return Card(
      margin: const EdgeInsets.only(bottom: 10),
      elevation: selected ? 2 : 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(
          color: selected ? scheme.primary : scheme.outlineVariant,
          width: selected ? 2 : 1,
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 8, 4, 8),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            CircleAvatar(
              backgroundColor: selected ? scheme.primaryContainer : null,
              foregroundColor: selected ? scheme.onPrimaryContainer : null,
              child: FittedBox(
                fit: BoxFit.scaleDown,
                child: Padding(
                  padding: const EdgeInsets.all(4),
                  child: Text(
                    card.lastFour,
                    style: const TextStyle(
                      fontWeight: FontWeight.w700,
                      fontSize: 12,
                    ),
                  ),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: InkWell(
                onTap: onSelect,
                borderRadius: BorderRadius.circular(8),
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    vertical: 4,
                    horizontal: 4,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '$brand · ${card.maskedPan}',
                        style: Theme.of(context).textTheme.titleMedium
                            ?.copyWith(fontWeight: FontWeight.w600),
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 2),
                      Text(
                        'Validade ${card.expirySlashDisplay}',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: scheme.onSurfaceVariant,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        'Limite de crédito',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: scheme.onSurfaceVariant,
                        ),
                      ),
                      const SizedBox(height: 6),
                      Text(
                        selected
                            ? 'Cartão selecionado — pré-visualização acima'
                            : 'Toque para selecionar este cartão',
                        style: Theme.of(context).textTheme.labelSmall?.copyWith(
                          color: selected
                              ? scheme.primary
                              : scheme.onSurfaceVariant,
                          fontWeight: selected ? FontWeight.w600 : null,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Padding(
                  padding: const EdgeInsets.only(top: 4, right: 4),
                  child: Text(
                    money.format(card.creditLimitCents / 100),
                    style: Theme.of(context).textTheme.titleSmall?.copyWith(
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                PopupMenuButton<String>(
                  icon: const Icon(Icons.more_vert),
                  onSelected: (value) {
                    if (value == 'edit') onEdit();
                    if (value == 'delete') onDelete();
                  },
                  itemBuilder: (context) => [
                    const PopupMenuItem(
                      value: 'edit',
                      child: ListTile(
                        leading: Icon(Icons.edit),
                        title: Text('Editar'),
                      ),
                    ),
                    const PopupMenuItem(
                      value: 'delete',
                      child: ListTile(
                        leading: Icon(Icons.delete_outline, color: Colors.red),
                        title: Text('Excluir'),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

/// Novo cartão ou edição: número e CVC editáveis (pré-preenchidos); botões «Gerar número» e «Gerar CVC» na mesma linha.
class CreditCardFormScreen extends StatefulWidget {
  const CreditCardFormScreen({super.key, required this.account, this.existing});

  final SavedAccount account;
  final SavedCreditCard? existing;

  @override
  State<CreditCardFormScreen> createState() => _CreditCardFormScreenState();
}

class _CreditCardFormScreenState extends State<CreditCardFormScreen> {
  late final TextEditingController _limitController;
  late final TextEditingController _panController;
  late final TextEditingController _cvcController;
  late final TextEditingController _expiryController;
  String _brand = 'VISA';

  static const _brands = [
    ('VISA', 'Visa'),
    ('MASTERCARD', 'Mastercard'),
    ('ELO', 'Elo'),
    ('AMEX', 'American Express'),
    ('HIPERCARD', 'Hipercard'),
    ('OUTRO', 'Outro'),
  ];

  bool get _isEdit => widget.existing != null;

  String _panDigitsFromController() =>
      _panController.text.replaceAll(RegExp(r'\D'), '');
  String _cvcFromController() =>
      _cvcController.text.replaceAll(RegExp(r'\D'), '');

  String _expiryDisplayForPreview() {
    final d = CardExpiry.digitsOnly(_expiryController.text);
    if (d.isEmpty) return '••/••';
    return CardExpiry.formatSlash(d);
  }

  void _regeneratePanOnly() {
    setState(() {
      _panController.text = CardNumberGenerator.generatePan(_brand);
    });
  }

  void _regenerateCvcOnly() {
    setState(() {
      _cvcController.text = CardNumberGenerator.generateCvc();
    });
  }

  @override
  void initState() {
    super.initState();
    _panController = TextEditingController();
    _cvcController = TextEditingController();
    final e = widget.existing;
    if (e != null) {
      _brand = e.brandCode;
      _limitController = TextEditingController(
        text: formatCentsToCurrencyField(e.creditLimitCents),
      );
      _expiryController = TextEditingController(
        text: CardExpiry.formatSlash(e.expiryMmYy),
      );
      final pd = e.panDigits?.replaceAll(RegExp(r'\D'), '') ?? '';
      if (pd.length >= 13) {
        _panController.text = pd;
        final cv = e.cvc.replaceAll(RegExp(r'\D'), '');
        _cvcController.text = (cv.length >= 3 && cv.length <= 4)
            ? cv
            : CardNumberGenerator.generateCvc();
      } else {
        _panController.text = CardNumberGenerator.generatePan(_brand);
        _cvcController.text = CardNumberGenerator.generateCvc();
      }
    } else {
      _limitController = TextEditingController(text: r'R$ 5.000,00');
      _expiryController = TextEditingController(
        text: CardExpiry.formatSlash(CardExpiry.randomFutureMmYy()),
      );
      _panController.text = CardNumberGenerator.generatePan(_brand);
      _cvcController.text = CardNumberGenerator.generateCvc();
    }
  }

  @override
  void dispose() {
    _limitController.dispose();
    _panController.dispose();
    _cvcController.dispose();
    _expiryController.dispose();
    super.dispose();
  }

  String get _maskedFallback {
    final d = _panDigitsFromController();
    if (d.length >= 4) {
      final last = d.substring(d.length - 4);
      return '•••• •••• •••• $last';
    }
    return '•••• •••• •••• 0000';
  }

  Future<void> _save() async {
    final limitCents = parseCurrencyToCents(_limitController.text);
    if (limitCents <= 0) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Limite de crédito deve ser maior que zero.'),
        ),
      );
      return;
    }
    final digits = _panDigitsFromController();
    if (digits.length < 13 ||
        digits.length > 19 ||
        !CardNumberGenerator.luhnValid(digits)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text(
            'Número inválido (13–19 dígitos e Luhn). Ajuste ou use «Gerar número».',
          ),
        ),
      );
      return;
    }
    if (!CardNumberGenerator.panMatchesBrand(digits, _brand)) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            'O número não corresponde à bandeira ${CreditCardVisual.brandLabel(_brand)} (IIN). '
            'Altere o número ou a bandeira, ou use «Gerar número».',
          ),
        ),
      );
      return;
    }
    final cvc = _cvcFromController();
    if (cvc.length < 3 || cvc.length > 4) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('CVC deve ter 3 ou 4 dígitos.')),
      );
      return;
    }
    final expiryDigits = CardExpiry.digitsOnly(_expiryController.text);
    if (expiryDigits.length != 4) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Validade incompleta. Use MM/AA (4 dígitos).'),
        ),
      );
      return;
    }
    if (!CardExpiry.isValid(expiryDigits)) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text(
            'Validade inválida: mês entre 01 e 12 e data não pode estar no passado.',
          ),
        ),
      );
      return;
    }
    final lastFour = digits.substring(digits.length - 4);

    if (_isEdit) {
      final e = widget.existing!;
      await LocalCreditCardsStore.instance.updateCard(
        SavedCreditCard(
          id: e.id,
          accountId: e.accountId,
          lastFour: lastFour,
          panDigits: digits,
          cvc: cvc,
          brandCode: _brand,
          creditLimitCents: limitCents,
          expiryMmYy: expiryDigits,
        ),
      );
    } else {
      await LocalCreditCardsStore.instance.addCard(
        SavedCreditCard(
          id: DateTime.now().microsecondsSinceEpoch.toString(),
          accountId: widget.account.id,
          lastFour: lastFour,
          panDigits: digits,
          cvc: cvc,
          brandCode: _brand,
          creditLimitCents: limitCents,
          expiryMmYy: expiryDigits,
        ),
      );
    }
    if (!mounted) return;
    Navigator.of(context).pop(true);
  }

  @override
  Widget build(BuildContext context) {
    final person = widget.account.personName.trim().isEmpty
        ? 'Sem nome'
        : widget.account.personName.trim();
    final theme = Theme.of(context);
    final panDigits = _panDigitsFromController();
    final cvcText = _cvcFromController();

    final keyboardInset = MediaQuery.viewInsetsOf(context).bottom;
    final safeBottom = MediaQuery.viewPaddingOf(context).bottom;

    return Scaffold(
      appBar: AppBar(title: Text(_isEdit ? 'Editar cartão' : 'Novo cartão')),
      body: ListView(
        padding: EdgeInsets.fromLTRB(
          12,
          12,
          12,
          12 + keyboardInset + safeBottom,
        ),
        children: [
          FlippableCreditCard(
            holderName: person,
            brandCode: _brand,
            creditLimitCents: parseCurrencyToCents(_limitController.text),
            cvc: cvcText.length >= 3 ? cvcText : '000',
            expiryDisplay: _expiryDisplayForPreview(),
            panDigits: panDigits.isNotEmpty ? panDigits : null,
            maskedFallback: _maskedFallback,
          ),
          const SizedBox(height: 4),
          Text(
            'Toque no cartão para virar e ver o verso.',
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 10),
          Text('Bandeira', style: theme.textTheme.labelLarge),
          const SizedBox(height: 6),
          LayoutBuilder(
            builder: (context, constraints) {
              const crossCount = 3;
              const spacing = 6.0;
              const rowHeight = 44.0;
              final maxWRaw = constraints.maxWidth.isFinite
                  ? constraints.maxWidth
                  : MediaQuery.sizeOf(context).width - 24;
              // tileW positivo; sem clamp artificial no aspect (altura = largura/ratio).
              final maxW = maxWRaw.clamp(120.0, double.infinity);
              final tileW = ((maxW - spacing * (crossCount - 1)) / crossCount)
                  .clamp(1.0, double.infinity);
              final aspect = tileW / rowHeight;
              final rowCount = (_brands.length + crossCount - 1) ~/ crossCount;
              final gridHeight =
                  rowCount * rowHeight + (rowCount - 1) * spacing;
              return SizedBox(
                height: gridHeight,
                child: GridView.builder(
                  shrinkWrap: true,
                  physics: const NeverScrollableScrollPhysics(),
                  itemCount: _brands.length,
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: crossCount,
                    crossAxisSpacing: spacing,
                    mainAxisSpacing: spacing,
                    childAspectRatio: aspect,
                  ),
                  itemBuilder: (context, index) {
                    final e = _brands[index];
                    final sel = _brand == e.$1;
                    return FilterChip(
                      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      visualDensity: VisualDensity.compact,
                      showCheckmark: false,
                      padding: const EdgeInsets.symmetric(
                        horizontal: 4,
                        vertical: 0,
                      ),
                      label: Center(
                        child: FittedBox(
                          fit: BoxFit.scaleDown,
                          child: Text(
                            e.$2,
                            maxLines: 2,
                            textAlign: TextAlign.center,
                            style: theme.textTheme.labelMedium,
                          ),
                        ),
                      ),
                      selected: sel,
                      onSelected: (_) {
                        setState(() {
                          _brand = e.$1;
                          _panController.text = CardNumberGenerator.generatePan(
                            _brand,
                          );
                        });
                      },
                    );
                  },
                ),
              );
            },
          ),
          const SizedBox(height: 8),
          TextFormField(
            controller: _panController,
            keyboardType: TextInputType.number,
            inputFormatters: [
              FilteringTextInputFormatter.digitsOnly,
              LengthLimitingTextInputFormatter(19),
            ],
            decoration: const InputDecoration(
              labelText: 'Número do cartão',
              hintText: 'Somente dígitos',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.numbers),
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 10),
          TextFormField(
            controller: _expiryController,
            keyboardType: TextInputType.number,
            inputFormatters: [const ExpiryMmYyInputFormatter()],
            decoration: const InputDecoration(
              labelText: 'Validade (MM/AA)',
              hintText: 'MM/AA',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.date_range),
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 10),
          TextFormField(
            controller: _cvcController,
            keyboardType: TextInputType.number,
            inputFormatters: [
              FilteringTextInputFormatter.digitsOnly,
              LengthLimitingTextInputFormatter(4),
            ],
            decoration: const InputDecoration(
              labelText: 'CVC (código de segurança)',
              hintText: '3 ou 4 dígitos',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.password),
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 8),
          LayoutBuilder(
            builder: (context, constraints) {
              const gap = 8.0;
              const h = 44.0;
              final half = ((constraints.maxWidth - gap) / 2).clamp(
                64.0,
                double.infinity,
              );
              final btnStyle = OutlinedButton.styleFrom(
                minimumSize: Size(half, h),
                maximumSize: Size(half, h),
                fixedSize: Size(half, h),
                padding: const EdgeInsets.symmetric(horizontal: 6),
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              );
              return Row(
                children: [
                  SizedBox(
                    width: half,
                    height: h,
                    child: OutlinedButton(
                      onPressed: _regeneratePanOnly,
                      style: btnStyle,
                      child: const FittedBox(
                        fit: BoxFit.scaleDown,
                        child: Text(
                          'Gerar número',
                          maxLines: 1,
                          textAlign: TextAlign.center,
                        ),
                      ),
                    ),
                  ),
                  const SizedBox(width: gap),
                  SizedBox(
                    width: half,
                    height: h,
                    child: OutlinedButton(
                      onPressed: _regenerateCvcOnly,
                      style: btnStyle,
                      child: const FittedBox(
                        fit: BoxFit.scaleDown,
                        child: Text(
                          'Gerar CVC',
                          maxLines: 1,
                          textAlign: TextAlign.center,
                        ),
                      ),
                    ),
                  ),
                ],
              );
            },
          ),
          const SizedBox(height: 12),
          TextFormField(
            controller: _limitController,
            keyboardType: TextInputType.number,
            inputFormatters: [
              FilteringTextInputFormatter.digitsOnly,
              CurrencyPtBrInputFormatter(),
            ],
            decoration: const InputDecoration(
              labelText: 'Limite de crédito',
              border: OutlineInputBorder(),
              prefixIcon: Icon(Icons.trending_up),
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 14),
          SizedBox(
            width: double.infinity,
            child: FilledButton.icon(
              onPressed: _save,
              style: FilledButton.styleFrom(
                minimumSize: const Size.fromHeight(44),
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                visualDensity: VisualDensity.compact,
              ),
              icon: const Icon(Icons.save),
              label: Text(_isEdit ? 'Salvar alterações' : 'Salvar cartão'),
            ),
          ),
          SizedBox(height: 32 + MediaQuery.sizeOf(context).height * 0.02),
        ],
      ),
    );
  }
}
