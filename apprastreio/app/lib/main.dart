import 'package:flutter/material.dart';
import 'package:logistica_holdfy/src/app.dart';
import 'package:logistica_holdfy/src/core/api_config.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final prefs = await SharedPreferences.getInstance();
  final host = prefs.getString('logistica_api_host');
  final port = prefs.getInt('logistica_api_port');
  ApiConfig.setRuntimeEndpoint(
    host: host,
    port: port ?? ApiConfig.defaultPort,
  );
  runApp(const LogisticaHoldFyApp());
}
