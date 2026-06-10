import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;

/// URL base do backend Rust LogisticaHoldFy (porta padrão **8092**).
///
/// Prioridade:
/// 1. `LOGISTICA_API_BASE_URL` — URL completa (padrão: SaveInCloud HTTPS).
///    Para dev local: `--dart-define=LOGISTICA_API_BASE_URL=http://192.168.33.109:8092`
/// 2. Override em tempo de execução ([setRuntimeEndpoint]).
/// 3. `LOGISTICA_API_HOST` + `LOGISTICA_API_PORT` compile-time.
/// 4. Valor por plataforma (Android emulador / desktop / telefone físico na Wi-Fi).
class ApiConfig {
  ApiConfig._();

  // URL completa — padrão aponta para o servidor SaveInCloud.
  // Para dev local sobrescreva com --dart-define=LOGISTICA_API_BASE_URL=http://192.168.33.109:8092
  static const String _dartDefineBaseUrl = String.fromEnvironment(
    'LOGISTICA_API_BASE_URL',
    defaultValue: 'https://holdfy-dev.sp1.br.saveincloud.net.br/svc/tracking',
  );

  static const String _dartDefineHost = String.fromEnvironment(
    'LOGISTICA_API_HOST',
    defaultValue: '',
  );
  static const String _dartDefinePort = String.fromEnvironment(
    'LOGISTICA_API_PORT',
    defaultValue: '',
  );

  static String? _runtimeHost;
  static int? _runtimePort;
  static String? _activeUrl;

  static const int defaultPort = 8092;

  static const String defaultLanHost = String.fromEnvironment(
    'LOGISTICA_API_LAN_HOST',
    defaultValue: '192.168.33.109',
  );

  // Usado pelo EndpointStore para apontar para o servidor seleccionado.
  static void setActiveUrl(String url) => _activeUrl = url;

  static void setRuntimeEndpoint({String? host, int? port}) {
    _runtimeHost = host;
    _runtimePort = port;
  }

  static void clearRuntimeEndpoint() {
    _runtimeHost = null;
    _runtimePort = null;
  }

  static int get port {
    if (_runtimePort != null && _runtimePort! > 0) return _runtimePort!;
    final dp = int.tryParse(_dartDefinePort.trim());
    if (dp != null && dp > 0) return dp;
    return defaultPort;
  }

  static String get host {
    final rt = _runtimeHost?.trim();
    if (rt != null && rt.isNotEmpty) return rt;

    final dd = _dartDefineHost.trim();
    if (dd.isNotEmpty) return dd;

    if (kIsWeb) return 'localhost';

    try {
      if (Platform.isAndroid) {
        const emulatorHost = String.fromEnvironment(
          'LOGISTICA_API_USE_EMULATOR',
          defaultValue: '',
        );
        if (emulatorHost == '1' || emulatorHost.toLowerCase() == 'true') {
          return '10.0.2.2';
        }
        return defaultLanHost;
      }
      if (Platform.isIOS) {
        return defaultLanHost;
      }
      if (Platform.isLinux || Platform.isMacOS || Platform.isWindows) {
        return '127.0.0.1';
      }
    } catch (_) {}

    return defaultLanHost;
  }

  // Prioridade: endpoint seleccionado pelo utilizador → dart-define → fallback host:port.
  static String get baseUrl {
    if (_activeUrl != null && _activeUrl!.isNotEmpty) return _activeUrl!;
    final fullUrl = _dartDefineBaseUrl.trim();
    if (fullUrl.isNotEmpty) return fullUrl;
    return 'http://$host:$port';
  }

  static String get endpointHints =>
      'Backend Rust LogisticaHoldFy — padrão: SaveInCloud HTTPS. '
      'Dev local: --dart-define=LOGISTICA_API_BASE_URL=http://192.168.33.109:8092. '
      'Emulador Android → 10.0.2.2 · Desktop → 127.0.0.1 · '
      'Telefone na Wi‑Fi → IP LAN do PC.';
}
