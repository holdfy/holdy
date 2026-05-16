import 'package:app_banco/src/core/banco_api_client.dart';
import 'package:app_banco/src/core/banco_api_config.dart';
import 'package:app_banco/src/core/currency_pt_br.dart';
import 'package:app_banco/src/data/br_banks.dart';
import 'package:app_banco/src/data/local_accounts_store.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:http/http.dart' as http;
import 'package:intl/intl.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<BrazilianBank?> pickBrazilianBank(BuildContext context) {
  return showModalBottomSheet<BrazilianBank>(
    context: context,
    isScrollControlled: true,
    showDragHandle: true,
    builder: (ctx) => DraggableScrollableSheet(
      expand: false,
      initialChildSize: 0.72,
      minChildSize: 0.4,
      maxChildSize: 0.92,
      builder: (context, scrollController) => _BrazilianBankPickerSheet(scrollController: scrollController),
    ),
  );
}

class HomeScreen extends StatefulWidget {
  const HomeScreen({
    super.key,
    required this.api,
    this.selectedAccount,
    required this.onAccountSelected,
    required this.onAccountDeleted,
  });

  final BancoApiClient api;
  final SavedAccount? selectedAccount;
  final ValueChanged<SavedAccount> onAccountSelected;
  final ValueChanged<String> onAccountDeleted;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final _money = NumberFormat.currency(locale: 'pt_BR', symbol: r'R$');
  List<SavedAccount> _accounts = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  Future<void> _reload() async {
    setState(() => _loading = true);
    final list = await LocalAccountsStore.instance.loadAccounts();
    if (!mounted) return;
    setState(() {
      _accounts = list;
      _loading = false;
    });
  }

  Future<void> _openAddAccount() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(builder: (context) => const AddAccountScreen()),
    );
    if (!mounted) return;
    await _reload();
  }

  Future<void> _openEditAccount(SavedAccount a) async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(builder: (context) => AddAccountScreen(existing: a)),
    );
    if (!mounted) return;
    await _reload();
  }

  Future<void> _confirmDelete(SavedAccount a) async {
    final displayName = a.personName.trim().isEmpty ? 'Sem nome' : a.personName.trim();
    final ok = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Excluir conta'),
        content: Text('Remover a conta de $displayName no ${a.bankName}?'),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('Cancelar')),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Excluir'),
          ),
        ],
      ),
    );
    if (ok != true || !mounted) return;
    await LocalAccountsStore.instance.deleteAccountById(a.id);
    widget.onAccountDeleted(a.id);
    await _reload();
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Conta removida')));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: RefreshIndicator(
        onRefresh: _reload,
        child: _loading
            ? ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                children: const [SizedBox(height: 120), Center(child: CircularProgressIndicator())],
              )
            : ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                padding: const EdgeInsets.fromLTRB(16, 16, 16, 88),
                children: [
                  Text('Manager Account', style: Theme.of(context).textTheme.headlineSmall),
                  const SizedBox(height: 16),
                  if (_accounts.isEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 48),
                      child: Center(
                        child: Text(
                          'Nenhuma conta cadastrada.\nToque em + para adicionar nome, banco, agência, conta e saldo.',
                          textAlign: TextAlign.center,
                          style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                                color: Theme.of(context).colorScheme.onSurfaceVariant,
                              ),
                        ),
                      ),
                    )
                  else
                    ..._accounts.map(
                      (a) => _AccountTile(
                        account: a,
                        money: _money,
                        selected: widget.selectedAccount?.id == a.id,
                        onSelect: () => widget.onAccountSelected(a),
                        onEdit: () => _openEditAccount(a),
                        onDelete: () => _confirmDelete(a),
                      ),
                    ),
                ],
              ),
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _openAddAccount,
        icon: const Icon(Icons.add),
        label: const Text('Nova conta'),
      ),
    );
  }
}

class _AccountTile extends StatelessWidget {
  const _AccountTile({
    required this.account,
    required this.money,
    required this.selected,
    required this.onSelect,
    required this.onEdit,
    required this.onDelete,
  });

