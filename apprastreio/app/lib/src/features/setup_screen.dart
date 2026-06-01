import 'package:flutter/material.dart';
import 'package:logistica_holdfy/src/core/api_client.dart';
import 'package:logistica_holdfy/src/core/api_config.dart';
import 'package:logistica_holdfy/src/theme.dart';
import 'package:shared_preferences/shared_preferences.dart';

class SetupScreen extends StatefulWidget {
  const SetupScreen({super.key, required this.api});

  final ApiClient api;

  @override
  State<SetupScreen> createState() => _SetupScreenState();
}

class _SetupScreenState extends State<SetupScreen> {
  final _hostController = TextEditingController();
  final _portController = TextEditingController(text: '${ApiConfig.defaultPort}');
  String? _healthMessage;
  bool _testing = false;

  @override
  void initState() {
    super.initState();
    _loadPrefs();
  }

  Future<void> _loadPrefs() async {
    final prefs = await SharedPreferences.getInstance();
    final h = prefs.getString('logistica_api_host');
    final p = prefs.getInt('logistica_api_port');
    if (!mounted) return;
    setState(() {
      _hostController.text = h ?? '';
      _portController.text = '${p ?? ApiConfig.defaultPort}';
    });
  }

  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final prefs = await SharedPreferences.getInstance();
    final hostRaw = _hostController.text.trim();
    final portParsed = int.tryParse(_portController.text.trim());
    final port = (portParsed != null && portParsed > 0)
        ? portParsed
        : ApiConfig.defaultPort;

    if (hostRaw.isEmpty) {
      await prefs.remove('logistica_api_host');
    } else {
      await prefs.setString('logistica_api_host', hostRaw);
    }
    await prefs.setInt('logistica_api_port', port);

    ApiConfig.setRuntimeEndpoint(
      host: hostRaw.isEmpty ? null : hostRaw,
      port: port,
    );

    if (!mounted) return;
    setState(() {
      _healthMessage = 'Servidor configurado: ${ApiConfig.baseUrl}';
    });
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('Backend: ${ApiConfig.baseUrl}')),
    );
  }

  Future<void> _testHealth() async {
    setState(() {
      _testing = true;
      _healthMessage = 'A testar ${ApiConfig.baseUrl}/health …';
    });
    try {
      final data = await widget.api.health();
      if (!mounted) return;
      setState(() {
        _healthMessage =
            'OK — ${data['service']} v${data['version']} (${data['status']})';
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _healthMessage = 'Falha: $e');
    } finally {
      if (mounted) setState(() => _testing = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Setup do servidor')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Icon(Icons.dns, color: holdfyAccent),
                      const SizedBox(width: 10),
                      Text(
                        'Backend Rust',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Text(
                    ApiConfig.endpointHints,
                    style: const TextStyle(
                      color: Color(0xFF94A3B8),
                      fontSize: 13,
                      height: 1.4,
                    ),
                  ),
                  const SizedBox(height: 16),
                  Container(
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: const Color(0xFF111827),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: const Color(0xFF2A3544)),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'URL actual',
                          style: TextStyle(
                            color: Color(0xFF64748B),
                            fontSize: 11,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                        const SizedBox(height: 4),
                        SelectableText(
                          ApiConfig.baseUrl,
                          style: TextStyle(
                            fontFamily: 'monospace',
                            fontWeight: FontWeight.bold,
                            color: holdfyAccent,
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: _hostController,
                    decoration: const InputDecoration(
                      labelText: 'Host / IP',
                      hintText: '127.0.0.1 ou 192.168.x.x',
                    ),
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: _portController,
                    keyboardType: TextInputType.number,
                    decoration: InputDecoration(
                      labelText: 'Porta',
                      hintText: '${ApiConfig.defaultPort}',
                    ),
                  ),
                  const SizedBox(height: 16),
                  Row(
                    children: [
                      Expanded(
                        child: FilledButton(
                          onPressed: _save,
                          child: const Text('Guardar'),
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: OutlinedButton(
                          onPressed: _testing ? null : _testHealth,
                          child: Text(_testing ? 'A testar…' : 'Testar /health'),
                        ),
                      ),
                    ],
                  ),
                  if (_healthMessage != null) ...[
                    const SizedBox(height: 12),
                    Text(
                      _healthMessage!,
                      style: const TextStyle(fontSize: 13),
                    ),
                  ],
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Integração APICash',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'O backend expõe GET /logistics/tracking/{code} no formato '
                    'compatível com apicash-logistics. Para testes locais, aponte '
                    'o CascadingTracker ou configure um proxy para esta URL.',
                    style: TextStyle(color: Color(0xFF94A3B8), fontSize: 13, height: 1.4),
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
