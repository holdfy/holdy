import 'package:flutter/material.dart';
import 'package:logistica_holdfy/src/app.dart';
import 'package:logistica_holdfy/src/core/api_config.dart';
import 'package:logistica_holdfy/src/core/endpoint_store.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final url = await EndpointStore.activeUrl();
  ApiConfig.setActiveUrl(url);
  runApp(const LogisticaHoldFyApp());
}
