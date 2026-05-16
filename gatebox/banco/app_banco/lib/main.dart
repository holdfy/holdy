import 'package:app_banco/src/app.dart';
import 'package:app_banco/src/core/banco_api_config.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
  ]);

  final prefs = await SharedPreferences.getInstance();
  final savedHost = prefs.getString('banco_api_host');
  final savedPort = prefs.getInt('banco_api_port');
  if ((savedHost != null && savedHost.trim().isNotEmpty) ||
      (savedPort != null && savedPort > 0)) {
    BancoApiConfig.setRuntimeEndpoint(
      host: savedHost?.trim().isNotEmpty == true ? savedHost!.trim() : null,
      port: savedPort,
    );
  } else {
    BancoApiConfig.setRuntimeEndpoint(host: BancoApiConfig.defaultLanHost);
  }

  runApp(const BancoSaczuckApp());
}
