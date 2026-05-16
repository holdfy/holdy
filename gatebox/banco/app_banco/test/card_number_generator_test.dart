import 'package:app_banco/src/core/card_number_generator.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('generatePan is Luhn-valid and matches brand', () {
    for (final brand in ['VISA', 'MASTERCARD', 'AMEX', 'ELO', 'HIPERCARD', 'OUTRO']) {
      for (var i = 0; i < 20; i++) {
        final pan = CardNumberGenerator.generatePan(brand);
        expect(CardNumberGenerator.luhnValid(pan), isTrue, reason: '$brand -> $pan');
        expect(CardNumberGenerator.panMatchesBrand(pan, brand), isTrue, reason: '$brand -> $pan');
      }
    }
  });

  test('inferBrandCodeFromPan examples', () {
    expect(CardNumberGenerator.inferBrandCodeFromPan('4532015112830366'), 'VISA');
    final hiper = CardNumberGenerator.generatePan('HIPERCARD');
    expect(CardNumberGenerator.inferBrandCodeFromPan(hiper), 'HIPERCARD');
    final amex = CardNumberGenerator.generatePan('AMEX');
    expect(CardNumberGenerator.inferBrandCodeFromPan(amex), 'AMEX');
    expect(amex.length, 15);
  });

  test('formatPanDisplay Amex spacing', () {
    expect(
      CardNumberGenerator.formatPanDisplay('378282246310005', 'AMEX'),
      '3782 822463 10005',
    );
  });
}