  final SavedAccount account;
  final NumberFormat money;
  final bool selected;
  final VoidCallback onSelect;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final personLabel = account.personName.trim().isEmpty ? 'Sem nome' : account.personName.trim();
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
              child: Text(account.bankCode),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: InkWell(
                onTap: onSelect,
                borderRadius: BorderRadius.circular(8),
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 4),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        personLabel,
                        style: Theme.of(context).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 4),
                      Text(
                        '${account.bankName} (${account.bankCode})',
                        style: Theme.of(context).textTheme.bodyMedium,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 2),
                      Text(
                        'Agência ${account.agency} · Conta ${account.accountNumber}',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: Theme.of(context).colorScheme.onSurfaceVariant,
                            ),
                      ),
                      const SizedBox(height: 6),
                      Text(
                        selected ? 'Conta selecionada — use as abas PIX ou Link' : 'Toque para selecionar esta conta',
                        style: Theme.of(context).textTheme.labelSmall?.copyWith(
                              color: selected ? scheme.primary : scheme.onSurfaceVariant,
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
                    money.format(account.balanceCents / 100),
                    style: Theme.of(context).textTheme.titleSmall?.copyWith(fontWeight: FontWeight.w700),
                  ),
                ),
                PopupMenuButton<String>(
                  icon: const Icon(Icons.more_vert),
                  onSelected: (value) {
                    if (value == 'edit') onEdit();
                    if (value == 'delete') onDelete();
                  },
                  itemBuilder: (context) => [
                    const PopupMenuItem(value: 'edit', child: ListTile(leading: Icon(Icons.edit), title: Text('Editar'))),
                    const PopupMenuItem(
                      value: 'delete',
                      child: ListTile(leading: Icon(Icons.delete_outline, color: Colors.red), title: Text('Excluir')),
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

class AddAccountScreen extends StatefulWidget {
  const AddAccountScreen({super.key, this.existing});

  /// Se preenchido, a tela funciona como edição e preserva o [SavedAccount.id].
  final SavedAccount? existing;

  @override
  State<AddAccountScreen> createState() => _AddAccountScreenState();
}

class _AddAccountScreenState extends State<AddAccountScreen> {
  final _formKey = GlobalKey<FormState>();
  final _personNameController = TextEditingController();
  final _agencyController = TextEditingController();
  final _accountController = TextEditingController();
  final _initialBalanceController = TextEditingController(text: r'R$ 0,00');
  BrazilianBank? _bank;

  bool get _isEdit => widget.existing != null;

  @override
  void initState() {
    super.initState();
    final e = widget.existing;
    if (e != null) {
      _personNameController.text = e.personName;
      _agencyController.text = e.agency;
      _accountController.text = e.accountNumber;
      _initialBalanceController.text = formatCentsToCurrencyField(e.balanceCents);
      _bank = BrazilianBank(e.bankCode, e.bankName);
    }
  }

  @override
  void dispose() {
    _personNameController.dispose();
    _agencyController.dispose();
    _accountController.dispose();
    _initialBalanceController.dispose();
    super.dispose();
  }

  Future<void> _selectBank() async {
    final chosen = await pickBrazilianBank(context);
    if (chosen != null) setState(() => _bank = chosen);
  }

  Future<void> _save() async {
    if (!_formKey.currentState!.validate()) return;
    if (_bank == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Selecione o banco na lista de códigos COMPE.')),
      );
      return;
    }
    final balanceCents = parseCurrencyToCents(_initialBalanceController.text);
    final account = SavedAccount(
      id: widget.existing?.id ?? DateTime.now().microsecondsSinceEpoch.toString(),
      personName: _personNameController.text.trim(),
      bankCode: _bank!.code,
      bankName: _bank!.name,
      agency: _agencyController.text.trim(),
      accountNumber: _accountController.text.trim(),
      balanceCents: balanceCents,
    );
    if (_isEdit) {
      await LocalAccountsStore.instance.updateAccount(account);
    } else {
      await LocalAccountsStore.instance.addAccount(account);
    }
    if (!mounted) return;
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(_isEdit ? 'Editar conta' : 'Nova conta')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            _isEdit
                ? 'Altere nome, banco, agência, conta ou saldo. Toque em Salvar para voltar à lista.'
                : 'Informe nome da pessoa, banco (COMPE), agência, conta corrente e saldo inicial.',
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(color: Theme.of(context).colorScheme.onSurfaceVariant),
          ),
          const SizedBox(height: 20),
          Form(
            key: _formKey,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                TextFormField(
                  controller: _personNameController,
                  textCapitalization: TextCapitalization.words,
                  decoration: const InputDecoration(
                    labelText: 'Nome da pessoa',
                    hintText: 'Titular da conta',
                    border: OutlineInputBorder(),
                  ),
                  validator: (v) {
                    if ((v?.trim() ?? '').isEmpty) return 'Informe o nome da pessoa';
                    return null;
                  },
                ),
                const SizedBox(height: 16),
                Card(
                  child: InkWell(
                    onTap: _selectBank,
                    borderRadius: BorderRadius.circular(12),
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: Row(
                        children: [
                          Icon(Icons.account_balance, color: Theme.of(context).colorScheme.primary),
                          const SizedBox(width: 12),
                          Expanded(
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Text('Banco', style: Theme.of(context).textTheme.labelLarge),
                                const SizedBox(height: 4),
                                Text(
                                  _bank?.label ?? 'Toque para escolher na lista de bancos do Brasil',
                                  style: Theme.of(context).textTheme.bodyLarge,
                                ),
                              ],
                            ),
                          ),
                          const Icon(Icons.chevron_right),
                        ],
                      ),
                    ),
                  ),
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _agencyController,
                  decoration: const InputDecoration(
                    labelText: 'Agência',
                    hintText: 'Somente números',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                    LengthLimitingTextInputFormatter(5),
                  ],
                  validator: (v) {
                    final t = v?.trim() ?? '';
                    if (t.isEmpty) return 'Informe a agência';
                    if (t.length > 5) return 'Agência deve ter até 5 dígitos';
                    return null;
                  },
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _accountController,
                  decoration: const InputDecoration(
                    labelText: 'Conta corrente',
                    hintText: 'Número e dígito (ex.: 12345-6 ou 123456)',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.text,
                  inputFormatters: [
                    FilteringTextInputFormatter.allow(RegExp(r'[0-9Xx\-]')),
                    LengthLimitingTextInputFormatter(14),
                  ],
                  validator: (v) {
                    final t = v?.trim() ?? '';
                    if (t.isEmpty) return 'Informe a conta corrente';
                    final digits = t.replaceAll(RegExp(r'[^0-9]'), '');
                    if (digits.length < 3) return 'Conta muito curta';
                    return null;
                  },
                ),
                const SizedBox(height: 16),
                TextFormField(
                  controller: _initialBalanceController,
                  decoration: InputDecoration(
                    labelText: _isEdit ? 'Saldo' : 'Saldo inicial',
                    hintText: 'Valor em reais',
                    border: const OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.digitsOnly,
                    CurrencyPtBrInputFormatter(),
                  ],
                  validator: (_) => null,
                ),
                const SizedBox(height: 28),
                FilledButton.icon(
                  onPressed: _save,
                  icon: const Icon(Icons.save),
                  label: Text(_isEdit ? 'Salvar alterações' : 'Salvar conta'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _BrazilianBankPickerSheet extends StatefulWidget {
  const _BrazilianBankPickerSheet({required this.scrollController});

  final ScrollController scrollController;

  @override
  State<_BrazilianBankPickerSheet> createState() => _BrazilianBankPickerSheetState();
}

class _BrazilianBankPickerSheetState extends State<_BrazilianBankPickerSheet> {
  final _searchController = TextEditingController();
  List<BrazilianBank> _options = kBrazilianBanks.take(20).toList();

  @override
  void initState() {
    super.initState();
    _searchController.addListener(_onSearch);
  }

  @override
  void dispose() {
    _searchController.removeListener(_onSearch);
    _searchController.dispose();
    super.dispose();
  }

  void _onSearch() {
    setState(() => _options = filterBrazilianBanks(_searchController.text));
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
          child: Text('Código COMPE — instituição', style: Theme.of(context).textTheme.titleMedium),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: TextField(
            controller: _searchController,
            decoration: const InputDecoration(
              prefixIcon: Icon(Icons.search),
              hintText: 'Código ou nome do banco',
              border: OutlineInputBorder(),
            ),
            autofocus: true,
          ),
        ),
        const SizedBox(height: 8),
        Expanded(
          child: ListView.builder(
            controller: widget.scrollController,
            itemCount: _options.length,
            itemBuilder: (context, index) {
              final b = _options[index];
              return ListTile(
                title: Text(b.code, style: const TextStyle(fontWeight: FontWeight.w600)),
                subtitle: Text(b.name, maxLines: 2, overflow: TextOverflow.ellipsis),
                onTap: () => Navigator.pop(context, b),
              );
            },
          ),
        ),
      ],
    );
  }
}

class AddFundsScreen extends StatefulWidget {
  const AddFundsScreen({super.key, required this.api});
  final BancoApiClient api;

  @override
  State<AddFundsScreen> createState() => _AddFundsScreenState();
}

class _AddFundsScreenState extends State<AddFundsScreen> {
  final amountController = TextEditingController(text: 'R\$ 0,00');
  final noteController = TextEditingController();
  final targetAgencyController = TextEditingController();
  final targetAccountController = TextEditingController();
  final targetDocumentController = TextEditingController();
  final pixCopyPasteController = TextEditingController(
    text: '00020126580014BR.GOV.BCB.PIX0136pix-demo@saczuck.bank5204000053039865406100.005802BR5920BANCO SACZUCK SANDBOX6009SAO PAULO62070503***6304ABCD',
  );
  String entryType = 'deposito';
  String targetPersonType = 'PF';
  String pixKey = 'pix-demo@saczuck.bank';
  int availableCents = 0;
  int blockedCents = 0;
  bool loading = true;
  String? error;

  @override
  void initState() {
    super.initState();
    _refreshBalance();
  }

  Future<void> _refreshBalance() async {
    setState(() {
      loading = true;
      error = null;
    });
    try {
      final balance = await widget.api.getBalance();
      final me = await widget.api.getMe();
      setState(() {
        availableCents = (balance['available_cents'] as num?)?.toInt() ?? 0;
        blockedCents = (balance['blocked_cents'] as num?)?.toInt() ?? 0;
        pixKey = (me['pix_key'] as String?) ?? pixKey;
        loading = false;
      });
    } catch (e) {
      setState(() {
        loading = false;
        error = e.toString();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final money = (availableCents / 100).toStringAsFixed(2);
    final blocked = (blockedCents / 100).toStringAsFixed(2);
    return Padding(
      padding: const EdgeInsets.all(16),
      child: ListView(
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: loading
                  ? const Center(child: CircularProgressIndicator())
                  : Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Saldo disponivel: R\$ $money', style: Theme.of(context).textTheme.titleLarge),
                        const SizedBox(height: 8),
                        Text('Saldo bloqueado: R\$ $blocked'),
                        if (error != null) ...[
                          const SizedBox(height: 8),
                          Text('Erro: $error'),
                        ],
                        const SizedBox(height: 8),
                        OutlinedButton(onPressed: _refreshBalance, child: const Text('Atualizar saldo')),
                      ],
                    ),
            ),
          ),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Receber via PIX (copia e cola)', style: Theme.of(context).textTheme.titleMedium),
                  const SizedBox(height: 8),
                  TextField(
                    controller: pixCopyPasteController,
                    maxLines: 3,
                    readOnly: true,
                    decoration: const InputDecoration(border: OutlineInputBorder()),
                  ),
                  const SizedBox(height: 8),
                  Text('Chave PIX: $pixKey'),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Receber via QR Code PIX', style: Theme.of(context).textTheme.titleMedium),
                  const SizedBox(height: 8),
                  const Center(
                    child: Icon(Icons.qr_code_2, size: 120),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'Sandbox: o QR representa a chave PIX de recebimento desta conta.',
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Depositar saldo', style: Theme.of(context).textTheme.titleMedium),
                  const SizedBox(height: 8),
                  TextField(
                    controller: amountController,
                    keyboardType: TextInputType.number,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly, CurrencyPtBrInputFormatter()],
                    decoration: const InputDecoration(labelText: 'Valor (R\$)'),
                  ),
                  DropdownButton<String>(
                    value: entryType,
                    items: const [
                      DropdownMenuItem(value: 'deposito', child: Text('Deposito')),
                      DropdownMenuItem(value: 'pix_recebido', child: Text('PIX recebido')),
                      DropdownMenuItem(value: 'ajuste_manual', child: Text('Ajuste manual')),
                    ],
                    onChanged: (value) => setState(() => entryType = value ?? 'deposito'),
                  ),
                  TextField(controller: noteController, decoration: const InputDecoration(labelText: 'Observacao')),
                  const SizedBox(height: 8),
                  Text('Conta destino (opcional)', style: Theme.of(context).textTheme.titleSmall),
                  TextField(
                    controller: targetAgencyController,
                    decoration: const InputDecoration(labelText: 'Agencia destino'),
                  ),
                  TextField(
                    controller: targetAccountController,
                    decoration: const InputDecoration(labelText: 'Conta destino'),
                  ),
                  DropdownButton<String>(
                    value: targetPersonType,
                    items: const [
                      DropdownMenuItem(value: 'PF', child: Text('Pessoa fisica (CPF)')),
                      DropdownMenuItem(value: 'PJ', child: Text('Pessoa juridica (CNPJ)')),
                    ],
                    onChanged: (value) => setState(() => targetPersonType = value ?? 'PF'),
                  ),
                  TextField(
                    controller: targetDocumentController,
                    keyboardType: TextInputType.number,
                    inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                    decoration: InputDecoration(
                      labelText: targetPersonType == 'PF' ? 'CPF destino (11 digitos)' : 'CNPJ destino (14 digitos)',
                    ),
                  ),
                  const SizedBox(height: 12),
                  FilledButton(
                    onPressed: () async {
                      final messenger = ScaffoldMessenger.of(context);
                      final amountCents = parseCurrencyToCents(amountController.text);
                      await widget.api.topup(
                        amountCents: amountCents,
                        entryType: entryType,
                        note: noteController.text,
                        targetAgency: targetAgencyController.text.trim(),
                        targetAccountNumber: targetAccountController.text.trim(),
                        targetPersonType: targetPersonType,
                        targetDocument: targetDocumentController.text.trim(),
                      );
                      await _refreshBalance();
                      if (!mounted) return;
                      messenger.showSnackBar(const SnackBar(content: Text('Deposito realizado e saldo atualizado')));
                    },
                    child: const Text('Confirmar deposito'),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

}

class StatementScreen extends StatelessWidget {
  const StatementScreen({super.key, required this.api});
  final BancoApiClient api;

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<dynamic>>(
      future: api.listTransactions(),
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Center(child: CircularProgressIndicator());
        }
        if (snapshot.hasError) {
          return Center(child: Text('Erro ao carregar extrato: ${snapshot.error}'));
        }
        if (!snapshot.hasData) return const Center(child: Text('Sem transacoes.'));
        final items = snapshot.data!;
        return ListView.builder(
          itemCount: items.length,
          itemBuilder: (context, index) {
            final tx = items[index] as Map<String, dynamic>;
            return ListTile(
              title: Text('${tx['type']} - ${(tx['amount_cents'] ?? 0) / 100}'),
              subtitle: Text('status=${tx['status']} gatebox=${tx['gatebox_charge_id'] ?? '-'}'),
              trailing: Text(tx['id'] ?? ''),
            );
          },
        );
      },
    );
  }
}

class PayScreen extends StatefulWidget {
  const PayScreen({super.key, required this.api});
  final BancoApiClient api;

  @override
  State<PayScreen> createState() => _PayScreenState();
}

class _PayScreenState extends State<PayScreen> {
  final referenceController = TextEditingController();
  String method = 'pix';
  String simulation = 'APPROVED';
  String result = '-';

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          DropdownButton<String>(
            value: method,
            items: const [
              DropdownMenuItem(value: 'pix', child: Text('Pagar PIX copia e cola')),
              DropdownMenuItem(value: 'qrcode', child: Text('Pagar QR Code PIX')),
              DropdownMenuItem(value: 'link', child: Text('Pagar link de pagamento')),
              DropdownMenuItem(value: 'card', child: Text('Pagar cartao (token / referencia)')),
            ],
            onChanged: (value) => setState(() => method = value ?? 'pix'),
          ),
          TextField(controller: referenceController, decoration: const InputDecoration(labelText: 'Referencia da cobranca Gatebox')),
          DropdownButton<String>(
            value: simulation,
            items: const [
              DropdownMenuItem(value: 'APPROVED', child: Text('Simular aprovado')),
              DropdownMenuItem(value: 'PENDING', child: Text('Simular pendente')),
              DropdownMenuItem(value: 'REJECTED', child: Text('Simular recusado')),
              DropdownMenuItem(value: 'INSUFFICIENT_BALANCE', child: Text('Simular saldo insuficiente')),
              DropdownMenuItem(value: 'TIMEOUT', child: Text('Simular timeout bancario')),
              DropdownMenuItem(value: 'TEMP_FAILURE', child: Text('Simular falha temporaria')),
              DropdownMenuItem(value: 'REFUNDED', child: Text('Simular estorno/reembolso')),
            ],
            onChanged: (value) => setState(() => simulation = value ?? 'APPROVED'),
          ),
          FilledButton(
            onPressed: () async {
              final data = await widget.api.pay(method: method, reference: referenceController.text, simulationState: simulation);
              setState(() => result = data.toString());
            },
            child: const Text('Validar e pagar'),
          ),
          const SizedBox(height: 12),
          Text(result),
        ],
      ),
    );
  }
}

