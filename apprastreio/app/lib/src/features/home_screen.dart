import 'package:flutter/material.dart';
import 'package:logistica_holdfy/src/core/api_client.dart';
import 'package:logistica_holdfy/src/features/tracker_detail_screen.dart';
import 'package:logistica_holdfy/src/theme.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.api});

  final ApiClient api;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  List<Tracker> _trackers = [];
  bool _loading = true;
  String? _error;
  bool _creating = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final items = await widget.api.listTrackers();
      if (!mounted) return;
      setState(() {
        _trackers = items;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = '$e';
        _loading = false;
      });
    }
  }

  Future<void> _createTracker() async {
    final result = await showDialog<_CreateFormResult>(
      context: context,
      builder: (ctx) => const _CreateTrackerDialog(),
    );
    if (result == null) return;

    setState(() => _creating = true);
    try {
      final tracker = await widget.api.createTracker(
        description: result.description,
        orderId: result.orderId,
        originCity: result.originCity,
        destinationCity: result.destinationCity,
        sellerPhone: result.sellerPhone,
      );
      if (!mounted) return;
      await Navigator.of(context).push(
        MaterialPageRoute(
          builder: (_) => TrackerDetailScreen(api: widget.api, tracker: tracker),
        ),
      );
      await _load();
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Erro ao criar: $e')),
      );
    } finally {
      if (mounted) setState(() => _creating = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'LogisticaHoldFy',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
            ),
            Text(
              'Simulador de rastreio',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: const Color(0xFF94A3B8),
                  ),
            ),
          ],
        ),
        actions: [
          IconButton(
            onPressed: _loading ? null : _load,
            icon: const Icon(Icons.refresh),
            tooltip: 'Atualizar',
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _creating ? null : _createTracker,
        backgroundColor: holdfyAccent,
        foregroundColor: const Color(0xFF042F2E),
        icon: _creating
            ? const SizedBox(
                width: 18,
                height: 18,
                child: CircularProgressIndicator(strokeWidth: 2),
              )
            : const Icon(Icons.add),
        label: Text(_creating ? 'Criando…' : 'Novo rastreio'),
      ),
      body: RefreshIndicator(
        onRefresh: _load,
        color: holdfyAccent,
        child: _buildBody(),
      ),
    );
  }

  Widget _buildBody() {
    if (_loading && _trackers.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null && _trackers.isEmpty) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.all(24),
        children: [
          Icon(Icons.cloud_off, size: 48, color: Colors.red.shade300),
          const SizedBox(height: 16),
          Text(
            'Não foi possível conectar ao servidor.',
            style: Theme.of(context).textTheme.titleMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          Text(
            _error!,
            style: const TextStyle(color: Color(0xFF94A3B8), fontSize: 13),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 24),
          FilledButton(onPressed: _load, child: const Text('Tentar novamente')),
        ],
      );
    }
    if (_trackers.isEmpty) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.all(24),
        children: [
          Icon(Icons.inventory_2_outlined, size: 56, color: holdfyAccent),
          const SizedBox(height: 16),
          Text(
            'Nenhum rastreio ainda',
            style: Theme.of(context).textTheme.titleMedium,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          const Text(
            'Crie um código de rastreio simulado e adicione etapas de entrega para testar integrações.',
            textAlign: TextAlign.center,
            style: TextStyle(color: Color(0xFF94A3B8)),
          ),
        ],
      );
    }

    return ListView.separated(
      physics: const AlwaysScrollableScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 96),
      itemCount: _trackers.length,
      separatorBuilder: (_, _) => const SizedBox(height: 10),
      itemBuilder: (context, i) {
        final t = _trackers[i];
        return _TrackerTile(
          tracker: t,
          onTap: () async {
            await Navigator.of(context).push(
              MaterialPageRoute(
                builder: (_) =>
                    TrackerDetailScreen(api: widget.api, tracker: t),
              ),
            );
            await _load();
          },
        );
      },
    );
  }
}

class _TrackerTile extends StatelessWidget {
  const _TrackerTile({required this.tracker, required this.onTap});

  final Tracker tracker;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final last = tracker.events.isNotEmpty ? tracker.events.last : null;
    return Card(
      child: InkWell(
        borderRadius: BorderRadius.circular(16),
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  color: holdfyAccent.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(Icons.qr_code_2, color: holdfyAccent),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      tracker.trackingCode,
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontWeight: FontWeight.bold,
                        fontSize: 15,
                      ),
                    ),
                    if (tracker.description != null &&
                        tracker.description!.isNotEmpty)
                      Text(
                        tracker.description!,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: Color(0xFF94A3B8),
                          fontSize: 13,
                        ),
                      ),
                    if (last != null)
                      Text(
                        last.description,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(fontSize: 12),
                      )
                    else
                      const Text(
                        'Aguardando 1ª etapa — vincule no WhatsApp',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: TextStyle(
                          fontSize: 11,
                          color: Color(0xFF64748B),
                        ),
                      ),
                  ],
                ),
              ),
              const Icon(Icons.chevron_right, color: Color(0xFF64748B)),
            ],
          ),
        ),
      ),
    );
  }
}

class _CreateFormResult {
  _CreateFormResult({
    this.description,
    this.orderId,
    this.originCity,
    this.destinationCity,
    this.sellerPhone,
  });

  final String? description;
  final String? orderId;
  final String? originCity;
  final String? destinationCity;
  final String? sellerPhone;
}

class _CreateTrackerDialog extends StatefulWidget {
  const _CreateTrackerDialog();

  @override
  State<_CreateTrackerDialog> createState() => _CreateTrackerDialogState();
}

class _CreateTrackerDialogState extends State<_CreateTrackerDialog> {
  final _desc = TextEditingController();
  final _orderId = TextEditingController();
  final _origin = TextEditingController(text: 'São Paulo');
  final _dest = TextEditingController(text: 'Rio de Janeiro');
  final _sellerPhone = TextEditingController();

  @override
  void dispose() {
    _desc.dispose();
    _orderId.dispose();
    _origin.dispose();
    _dest.dispose();
    _sellerPhone.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Novo rastreio'),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: _desc,
              decoration: const InputDecoration(
                labelText: 'Descrição (opcional)',
                hintText: 'Produto X',
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _orderId,
              decoration: const InputDecoration(
                labelText: 'Ref. pedido / transação',
                hintText: 'UUID ou ref curta do pedido HoldFy',
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _origin,
              decoration: const InputDecoration(labelText: 'Cidade origem'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _dest,
              decoration: const InputDecoration(labelText: 'Cidade destino'),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _sellerPhone,
              keyboardType: TextInputType.phone,
              decoration: const InputDecoration(
                labelText: 'WhatsApp vendedor (notificações)',
                hintText: '5511999999999',
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancelar'),
        ),
        FilledButton(
          onPressed: () {
            Navigator.pop(
              context,
              _CreateFormResult(
                description: _desc.text.trim(),
                orderId: _orderId.text.trim(),
                originCity: _origin.text.trim(),
                destinationCity: _dest.text.trim(),
                sellerPhone: _sellerPhone.text.trim(),
              ),
            );
          },
          child: const Text('Gerar código'),
        ),
      ],
    );
  }
}
