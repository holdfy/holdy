import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

/// Logos Visa, Mastercard e American Express: SVGs do projeto [Simple Icons](https://github.com/simple-icons/simple-icons) (CC0).
/// Elo, Hipercard e «Outro»: SVGs texto simples locais (não há ícone CC0 oficial no Simple Icons para Elo).
class CardBrandLogo extends StatelessWidget {
  const CardBrandLogo({super.key, required this.brandCode, this.height = 26});

  final String brandCode;
  final double height;

  static String _assetPath(String code) {
    switch (code.toUpperCase()) {
      case 'VISA':
        return 'assets/card_brands/visa.svg';
      case 'MASTERCARD':
        return 'assets/card_brands/mastercard.svg';
      case 'AMEX':
        return 'assets/card_brands/americanexpress.svg';
      case 'ELO':
        return 'assets/card_brands/elo_placeholder.svg';
      case 'HIPERCARD':
        return 'assets/card_brands/hipercard_placeholder.svg';
      case 'OUTRO':
        return 'assets/card_brands/outro_placeholder.svg';
      default:
        return 'assets/card_brands/outro_placeholder.svg';
    }
  }

  /// Simple Icons são monocromáticos em preto; no cartão aplicamos branco.
  static bool _needsWhiteTint(String code) {
    switch (code.toUpperCase()) {
      case 'ELO':
      case 'HIPERCARD':
      case 'OUTRO':
        return false;
      default:
        return true;
    }
  }

  @override
  Widget build(BuildContext context) {
    final path = _assetPath(brandCode);
    return SvgPicture.asset(
      path,
      height: height,
      fit: BoxFit.contain,
      alignment: Alignment.centerRight,
      colorFilter: _needsWhiteTint(brandCode)
          ? const ColorFilter.mode(Colors.white, BlendMode.srcIn)
          : null,
    );
  }
}
