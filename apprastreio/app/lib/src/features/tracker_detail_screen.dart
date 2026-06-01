import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:logistica_holdfy/src/core/api_client.dart';
import 'package:logistica_holdfy/src/theme.dart';

enum _StepState { completed, active, locked }

class TrackerDetailScreen extends StatefulWidget {
  const TrackerDetailScreen({
    super.key,
    required this.api,
    required this.tracker,
  });

  final ApiClient api;
  final Tracker tracker;

  @override
  State<TrackerDetailScreen> createState() => _TrackerDetailScreenState();
}

class _TrackerDetailScreenState extends State<TrackerDetailScreen> {
  late Tracker _tracker;
  List<PresetStep> _presets = [];
  bool _loadingPresets = true;
  String? _addingKey;

  @override
  void initState() {
    super.initState();
    _tracker = widget.tracker;
    _loadPresets();
  }

  Future<void> _loadPresets() async {
    try {
      final presets = await widget.api.listPresets();
      if (!mounted) return;
      setState(() {
        _presets = presets;
        _loadingPresets = false;
      });
    } catch (_) {
      if (mounted) setState(() => _loadingPresets = false);
    }
  }

  Future<void> _refresh() async {
    try {
      final t = await widget.api.getTracker(_tracker.trackingCode);
      if (!mounted) return;
      setState(() => _tracker = t);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Erro: $e')),
      );
    }
  }

  Future<void> _addPreset(String key) async {
    setState(() => _addingKey = key);
    try {
      final t = await widget.api.addPresetStep(_tracker.trackingCode, key);
      if (!mounted) return;
      setState(() => _tracker = t);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Etapa registrada — vendedor notificado')),
      );
    } on ApiException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Erro: $e')),
      );
    } finally {
      if (mounted) setState(() => _addingKey = null);
    }
  }

  void _copy(String text, String message) {
    Clipboard.setData(ClipboardData(text: text));
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }

  _StepState _stateForIndex(int index) {
    if (index < _tracker.nextPresetIndex) return _StepState.completed;
    if (index == _tracker.nextPresetIndex) return _StepState.active;
    return _StepState.locked;
  }

  bool get _allComplete =>
      _presets.isNotEmpty && _tracker.nextPresetIndex >= _presets.length;

  @override
  Widget build(BuildContext context) {
    final events = _tracker.events.reversed.toList();

    return Scaffold(
      appBar: AppBar(
        title: const Text('Detalhe do rastreio'),
        actions: [
          IconButton(onPressed: _refresh, icon: const Icon(Icons.refresh)),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          _WhatsAppLinkCard(
            tracker: _tracker,
            onCopyCode: () => _copy(
              _tracker.trackingCode,
              'Código copiado',
            ),
            onCopyWhatsApp: () => _copy(
              _tracker.whatsAppLinkText,
              'Texto copiado — cole no WhatsApp HoldFy',
            ),
          ),
          const SizedBox(height: 16),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (_tracker.orderId != null &&
                      _tracker.orderId!.isNotEmpty) ...[
                    Text(
                      'Pedido / transação',
                      style: Theme.of(context).textTheme.labelMedium?.copyWith(
                            color: const Color(0xFF94A3B8),
                          ),
                    ),
                    Text(
                      _tracker.orderId!,
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    const SizedBox(height: 10),
                  ],
                  if (_tracker.description != null &&
                      _tracker.description!.isNotEmpty)
                    Text(
                      _tracker.description!,
                      style: const TextStyle(color: Color(0xFF94A3B8)),
                    ),
                  if (_tracker.originCity != null ||
                      _tracker.destinationCity != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        '${_tracker.originCity ?? '?'} → ${_tracker.destinationCity ?? '?'}',
                        style: TextStyle(
                          color: holdfyAccent.withValues(alpha: 0.9),
                        ),
                      ),
                    ),
                  if (_tracker.sellerPhone != null &&
                      _tracker.sellerPhone!.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        'Notificações → vendedor ${_tracker.sellerPhone}',
                        style: const TextStyle(
                          fontSize: 12,
                          color: Color(0xFF94A3B8),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 20),
          Text(
            'Avançar entrega',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 4),
          Text(
            _allComplete
                ? 'Todas as etapas concluídas.'
                : 'Toque na próxima etapa da sequência. Cada toque notifica o vendedor.',
            style: const TextStyle(color: Color(0xFF94A3B8), fontSize: 13),
          ),
          const SizedBox(height: 12),
          if (_loadingPresets)
            const Center(
              child: Padding(
                padding: EdgeInsets.all(16),
                child: CircularProgressIndicator(),
              ),
            )
          else
            ..._presets.asMap().entries.map((entry) {
              final index = entry.key;
              final p = entry.value;
              final stepState = _stateForIndex(index);
              final busy = _addingKey == p.key;
              return Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: _PresetStepTile(
                  preset: p,
                  stepState: stepState,
                  busy: busy,
                  onTap: stepState == _StepState.active && !busy
                      ? () => _addPreset(p.key)
                      : null,
                ),
              );
            }),
          const SizedBox(height: 20),
          Text('Histórico', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 12),
          if (events.isEmpty)
            const Text(
              'Nenhuma etapa avançada ainda.',
              style: TextStyle(color: Color(0xFF94A3B8)),
            )
          else
            ...events.asMap().entries.map((entry) {
              final idx = entry.key;
              final ev = entry.value;
              return _TimelineTile(
                event: ev,
                isLatest: idx == 0,
                formattedDate: _formatDateTime(ev.occurredAt),
              );
            }),
        ],
      ),
    );
  }

  static String _formatDateTime(DateTime dt) {
    final local = dt.toLocal();
    String two(int n) => n.toString().padLeft(2, '0');
    return '${two(local.day)}/${two(local.month)}/${local.year} '
        '${two(local.hour)}:${two(local.minute)}';
  }
}

