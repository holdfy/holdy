import 'package:app_banco/src/core/banco_api_client.dart';
import 'package:app_banco/src/core/banco_api_config.dart';
import 'package:app_banco/src/core/endpoint_store.dart';
import 'package:flutter/material.dart';

class SetupScreen extends StatefulWidget {
  const SetupScreen({super.key, required this.api});

  final BancoApiClient api;

  @override
  State<SetupScreen> createState() => _SetupScreenState();
}

class _SetupScreenState extends State<SetupScreen> {
  List<SavedEndpoint> _endpoints = [];
  String _activeId = EndpointStore.builtInId;
  bool _loading = true;
  String? _testResult;
  bool _testing = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final endpoints = await EndpointStore.load();
    final activeId = await EndpointStore.getActiveId();
    if (!mounted) return;
    setState(() {
      _endpoints = endpoints;
      _activeId = activeId;
      _loading = false;
    });
  }

  Future<void> _selectEndpoint(String id) async {
    await EndpointStore.setActiveId(id);
    final ep = _endpoints.firstWhere(
      (e) => e.id == id,
      orElse: () => EndpointStore.builtIn,
    );
    BancoApiConfig.setActiveUrl(ep.url);
    setState(() {
      _activeId = id;
      _testResult = null;
    });
  }

  Future<void> _testActive() async {
    setState(() {
      _testing = true;
      _testResult = 'A testar ${BancoApiConfig.baseUrl}/health …';
    });
    try {
      final result = await widget.api.health();
      if (!mounted) return;
      setState(() {
        _testResult = '✓  ${result['service'] ?? 'ok'} — ${result['status'] ?? 'healthy'}';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _testResult = '✗  $e');
    } finally {
      if (mounted) setState(() => _testing = false);
    }
  }

  Future<void> _showAddEditDialog({SavedEndpoint? editing}) async {
    final nameCtrl = TextEditingController(text: editing?.name ?? '');
    // Extrair o IP da URL armazenada ao editar
    String existingIp = '';
    if (editing != null) {
      final uri = Uri.tryParse(editing.url);
      existingIp = uri?.host ?? editing.url;
    }
    final ipCtrl = TextEditingController(text: existingIp);

    final result = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text(editing == null ? 'Adicionar rede local' : 'Editar rede local'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameCtrl,
              decoration: const InputDecoration(
                labelText: 'Nome',
                hintText: 'Ex: Casa, Trabalho, VPN…',
              ),
              textCapitalization: TextCapitalization.sentences,
            ),
            const SizedBox(height: 12),
            TextField(
              controller: ipCtrl,
              decoration: const InputDecoration(
                labelText: 'IP do servidor',
                hintText: '192.168.x.x',
                helperText: 'Porta :${EndpointStore.localPort} adicionada automaticamente',
              ),
              keyboardType: const TextInputType.numberWithOptions(decimal: true),
              autocorrect: false,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () {
              if (nameCtrl.text.trim().isEmpty || ipCtrl.text.trim().isEmpty) return;
              Navigator.pop(ctx, true);
            },
            child: const Text('Guardar'),
          ),
        ],
      ),
    );

    if (result != true || !mounted) return;

    final name = nameCtrl.text.trim();
    final url = EndpointStore.urlFromIp(ipCtrl.text);

    setState(() {
      if (editing == null) {
        _endpoints = [
          ..._endpoints,
          SavedEndpoint(id: EndpointStore.newId(), name: name, url: url),
        ];
      } else {
        _endpoints = _endpoints
            .map((e) => e.id == editing.id ? e.copyWith(name: name, url: url) : e)
            .toList();
      }
    });
    await EndpointStore.saveList(_endpoints);
  }

  Future<void> _delete(SavedEndpoint ep) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Remover servidor?'),
        content: Text('Remover "${ep.name}"?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancelar'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('Remover'),
          ),
        ],
      ),
    );
    if (confirm != true || !mounted) return;

    final wasActive = _activeId == ep.id;
    final updated = _endpoints.where((e) => e.id != ep.id).toList();
    await EndpointStore.saveList(updated);
    if (wasActive) await _selectEndpoint(EndpointStore.builtInId);
    setState(() => _endpoints = updated);
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) return const Center(child: CircularProgressIndicator());

    final theme = Theme.of(context);
    final accent = theme.colorScheme.primary;
    final activeEp = _endpoints.firstWhere(
      (e) => e.id == _activeId,
      orElse: () => EndpointStore.builtIn,
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Servidores')),
      floatingActionButton: FloatingActionButton(
        onPressed: () => _showAddEditDialog(),
        tooltip: 'Adicionar servidor',
        child: const Icon(Icons.add),
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(0, 8, 0, 88),
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Container(
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: accent.withAlpha(25),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: accent.withAlpha(80)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(children: [
                    Icon(Icons.wifi_tethering, color: accent, size: 18),
                    const SizedBox(width: 8),
                    Text(
                      'Activo: ${activeEp.name}',
                      style: TextStyle(
                        color: accent,
                        fontWeight: FontWeight.bold,
                        fontSize: 13,
                      ),
                    ),
                  ]),
                  const SizedBox(height: 6),
                  SelectableText(
                    activeEp.url,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 12,
                    ),
                  ),
                  const SizedBox(height: 10),
                  OutlinedButton.icon(
                    onPressed: _testing ? null : _testActive,
                    icon: _testing
                        ? const SizedBox(
                            width: 14,
                            height: 14,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.health_and_safety_outlined, size: 16),
                    label: Text(_testing ? 'A testar…' : 'Testar conexão'),
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 12, vertical: 6),
                      textStyle: const TextStyle(fontSize: 13),
                    ),
                  ),
                  if (_testResult != null) ...[
                    const SizedBox(height: 8),
                    Text(
                      _testResult!,
                      style: TextStyle(
                        fontSize: 12,
                        color: _testResult!.startsWith('✓')
                            ? Colors.green
                            : Colors.red,
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
          const Divider(height: 24),
          Padding(
            padding: const EdgeInsets.only(left: 16, bottom: 4),
            child: Text(
              'TODOS OS SERVIDORES',
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.w700,
                color: theme.colorScheme.onSurface.withAlpha(90),
                letterSpacing: 1.2,
              ),
            ),
          ),
          for (final ep in _endpoints)
            _EndpointTile(
              ep: ep,
              isActive: ep.id == _activeId,
              onSelect: () => _selectEndpoint(ep.id),
              onEdit: ep.builtIn ? null : () => _showAddEditDialog(editing: ep),
              onDelete: ep.builtIn ? null : () => _delete(ep),
            ),
        ],
      ),
    );
  }
}

