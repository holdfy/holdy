import 'package:flutter/services.dart';
import 'package:intl/intl.dart';

int parseCurrencyToCents(String value) {
  final digits = value.replaceAll(RegExp(r'[^0-9]'), '');
  return int.tryParse(digits) ?? 0;
}

String formatCentsToCurrencyField(int cents) {
  return NumberFormat.currency(locale: 'pt_BR', symbol: r'R$').format(cents / 100);
}

class CurrencyPtBrInputFormatter extends TextInputFormatter {
  final NumberFormat _formatter = NumberFormat.currency(locale: 'pt_BR', symbol: 'R\$');

  @override
  TextEditingValue formatEditUpdate(TextEditingValue oldValue, TextEditingValue newValue) {
    if (newValue.text.isEmpty) {
      return const TextEditingValue(text: 'R\$ 0,00', selection: TextSelection.collapsed(offset: 7));
    }
    final digits = newValue.text.replaceAll(RegExp(r'[^0-9]'), '');
    final value = (double.tryParse(digits) ?? 0) / 100;
    final newText = _formatter.format(value);
    return TextEditingValue(
      text: newText,
      selection: TextSelection.collapsed(offset: newText.length),
    );
  }
}
