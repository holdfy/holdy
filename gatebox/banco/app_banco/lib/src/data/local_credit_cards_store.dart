import 'dart:convert';

import 'package:app_banco/src/core/card_expiry.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Cartão de crédito fictício vinculado a uma [SavedAccount] (dados locais / demo).
/// [panDigits] guarda o número completo fictício (Luhn) para exibir na frente ao virar;
/// em aparelhos reais evitaria-se — aqui é só sandbox no dispositivo.
class SavedCreditCard {
  SavedCreditCard({
    required this.id,
    required this.accountId,
    required this.lastFour,
    required this.brandCode,
    required this.creditLimitCents,
    this.panDigits,
    required this.cvc,
    required this.expiryMmYy,
  });

  final String id;
  final String accountId;
  final String lastFour;

  /// Ex.: VISA, MASTERCARD, ELO, AMEX, HIPERCARD, OUTRO
  final String brandCode;
  final int creditLimitCents;

  /// 16 dígitos (só dígitos), opcional para cartões antigos sem geração.
  final String? panDigits;

  /// Três dígitos CVC fictício.
  final String cvc;

  /// Validade **MMYY** (4 dígitos), ex. `0828`.
  final String expiryMmYy;

  String get maskedPan => '•••• •••• •••• $lastFour';

  String get expirySlashDisplay => CardExpiry.formatSlash(expiryMmYy);

  Map<String, dynamic> toJson() => {
    'id': id,
    'account_id': accountId,
    'last_four': lastFour,
    'brand_code': brandCode,
    'credit_limit_cents': creditLimitCents,
    if (panDigits != null && panDigits!.isNotEmpty) 'pan_digits': panDigits,
    'cvc': cvc,
    'expiry_mm_yy': expiryMmYy,
  };

  factory SavedCreditCard.fromJson(Map<String, dynamic> j) {
    final rawExp = j['expiry_mm_yy'] as String?;
    final expDigits = CardExpiry.digitsOnly(rawExp ?? '');
    String expiry;
    if (expDigits.length == 4 && CardExpiry.isValid(expDigits)) {
      expiry = expDigits;
    } else if (expDigits.length == 4) {
      expiry = CardExpiry.randomFutureMmYy();
    } else {
      expiry = '1229';
    }
    return SavedCreditCard(
      id: j['id'] as String? ?? '',
      accountId: j['account_id'] as String? ?? '',
      lastFour: j['last_four'] as String? ?? '',
      brandCode: (j['brand_code'] as String? ?? 'OUTRO').toUpperCase(),
      creditLimitCents: (j['credit_limit_cents'] as num?)?.toInt() ?? 0,
      panDigits: j['pan_digits'] as String?,
      cvc: (j['cvc'] as String?) ?? '000',
      expiryMmYy: expiry,
    );
  }
}

class LocalCreditCardsStore {
  LocalCreditCardsStore._();
  static final LocalCreditCardsStore instance = LocalCreditCardsStore._();

  static const _key = 'local_credit_cards_v1';

  Future<List<SavedCreditCard>> loadAll() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_key);
    if (raw == null || raw.isEmpty) return [];
    try {
      final decoded = jsonDecode(raw) as List<dynamic>;
      return decoded
          .map((e) => SavedCreditCard.fromJson(e as Map<String, dynamic>))
          .toList();
    } catch (_) {
      return [];
    }
  }

  Future<List<SavedCreditCard>> cardsForAccount(String accountId) async {
    final all = await loadAll();
    return all.where((c) => c.accountId == accountId).toList();
  }

  Future<void> addCard(SavedCreditCard card) async {
    final list = await loadAll();
    list.add(card);
    await _persist(list);
  }

  Future<void> updateCard(SavedCreditCard updated) async {
    final list = await loadAll();
    final i = list.indexWhere((c) => c.id == updated.id);
    if (i >= 0) {
      list[i] = updated;
      await _persist(list);
    }
  }

  Future<void> deleteCardById(String id) async {
    final list = await loadAll();
    list.removeWhere((c) => c.id == id);
    await _persist(list);
  }

  Future<void> deleteCardsForAccount(String accountId) async {
    final list = await loadAll();
    list.removeWhere((c) => c.accountId == accountId);
    await _persist(list);
  }

  Future<void> _persist(List<SavedCreditCard> list) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(
      _key,
      jsonEncode(list.map((e) => e.toJson()).toList()),
    );
  }
}
