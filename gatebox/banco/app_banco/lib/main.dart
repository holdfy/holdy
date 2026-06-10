import 'package:app_banco/src/app.dart';
import 'package:app_banco/src/core/banco_api_config.dart';
import 'package:app_banco/src/core/endpoint_store.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await SystemChrome.setPreferredOrientations([DeviceOrientation.portraitUp]);
  final url = await EndpointStore.activeUrl();
  BancoApiConfig.setActiveUrl(url);
  runApp(const BancoSaczuckApp());
}
