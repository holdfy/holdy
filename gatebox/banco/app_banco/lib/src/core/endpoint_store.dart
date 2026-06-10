import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

class SavedEndpoint {
  final String id;
  final String name;
  final String url;
  final bool builtIn;

  const SavedEndpoint({
    required this.id,
    required this.name,
    required this.url,
    this.builtIn = false,
  });

  SavedEndpoint copyWith({String? name, String? url}) => SavedEndpoint(
        id: id,
        name: name ?? this.name,
        url: url ?? this.url,
        builtIn: builtIn,
      );

  Map<String, dynamic> toJson() => {'id': id, 'name': name, 'url': url};

  factory SavedEndpoint.fromJson(Map<String, dynamic> j) => SavedEndpoint(
        id: j['id'] as String,
        name: j['name'] as String,
        url: j['url'] as String,
      );
}

class EndpointStore {
  static const String _kList = 'endpoints_v2_banco';
  static const String _kActive = 'active_endpoint_id_banco';
  static const String builtInId = 'builtin_saveincloud';

  // Porta fixa — o utilizador nunca mexe nisto.
  static const int localPort = 8091;

  /// Constrói URL a partir de um IP simples: "192.168.1.5" → "http://192.168.1.5:8091"
  static String urlFromIp(String ip) => 'http://${ip.trim()}:$localPort';

  static const SavedEndpoint builtIn = SavedEndpoint(
    id: builtInId,
    name: 'SaveInCloud (servidor)',
    url: 'https://holdfy-dev.sp1.br.saveincloud.net.br/svc/banco',
    builtIn: true,
  );

  static Future<List<SavedEndpoint>> load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kList);
    final extras = <SavedEndpoint>[];
    if (raw != null) {
      final decoded = jsonDecode(raw) as List<dynamic>;
      extras.addAll(
        decoded.map((e) => SavedEndpoint.fromJson(e as Map<String, dynamic>)),
      );
    }
    return [builtIn, ...extras];
  }

  static Future<String> getActiveId() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getString(_kActive) ?? builtInId;
  }

  static Future<void> setActiveId(String id) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kActive, id);
  }

  static Future<void> saveList(List<SavedEndpoint> endpoints) async {
    final prefs = await SharedPreferences.getInstance();
    final nonBuiltIn = endpoints.where((e) => !e.builtIn).toList();
    await prefs.setString(
      _kList,
      jsonEncode(nonBuiltIn.map((e) => e.toJson()).toList()),
    );
  }

  static Future<String> activeUrl() async {
    final all = await load();
    final activeId = await getActiveId();
    final ep = all.firstWhere((e) => e.id == activeId, orElse: () => builtIn);
    return ep.url;
  }

  static String newId() =>
      DateTime.now().millisecondsSinceEpoch.toString();
}