class _WhatsAppLinkCard extends StatelessWidget {
  const _WhatsAppLinkCard({
    required this.tracker,
    required this.onCopyCode,
    required this.onCopyWhatsApp,
  });

  final Tracker tracker;
  final VoidCallback onCopyCode;
  final VoidCallback onCopyWhatsApp;

  @override
  Widget build(BuildContext context) {
    return Card(
      color: holdfyAccent.withValues(alpha: 0.08),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Icon(Icons.chat_outlined, color: holdfyAccent),
                const SizedBox(width: 8),
                Text(
                  'Vincular no WhatsApp',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              tracker.trackingCode,
              style: const TextStyle(
                fontFamily: 'monospace',
                fontSize: 18,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 6),
            const Text(
              '1. Copie o código ou o texto abaixo\n'
              '2. Cole no chat WhatsApp HoldFy para vincular ao pedido\n'
              '3. Avance as etapas aqui — o vendedor recebe cada atualização',
              style: TextStyle(
                color: Color(0xFF94A3B8),
                fontSize: 12,
                height: 1.45,
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: onCopyCode,
                    icon: const Icon(Icons.copy, size: 18),
                    label: const Text('Copiar código'),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: FilledButton.icon(
                    onPressed: onCopyWhatsApp,
                    icon: const Icon(Icons.message_outlined, size: 18),
                    label: const Text('Copiar p/ WhatsApp'),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
              decoration: BoxDecoration(
                color: const Color(0xFF111827),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: const Color(0xFF2A3544)),
              ),
              child: Text(
                tracker.whatsAppLinkText,
                style: const TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 13,
                  color: Color(0xFF94A3B8),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _PresetStepTile extends StatelessWidget {
  const _PresetStepTile({
    required this.preset,
    required this.stepState,
    required this.busy,
    this.onTap,
  });

  final PresetStep preset;
  final _StepState stepState;
  final bool busy;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final isCompleted = stepState == _StepState.completed;
    final isActive = stepState == _StepState.active;
    final isLocked = stepState == _StepState.locked;

    Color avatarBg;
    Color avatarFg;
    IconData icon;
    if (isCompleted) {
      avatarBg = Colors.green.withValues(alpha: 0.2);
      avatarFg = Colors.green.shade400;
      icon = Icons.check;
    } else if (isActive) {
      avatarBg = holdfyAccent.withValues(alpha: 0.15);
      avatarFg = holdfyAccent;
      icon = Icons.play_arrow_rounded;
    } else {
      avatarBg = const Color(0xFF1A2332);
      avatarFg = const Color(0xFF64748B);
      icon = Icons.lock_outline;
    }

    return Card(
      color: isLocked ? const Color(0xFF141B24) : null,
      child: ListTile(
        enabled: onTap != null,
        leading: CircleAvatar(
          backgroundColor: avatarBg,
          child: busy
              ? const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : Icon(icon, color: avatarFg, size: 20),
        ),
        title: Text(
          preset.label,
          style: TextStyle(
            fontWeight: isActive ? FontWeight.w700 : FontWeight.w500,
            color: isLocked ? const Color(0xFF64748B) : null,
          ),
        ),
        subtitle: Text(
          isCompleted
              ? 'Concluída'
              : isActive
                  ? preset.description
                  : 'Aguardando etapa anterior',
          style: TextStyle(
            fontSize: 12,
            color: isLocked ? const Color(0xFF475569) : null,
          ),
        ),
        trailing: isActive && !busy
            ? Icon(Icons.touch_app_outlined, color: holdfyAccent)
            : null,
        onTap: onTap,
      ),
    );
  }
}

class _TimelineTile extends StatelessWidget {
  const _TimelineTile({
    required this.event,
    required this.isLatest,
    required this.formattedDate,
  });

  final TrackingEvent event;
  final bool isLatest;
  final String formattedDate;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Column(
            children: [
              Container(
                width: 12,
                height: 12,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: isLatest ? holdfyAccent : const Color(0xFF475569),
                ),
              ),
              Container(width: 2, height: 48, color: const Color(0xFF2A3544)),
            ],
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Card(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      event.description,
                      style: TextStyle(
                        fontWeight:
                            isLatest ? FontWeight.w600 : FontWeight.normal,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      formattedDate,
                      style: const TextStyle(
                        color: Color(0xFF64748B),
                        fontSize: 11,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
