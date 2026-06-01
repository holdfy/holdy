import 'package:flutter/material.dart';
import 'package:logistica_holdfy/src/core/api_client.dart';
import 'package:logistica_holdfy/src/features/home_screen.dart';
import 'package:logistica_holdfy/src/features/setup_screen.dart';
import 'package:logistica_holdfy/src/theme.dart';

class LogisticaHoldFyApp extends StatefulWidget {
  const LogisticaHoldFyApp({super.key});

  @override
  State<LogisticaHoldFyApp> createState() => _LogisticaHoldFyAppState();
}

class _LogisticaHoldFyAppState extends State<LogisticaHoldFyApp> {
  final ApiClient _api = ApiClient();
  int _tab = 0;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LogisticaHoldFy',
      debugShowCheckedModeBanner: false,
      theme: buildDarkTheme(),
      home: Scaffold(
        body: IndexedStack(
          index: _tab,
          children: [
            HomeScreen(api: _api),
            SetupScreen(api: _api),
          ],
        ),
        bottomNavigationBar: NavigationBar(
          selectedIndex: _tab,
          onDestinationSelected: (i) => setState(() => _tab = i),
          destinations: const [
            NavigationDestination(
              icon: Icon(Icons.local_shipping_outlined),
              selectedIcon: Icon(Icons.local_shipping),
              label: 'Rastreios',
            ),
            NavigationDestination(
              icon: Icon(Icons.settings_outlined),
              selectedIcon: Icon(Icons.settings),
              label: 'Setup',
            ),
          ],
        ),
      ),
    );
  }
}
