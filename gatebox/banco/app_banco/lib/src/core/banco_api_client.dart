import 'dart:convert';

import 'package:app_banco/src/core/banco_api_config.dart';
import 'package:http/http.dart' as http;

class BancoApiClient {
  /// [baseUrl] opcional (ex.: testes); caso omisso usa [BancoApiConfig.baseUrl] em cada pedido.
  BancoApiClient({String? baseUrl}) : _baseUrlOverride = baseUrl;

  static const Duration _defaultTimeout = Duration(seconds: 45);

  final String? _baseUrlOverride;

  /// URL efectiva (override de testes ou config dinâmica).
  String get baseUrl => _baseUrlOverride ?? BancoApiConfig.baseUrl;

  String userId = 'demo-user';
  bool _sessionReady = false;
  static const _demoEmail = 'demo@saczuck.bank';
  static const _demoPassword = '12345678';

  Future<Map<String, dynamic>> health() async {
    return _get('/health');
  }

  Future<Map<String, dynamic>> getMe() async {
    await _ensureSession();
    return _get('/accounts/me');
  }

  Future<Map<String, dynamic>> getBalance() async {
    await _ensureSession();
    return _get('/accounts/me/balance');
  }

  Future<List<dynamic>> listTransactions() async {
    await _ensureSession();
    final json = await _get('/transactions');
    return (json['items'] as List<dynamic>? ?? []);
  }

  Future<void> topup({
    required int amountCents,
    required String entryType,
    required String note,
    String? targetAgency,
    String? targetAccountNumber,
    String? targetPersonType,
    String? targetDocument,
  }) async {
    await _ensureSession();
    final payload = <String, dynamic>{
      'amount_cents': amountCents,
      'entry_type': entryType,
      'note': note,
    };
    if ((targetAgency ?? '').isNotEmpty) {
      payload['target_agency'] = targetAgency;
    }
    if ((targetAccountNumber ?? '').isNotEmpty) {
      payload['target_account_number'] = targetAccountNumber;
    }
    if ((targetPersonType ?? '').isNotEmpty) {
      payload['target_person_type'] = targetPersonType;
    }
    if ((targetDocument ?? '').isNotEmpty) {
      payload['target_document'] = targetDocument;
    }
    await _post('/accounts/me/topup', body: payload);
  }

  Future<Map<String, dynamic>> pay({
    required String method,
    required String reference,
    required String simulationState,
  }) async {
    await _ensureSession();
    // Não use o QR inteiro no header: fica enorme e pode falhar em alguns ambientes/proxies.
    // O backend só precisa de um identificador estável por tentativa.
    final ref = reference.trim();
    final shortRef = ref.length <= 32
        ? ref
        : '${ref.substring(0, 16)}-${ref.substring(ref.length - 16)}';
    return _post(
      '/payments/$method',
      body: {'reference': reference, 'simulation_state': simulationState},
      idempotencyKey: '${DateTime.now().millisecondsSinceEpoch}-$shortRef',
    );
  }

  Future<Map<String, dynamic>> getSimulationSettings() async {
    await _ensureSession();
    return _get('/simulation/settings');
  }

  Future<void> putSimulationSettings(Map<String, dynamic> body) async {
    await _ensureSession();
    return _put('/simulation/settings', body: body);
  }

  /// Volta a autenticar após mudar host/porta no Setup.
  void resetSession() {
    _sessionReady = false;
    userId = 'demo-user';
  }

  Future<Map<String, dynamic>> _get(String path) async {
    final response = await http
        .get(
          Uri.parse('$baseUrl$path'),
          headers: _headers(),
        )
        .timeout(_defaultTimeout, onTimeout: () => throw _timeoutErr(path));
    return _decode(response);
  }

  Future<Map<String, dynamic>> _post(
    String path, {
    required Map<String, dynamic> body,
    String? idempotencyKey,
  }) async {
    final headers = _headers();
    if (idempotencyKey != null) {
      headers['Idempotency-Key'] = idempotencyKey;
    }
    final response = await http
        .post(
          Uri.parse('$baseUrl$path'),
          headers: headers,
          body: jsonEncode(body),
        )
        .timeout(_defaultTimeout, onTimeout: () => throw _timeoutErr(path));
    return _decode(response);
  }

  Future<void> _put(String path, {required Map<String, dynamic> body}) async {
    final response = await http
        .put(
          Uri.parse('$baseUrl$path'),
          headers: _headers(),
          body: jsonEncode(body),
        )
        .timeout(_defaultTimeout, onTimeout: () => throw _timeoutErr(path));
    _decode(response);
  }

  Exception _timeoutErr(String path) => Exception(
        'Tempo esgotado ao chamar $path — confirme backend Go (:8091), Gatebox (:8081) e rede.',
      );

  Map<String, String> _headers() => {
    'Content-Type': 'application/json',
    'X-User-ID': userId,
  };

  Future<void> _ensureSession() async {
    if (_sessionReady) return;
    try {
      final login = await _post(
        '/auth/login',
        body: {'email': _demoEmail, 'password': _demoPassword},
      );
      userId = (login['user_id'] as String?) ?? userId;
      _sessionReady = true;
      return;
    } catch (_) {}

    try {
      await _post(
        '/accounts',
        body: {
          'full_name': 'Usuario Demo Developer Bank',
          'person_type': 'PF',
          'document': '11111111111',
          'email': _demoEmail,
          'password': _demoPassword,
        },
      );
    } catch (_) {
      // Conta pode ja existir; segue para login.
    }

    final login = await _post(
      '/auth/login',
      body: {'email': _demoEmail, 'password': _demoPassword},
    );
    userId = (login['user_id'] as String?) ?? userId;
    _sessionReady = true;
  }

  Map<String, dynamic> _decode(http.Response response) {
    final parsed =
        (jsonDecode(response.body) as Map<String, dynamic>?) ??
        <String, dynamic>{};
    if (response.statusCode >= 400) {
      throw Exception(
        parsed['error'] ?? 'Erro de API (${response.statusCode})',
      );
    }
    return parsed;
  }
}
