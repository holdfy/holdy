import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;

/// URL base do backend Rust LogisticaHoldFy (porta padrão **8092**).
class ApiConfig {
  ApiConfig._();

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

  static const int defaultPort = 8092;

  static const String defaultLanHost = String.fromEnvironment(
    'LOGISTICA_API_LAN_HOST',
    defaultValue: '10.20.3.75',
  );

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

  static String get baseUrl => 'http://$host:$port';

  static String get endpointHints =>
      'Backend Rust LogisticaHoldFy — porta padrão $defaultPort. '
      'Emulador Android → 10.0.2.2 · Desktop → 127.0.0.1 · '
      'Telefone na Wi‑Fi → IP LAN do PC.';
}
