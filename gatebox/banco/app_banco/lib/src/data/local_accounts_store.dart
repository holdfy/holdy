import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

class SavedAccount {
  SavedAccount({
    required this.id,
    required this.personName,
    required this.bankCode,
    required this.bankName,
    required this.agency,
    required this.accountNumber,
    required this.balanceCents,
  });

  final String id;
  final String personName;
  final String bankCode;
  final String bankName;
  final String agency;
  final String accountNumber;
  final int balanceCents;

  Map<String, dynamic> toJson() => {
        'id': id,
        'person_name': personName,
        'bank_code': bankCode,
        'bank_name': bankName,
        'agency': agency,
        'account_number': accountNumber,
        'balance_cents': balanceCents,
      };

  factory SavedAccount.fromJson(Map<String, dynamic> j) {
    return SavedAccount(
      id: j['id'] as String? ?? '',
      personName: j['person_name'] as String? ?? '',
      bankCode: j['bank_code'] as String? ?? '',
      bankName: j['bank_name'] as String? ?? '',
      agency: j['agency'] as String? ?? '',
      accountNumber: j['account_number'] as String? ?? '',
      balanceCents: (j['balance_cents'] as num?)?.toInt() ?? 0,
    );
  }
}

class LocalAccountsStore {
  LocalAccountsStore._();
  static final LocalAccountsStore instance = LocalAccountsStore._();

  static const _key = 'local_bank_accounts_v1';

  Future<List<SavedAccount>> loadAccounts() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_key);
    if (raw == null || raw.isEmpty) return [];
    try {
      final decoded = jsonDecode(raw) as List<dynamic>;
      return decoded.map((e) => SavedAccount.fromJson(e as Map<String, dynamic>)).toList();
    } catch (_) {
      return [];
    }
  }

  Future<void> addAccount(SavedAccount account) async {
    final list = await loadAccounts();
    list.add(account);
    await _persist(list);
  }

  Future<void> updateAccount(SavedAccount updated) async {
    final list = await loadAccounts();
    final i = list.indexWhere((a) => a.id == updated.id);
    if (i >= 0) {
      list[i] = updated;
      await _persist(list);
    }
  }

  Future<void> deleteAccountById(String id) async {
    final list = await loadAccounts();
    list.removeWhere((a) => a.id == id);
    await _persist(list);
  }

  Future<void> _persist(List<SavedAccount> list) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, jsonEncode(list.map((e) => e.toJson()).toList()));
  }
}
