import 'dart:async';

import 'package:app_banco/src/core/banco_api_client.dart';
import 'package:app_banco/src/data/local_accounts_store.dart';
import 'package:app_banco/src/data/local_credit_cards_store.dart';
import 'package:app_banco/src/features/account_card_pay_screen.dart';
import 'package:app_banco/src/features/account_link_pay_screen.dart';
import 'package:app_banco/src/features/account_qr_pay_screen.dart';
import 'package:app_banco/src/features/screens.dart';
import 'package:flutter/material.dart';

class BancoSaczuckApp extends StatefulWidget {
  const BancoSaczuckApp({super.key});

  @override
  State<BancoSaczuckApp> createState() => _BancoSaczuckAppState();
}

class _BancoSaczuckAppState extends State<BancoSaczuckApp> {
  final BancoApiClient api = BancoApiClient();
  int currentIndex = 0;
  SavedAccount? _selectedAccount;

  void _onAccountDeleted(String id) {
    if (_selectedAccount?.id == id) {
      setState(() => _selectedAccount = null);
    }
    unawaited(LocalCreditCardsStore.instance.deleteCardsForAccount(id));
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Developer Bank',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(colorSchemeSeed: Colors.blue, useMaterial3: true),
      home: Scaffold(
        appBar: AppBar(title: const Text('Developer Bank')),
        body: IndexedStack(
          index: currentIndex,
          children: [
            HomeScreen(
              api: api,
              selectedAccount: _selectedAccount,
              onAccountSelected: (a) => setState(() => _selectedAccount = a),
              onAccountDeleted: _onAccountDeleted,
            ),
            PixTabScreen(
              account: _selectedAccount,
              api: api,
              pixTabActive: currentIndex == 1,
              onPaymentSettled: (updated) => setState(() => _selectedAccount = updated),
            ),
            LinkTabScreen(account: _selectedAccount, api: api),
            CardTabScreen(account: _selectedAccount),
            SimulationScreen(api: api),
          ],
        ),
        bottomNavigationBar: NavigationBar(
          selectedIndex: currentIndex,
          onDestinationSelected: (value) => setState(() => currentIndex = value),
          destinations: const [
            NavigationDestination(icon: Icon(Icons.account_balance_wallet_outlined), label: 'Contas'),
            NavigationDestination(icon: Icon(Icons.pix), label: 'PIX'),
            NavigationDestination(icon: Icon(Icons.link_rounded), label: 'Link'),
            NavigationDestination(icon: Icon(Icons.credit_card), label: 'Cartão'),
            NavigationDestination(icon: Icon(Icons.tune), label: 'Setup'),
          ],
        ),
      ),
    );
  }
}
