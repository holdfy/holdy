import 'dart:math';

/// Gera PAN fictício com **Luhn válido** e **prefixo IIN/BIN** alinhado à bandeira (regras públicas).
/// Apenas para demo / formulário local — não usar em pagamentos reais.
class CardNumberGenerator {
  CardNumberGenerator._();

  static final _random = Random.secure();

  /// Prefixos Elo (6 dígitos) e intervalos conhecidos — listas públicas de identificação de bandeira.
  static const _eloExactSix = <int>{
    401178, 401179, 431274, 438935, 451416, 457393, 457631, 457632, 504175,
    627780, 636297, 636368, 650031, 650033, 650035, 650051, 650405, 650439,
    650485, 650538, 650541, 650598, 650700, 650718, 650720, 650727, 650901,
    650920, 651652, 651679, 655000, 655019, 655021, 655058,
  };

  static bool _eloSix(int six) {
    if (_eloExactSix.contains(six)) return true;
    if (six >= 506699 && six <= 506778) return true;
    if (six >= 509000 && six <= 509999) return true;
    return false;
  }

  static bool luhnValid(String digits) {
    final d = digits.replaceAll(RegExp(r'\D'), '');
    if (d.isEmpty) return false;
    var sum = 0;
    var alternate = false;
    for (var i = d.length - 1; i >= 0; i--) {
      var n = d.codeUnitAt(i) - 0x30;
      if (n < 0 || n > 9) return false;
      if (alternate) {
        n *= 2;
        if (n > 9) n -= 9;
      }
      sum += n;
      alternate = !alternate;
    }
    return sum % 10 == 0;
  }

  static String formatPanGroups(String digits) {
    final d = digits.replaceAll(RegExp(r'\D'), '');
    final b = StringBuffer();
    for (var i = 0; i < d.length; i++) {
      if (i > 0 && i % 4 == 0) b.write(' ');
      b.write(d[i]);
    }
    return b.toString();
  }

  /// Formatação típica por bandeira (ex.: Amex 4-6-5).
  static String formatPanDisplay(String raw, String brandCode) {
    final d = raw.replaceAll(RegExp(r'\D'), '');
    if (d.isEmpty) return '';
    if (brandCode.toUpperCase() == 'AMEX' && d.length == 15) {
      return '${d.substring(0, 4)} ${d.substring(4, 10)} ${d.substring(10)}';
    }
    return formatPanGroups(d);
  }

  static int _targetPanLength(String brandUpper) {
    switch (brandUpper) {
      case 'AMEX':
        return 15;
      default:
        return 16;
    }
  }

  static String _randomDigits(int n) {
    final sb = StringBuffer();
    for (var i = 0; i < n; i++) {
      sb.write(_random.nextInt(10));
    }
    return sb.toString();
  }

  static String _sixDigitMastercard() {
    if (_random.nextBool()) {
      final v = 510000 + _random.nextInt(559999 - 510000 + 1);
      return v.toString().padLeft(6, '0');
    }
    final v = 222100 + _random.nextInt(272099 - 222100 + 1);
    return v.toString();
  }

  static String _sixDigitVisa() {
    final v = 400000 + _random.nextInt(100000);
    return v.toString();
  }

  static String _sixDigitAmex() {
    final head = _random.nextBool() ? '34' : '37';
    return head + _randomDigits(4);
  }

  static String _sixDigitElo() {
    final list = _eloExactSix.toList();
    return list[_random.nextInt(list.length)].toString();
  }

  static String _sixDigitOutro() {
    if (_random.nextBool()) {
      return '6011${_randomDigits(2)}';
    }
    return '3528${_randomDigits(2)}';
  }

  static String _prefixSix(String brandUpper) {
    switch (brandUpper) {
      case 'VISA':
        return _sixDigitVisa();
      case 'MASTERCARD':
        return _sixDigitMastercard();
      case 'AMEX':
        return _sixDigitAmex();
      case 'ELO':
        return _sixDigitElo();
      case 'HIPERCARD':
        return '606282';
      case 'OUTRO':
        return _sixDigitOutro();
      default:
        return _sixDigitVisa();
    }
  }

  /// Gera PAN com prefixo coerente com [brandCode], comprimento de rede (Amex 15, resto 16) e **Luhn válido**.
  static String generatePan(String brandCode) {
    final b = brandCode.toUpperCase();
    final len = _targetPanLength(b);
    for (var attempt = 0; attempt < 32; attempt++) {
      final prefix = _prefixSix(b);
      final bodyLen = len - 1 - prefix.length;
      if (bodyLen < 0) continue;
      final base = prefix + _randomDigits(bodyLen);
      for (var c = 0; c < 10; c++) {
        final cand = '$base$c';
        if (luhnValid(cand)) return cand;
      }
    }
    return '4532015112830366';
  }

  static String generateCvc() {
    return (100 + _random.nextInt(900)).toString();
  }

  /// Infere bandeira a partir dos primeiros dígitos (IIN) e do comprimento — heurística pública.
  /// Comprimento menor que 6 devolve `OUTRO`.
  static String inferBrandCodeFromPan(String raw) {
    final d = raw.replaceAll(RegExp(r'\D'), '');
    if (d.length < 6) return 'OUTRO';
    final six = int.tryParse(d.substring(0, 6));
    if (six == null) return 'OUTRO';

    if (six == 606282) return 'HIPERCARD';
    if (_eloSix(six)) return 'ELO';

    final two = int.tryParse(d.substring(0, 2));
    final four = int.tryParse(d.substring(0, 4));
    if (d.length == 15 && (two == 34 || two == 37)) return 'AMEX';

    if (four != null && four >= 2221 && four <= 2720) return 'MASTERCARD';
    if (two != null && two >= 51 && two <= 55) return 'MASTERCARD';

    if (d[0] == '4') return 'VISA';

    return 'OUTRO';
  }

  /// Verdadeiro se o número for **Luhn-válido**, comprimento típico da rede e IIN compatível com a bandeira.
  static bool panMatchesBrand(String raw, String brandCode) {
    final d = raw.replaceAll(RegExp(r'\D'), '');
    if (!luhnValid(d)) return false;
    final want = brandCode.toUpperCase();
    if (want == 'AMEX') {
      if (d.length != 15) return false;
    } else if (want == 'OUTRO') {
      if (d.length < 13 || d.length > 19) return false;
    } else {
      if (d.length != 16) return false;
    }
    final inferred = inferBrandCodeFromPan(d);
    if (want == 'OUTRO') {
      return inferred == 'OUTRO';
    }
    return inferred == want;
  }
}
