import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;

/// URL base do **backend_banco** (Go), porta típica **8091** (`runapp.sh restart banco`).
///
/// Não confundir com o **Gatebox Rust** (`runapp.sh` → porta **8081**): o telefone/app **não** chama o Gatebox
/// directamente; o backend Go é que usa `GATEBOX_BASE_URL` no servidor para validar PIX.
///
/// Prioridade:
/// 1. Override em tempo de execução ([setRuntimeEndpoint]) — gravado nas preferências pelo ecrã Setup.
/// 2. Compile-time: `flutter run --dart-define=BANCO_API_HOST=192.168.1.10 --dart-define=BANCO_API_PORT=8091`
/// 3. Valor por plataforma:
///    - **Android emulador**: `10.0.2.2` aponta para o PC host.
///    - **iOS simulador**: `localhost`.
///    - **Linux/macOS/Windows** (dev): `127.0.0.1`.
///    - **Web**: `localhost`.
///
/// **Telefone físico** na mesma Wi‑Fi: no ecrã Setup defina o IP do PC (ex.: `192.168.x.x`), não use `localhost`.
class BancoApiConfig {
  BancoApiConfig._();

  static const String _dartDefineHost = String.fromEnvironment(
    'BANCO_API_HOST',
    defaultValue: '',
  );
  static const String _dartDefinePort = String.fromEnvironment(
    'BANCO_API_PORT',
    defaultValue: '',
  );

  static String? _runtimeHost;
  static int? _runtimePort;

  static const int defaultPort = 8091;

  /// IP LAN do servidor de desenvolvimento (telefone físico na mesma rede).
  /// Sobrescreva com `--dart-define=BANCO_API_LAN_HOST=…` ou no ecrã Setup.
  static const String defaultLanHost = String.fromEnvironment(
    'BANCO_API_LAN_HOST',
    defaultValue: '192.168.33.109',
  );

  /// Sobrescreve host/porta até ao próximo [clearRuntimeEndpoint] ou restart sem prefs.
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
        // Emulador Android: 10.0.2.2. Telefone físico: use Setup ou BANCO_API_LAN_HOST (defeito LAN).
        const emulatorHost = String.fromEnvironment(
          'BANCO_API_USE_EMULATOR',
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
    } catch (_) {
      // Platform não disponível (alguns testes).
    }
    return defaultLanHost;
  }

  static String get baseUrl => 'http://$host:$port';

  static String get endpointHints =>
      'Host/porta = API do Banco Go (defeito porta $defaultPort). '
      'Não use a porta 8081 aqui — essa é o Gatebox no PC (runapp.sh). '
      'Emulador Android → 10.0.2.2 · iOS simulador → localhost · '
      'Linux desktop → 127.0.0.1 · Telefone na Wi‑Fi → IP LAN do PC.';
}