class SimulationScreen extends StatefulWidget {
  const SimulationScreen({super.key, required this.api});
  final BancoApiClient api;

  @override
  State<SimulationScreen> createState() => _SimulationScreenState();
}

class _SimulationScreenState extends State<SimulationScreen> {
  final _hostController = TextEditingController();
  final _portController = TextEditingController(text: '${BancoApiConfig.defaultPort}');
  bool autoApprove = false;
  bool autoReject = false;
  bool timeout = false;
  bool insufficientBalance = false;
  bool webhookActive = true;
  int delay = 0;
  double failureRate = 0;
  String environment = 'sandbox';
  String? _endpointTestMessage;

  @override
  void initState() {
    super.initState();
    _loadEndpointFields();
  }

  Future<void> _loadEndpointFields() async {
    final prefs = await SharedPreferences.getInstance();
    final h = prefs.getString('banco_api_host');
    final p = prefs.getInt('banco_api_port');
    if (!mounted) return;
    setState(() {
      _hostController.text = h ?? '';
      _portController.text = '${p ?? BancoApiConfig.defaultPort}';
    });
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    super.dispose();
  }

  Future<void> _saveEndpoint() async {
    final prefs = await SharedPreferences.getInstance();
    final hostRaw = _hostController.text.trim();
    final portParsed = int.tryParse(_portController.text.trim());
    final port = (portParsed != null && portParsed > 0)
        ? portParsed
        : BancoApiConfig.defaultPort;

    if (hostRaw.isEmpty) {
      await prefs.remove('banco_api_host');
    } else {
      await prefs.setString('banco_api_host', hostRaw);
    }
    await prefs.setInt('banco_api_port', port);

    BancoApiConfig.setRuntimeEndpoint(
      host: hostRaw.isEmpty ? null : hostRaw,
      port: port,
    );
    widget.api.resetSession();

    if (!mounted) return;
    setState(() {
      _endpointTestMessage =
          'Servidor: ${BancoApiConfig.baseUrl} — sessão reiniciada.';
    });
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('API Banco (Go): ${BancoApiConfig.baseUrl}')),
    );
    if (port == 8081 && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: const Text(
            'A porta 8081 é o Gatebox (Rust). O app deve usar o backend Banco na 8091 — ver texto acima.',
          ),
          backgroundColor: Colors.orange.shade900,
          duration: const Duration(seconds: 10),
        ),
      );
    }
  }

  Future<void> _testHealth() async {
    final url = '${widget.api.baseUrl}/health';
    setState(() => _endpointTestMessage = 'A testar $url …');
    try {
      final r = await http.get(Uri.parse(url)).timeout(const Duration(seconds: 6));
      if (!mounted) return;
      setState(() {
        _endpointTestMessage = r.statusCode == 200
            ? 'OK (${r.statusCode}): ${r.body}'
            : 'HTTP ${r.statusCode}: ${r.body}';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _endpointTestMessage = 'Falha: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  'Servidor backend (Go)',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 6),
                Text(
                  'URL actual: ${widget.api.baseUrl}\n${BancoApiConfig.endpointHints}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _hostController,
                  decoration: const InputDecoration(
                    labelText: 'Host / IP (opcional)',
                    hintText: 'Vazio = defeito (ex.: 10.0.2.2 no emulador Android)',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.url,
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: _portController,
                  decoration: const InputDecoration(
                    labelText: 'Porta',
                    border: OutlineInputBorder(),
                  ),
                  keyboardType: TextInputType.number,
                  inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                ),
                const SizedBox(height: 12),
                Row(
                  children: [
                    FilledButton(
                      onPressed: _saveEndpoint,
                      child: const Text('Guardar servidor'),
                    ),
                    const SizedBox(width: 12),
                    OutlinedButton(
                      onPressed: _testHealth,
                      child: const Text('Testar /health'),
                    ),
                  ],
                ),
                if (_endpointTestMessage != null) ...[
                  const SizedBox(height: 12),
                  SelectableText(
                    _endpointTestMessage!,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        SwitchListTile(value: autoApprove, onChanged: (v) => setState(() => autoApprove = v), title: const Text('Auto aprovacao')),
        SwitchListTile(value: autoReject, onChanged: (v) => setState(() => autoReject = v), title: const Text('Auto recusa')),
        SwitchListTile(value: timeout, onChanged: (v) => setState(() => timeout = v), title: const Text('Timeout')),
        SwitchListTile(value: insufficientBalance, onChanged: (v) => setState(() => insufficientBalance = v), title: const Text('Saldo insuficiente')),
        SwitchListTile(value: webhookActive, onChanged: (v) => setState(() => webhookActive = v), title: const Text('Webhook ativo')),
        Text('Delay processamento: $delay ms'),
        Slider(value: delay.toDouble(), max: 5000, onChanged: (v) => setState(() => delay = v.round())),
        Text('Falha randomica: ${failureRate.toStringAsFixed(2)}'),
        Slider(value: failureRate, onChanged: (v) => setState(() => failureRate = v)),
        DropdownButton<String>(
          value: environment,
          items: const [
            DropdownMenuItem(value: 'sandbox', child: Text('Sandbox')),
            DropdownMenuItem(value: 'homolog', child: Text('Homolog')),
            DropdownMenuItem(value: 'production-ready', child: Text('Preparado para Producao')),
          ],
          onChanged: (value) => setState(() => environment = value ?? 'sandbox'),
        ),
        FilledButton(
          onPressed: () async {
            final messenger = ScaffoldMessenger.of(context);
            await widget.api.putSimulationSettings({
              'auto_approve': autoApprove,
              'auto_reject': autoReject,
              'processing_delay_ms': delay,
              'random_failure_rate': failureRate,
              'timeout_enabled': timeout,
              'insufficient_balance_enabled': insufficientBalance,
              'webhook_active': webhookActive,
              'gatebox_environment': environment,
            });
            if (!mounted) return;
            messenger.showSnackBar(const SnackBar(content: Text('Configuracoes salvas')));
          },
          child: const Text('Salvar simulacao'),
        ),
      ],
    );
  }
}

