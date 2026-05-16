import 'dart:math';

import 'package:flutter/services.dart';

/// Validade do cartão em **MMYY** (4 dígitos), ex.: `0828` = agosto/2028 — só demo local.
class CardExpiry {
  CardExpiry._();

  static final _random = Random.secure();

  /// Apenas dígitos, no máximo 4.
  static String digitsOnly(String raw) => raw.replaceAll(RegExp(r'\D'), '');

  /// `MM/AA` a partir de até 4 dígitos (para o campo de texto).
  static String formatSlash(String raw) {
    final d = digitsOnly(raw);
    if (d.isEmpty) return '';
    if (d.length <= 2) return d;
    return '${d.substring(0, 2)}/${d.substring(2, d.length.clamp(0, 4))}';
  }

  /// Data fictícia entre ~2 e ~6 anos no futuro (mês coerente).
  static String randomFutureMmYy() {
    final now = DateTime.now();
    final monthsAhead = 24 + _random.nextInt(37);
    var y = now.year;
    var m = now.month + monthsAhead;
    while (m > 12) {
      m -= 12;
      y += 1;
    }
    return '${m.toString().padLeft(2, '0')}${(y % 100).toString().padLeft(2, '0')}';
  }

  static int _fullYearFromYy(int yy) => 2000 + yy;

  /// Último instante do mês de validade (cartão válido até fim desse mês).
  static DateTime endOfExpiryMonth(String mmYy) {
    final m = int.parse(mmYy.substring(0, 2));
    final yy = int.parse(mmYy.substring(2, 4));
    final y = _fullYearFromYy(yy);
    return DateTime(y, m + 1, 0);
  }

  /// Mês 01–12, não passada (comparado ao fim do mês de validade).
  static bool isValid(String mmYy) {
    if (mmYy.length != 4) return false;
    final m = int.tryParse(mmYy.substring(0, 2));
    final yy = int.tryParse(mmYy.substring(2, 4));
    if (m == null || yy == null) return false;
    if (m < 1 || m > 12) return false;
    final end = endOfExpiryMonth(mmYy);
    final today = DateTime.now();
    final startOfToday = DateTime(today.year, today.month, today.day);
    return !end.isBefore(startOfToday);
  }
}

/// Campo de texto `MM/AA` com no máximo 4 dígitos.
class ExpiryMmYyInputFormatter extends TextInputFormatter {
  const ExpiryMmYyInputFormatter();

  @override
  TextEditingValue formatEditUpdate(
    TextEditingValue oldValue,
    TextEditingValue newValue,
  ) {
    var digits = newValue.text.replaceAll(RegExp(r'\D'), '');
    if (digits.length > 4) digits = digits.substring(0, 4);
    final formatted = CardExpiry.formatSlash(digits);
    return TextEditingValue(
      text: formatted,
      selection: TextSelection.collapsed(offset: formatted.length),
    );
  }
}