class _EndpointTile extends StatelessWidget {
  const _EndpointTile({
    required this.ep,
    required this.isActive,
    required this.onSelect,
    this.onEdit,
    this.onDelete,
  });

  final SavedEndpoint ep;
  final bool isActive;
  final VoidCallback onSelect;
  final VoidCallback? onEdit;
  final VoidCallback? onDelete;

  @override
  Widget build(BuildContext context) {
    final accent = Theme.of(context).colorScheme.primary;
    return ListTile(
      contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 2),
      leading: Radio<bool>(
        value: true,
        groupValue: isActive,
        onChanged: (_) => onSelect(),
      ),
      title: Text(
        ep.name,
        style: TextStyle(
          fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
          color: isActive ? accent : null,
        ),
      ),
      subtitle: Text(
        ep.url,
        style: const TextStyle(fontFamily: 'monospace', fontSize: 11),
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      trailing: ep.builtIn
          ? const Tooltip(
              message: 'Servidor padrão — não pode ser removido',
              child: Icon(Icons.lock_outline, size: 18),
            )
          : Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (onEdit != null)
                  IconButton(
                    icon: const Icon(Icons.edit_outlined, size: 20),
                    onPressed: onEdit,
                    tooltip: 'Editar',
                  ),
                if (onDelete != null)
                  IconButton(
                    icon: const Icon(Icons.delete_outline, size: 20),
                    color: Colors.redAccent,
                    onPressed: onDelete,
                    tooltip: 'Remover',
                  ),
              ],
            ),
      onTap: onSelect,
    );
  }
}
