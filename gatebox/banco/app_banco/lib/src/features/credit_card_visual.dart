import 'dart:math' as math;

import 'package:app_banco/src/core/card_number_generator.dart';
import 'package:app_banco/src/features/card_brand_logo.dart';
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

/// Frente do cartão (gradiente). [panDisplay] pode ser número formatado ou mascarado.
class CreditCardVisual extends StatelessWidget {
  const CreditCardVisual({
    super.key,
    required this.holderName,
    required this.panDisplay,
    required this.brandCode,
    required this.creditLimitCents,
    required this.expiryDisplay,
  });

  final String holderName;
  final String panDisplay;
  final String brandCode;
  final int creditLimitCents;

  /// Ex.: `08/28`
  final String expiryDisplay;

  static String brandLabel(String code) {
    switch (code.toUpperCase()) {
      case 'VISA':
        return 'VISA';
      case 'MASTERCARD':
        return 'Mastercard';
      case 'ELO':
        return 'Elo';
      case 'AMEX':
        return 'American Express';
      case 'HIPERCARD':
        return 'Hipercard';
      case 'OUTRO':
        return 'Outro';
      default:
        return 'Cartão';
    }
  }

  static (List<Color>, Alignment, Alignment) gradientForBrand(
    String brandCode,
  ) {
    switch (brandCode.toUpperCase()) {
      case 'VISA':
        return (
          [
            const Color(0xFF1A237E),
            const Color(0xFF3949AB),
            const Color(0xFF5C6BC0),
          ],
          Alignment.topLeft,
          Alignment.bottomRight,
        );
      case 'MASTERCARD':
        return (
          [
            const Color(0xFF3E2723),
            const Color(0xFF6D4C41),
            const Color(0xFFFF6F00),
          ],
          Alignment.topRight,
          Alignment.bottomLeft,
        );
      case 'ELO':
        return (
          [
            const Color(0xFF263238),
            const Color(0xFFFFC107),
            const Color(0xFFFF6F00),
          ],
          Alignment.topLeft,
          Alignment.bottomRight,
        );
      case 'AMEX':
        return (
          [
            const Color(0xFF004D40),
            const Color(0xFF00897B),
            const Color(0xFF4DB6AC),
          ],
          Alignment.topLeft,
          Alignment.bottomRight,
        );
      case 'HIPERCARD':
        return (
          [
            const Color(0xFF880E4F),
            const Color(0xFFD81B60),
            const Color(0xFFFF5252),
          ],
          Alignment.topLeft,
          Alignment.bottomRight,
        );
      default:
        return (
          [
            const Color(0xFF4A148C),
            const Color(0xFF7B1FA2),
            const Color(0xFFE040FB),
          ],
          Alignment.topRight,
          Alignment.bottomLeft,
        );
    }
  }

  @override
  Widget build(BuildContext context) {
    final money = NumberFormat.currency(locale: 'pt_BR', symbol: r'R$');
    final (colors, begin, end) = gradientForBrand(brandCode);
    final limite = money.format(creditLimitCents / 100);

    return AspectRatio(
      aspectRatio: 1.586,
      child: Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(20),
          gradient: LinearGradient(colors: colors, begin: begin, end: end),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.35),
              blurRadius: 16,
              offset: const Offset(0, 8),
            ),
          ],
        ),
        clipBehavior: Clip.antiAlias,
        child: Stack(
          children: [
            Positioned(
              right: -20,
              top: -20,
              child: Icon(
                Icons.blur_on,
                size: 120,
                color: Colors.white.withValues(alpha: 0.08),
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 20, 20, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Container(
                        width: 48,
                        height: 36,
                        decoration: BoxDecoration(
                          borderRadius: BorderRadius.circular(6),
                          gradient: const LinearGradient(
                            colors: [Color(0xFFFFE082), Color(0xFFFFD54F)],
                          ),
                        ),
                      ),
                      Expanded(
                        child: Align(
                          alignment: Alignment.centerRight,
                          child: CardBrandLogo(
                            brandCode: brandCode,
                            height: 24,
                          ),
                        ),
                      ),
                    ],
                  ),
                  const Spacer(),
                  Text(
                    panDisplay,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 18,
                      fontWeight: FontWeight.w500,
                      letterSpacing: 1.2,
                    ),
                  ),
                  const SizedBox(height: 10),
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.end,
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              holderName.toUpperCase(),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: TextStyle(
                                color: Colors.white.withValues(alpha: 0.92),
                                fontSize: 12,
                                fontWeight: FontWeight.w600,
                                letterSpacing: 0.8,
                              ),
                            ),
                            const SizedBox(height: 2),
                            Text(
                              'Limite $limite',
                              style: TextStyle(
                                color: Colors.white.withValues(alpha: 0.85),
                                fontSize: 11,
                              ),
                            ),
                          ],
                        ),
                      ),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.end,
                        children: [
                          Text(
                            'VALIDADE',
                            style: TextStyle(
                              color: Colors.white.withValues(alpha: 0.55),
                              fontSize: 8,
                              fontWeight: FontWeight.w600,
                              letterSpacing: 0.6,
                            ),
                          ),
                          const SizedBox(height: 2),
                          Text(
                            expiryDisplay,
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 13,
                              fontWeight: FontWeight.w600,
                              letterSpacing: 0.9,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Verso: faixa magnética + CVC.
class _CreditCardBack extends StatelessWidget {
  const _CreditCardBack({required this.brandCode, required this.cvc});

  final String brandCode;
  final String cvc;

  @override
  Widget build(BuildContext context) {
    final (colors, begin, end) = CreditCardVisual.gradientForBrand(brandCode);
    final showCvc = (cvc.length >= 3 && cvc.length <= 4) ? cvc : '— — —';

    return AspectRatio(
      aspectRatio: 1.586,
      child: Container(
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(20),
          gradient: LinearGradient(colors: colors, begin: begin, end: end),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.35),
              blurRadius: 16,
              offset: const Offset(0, 8),
            ),
          ],
        ),
        clipBehavior: Clip.antiAlias,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const SizedBox(height: 28),
            Container(height: 44, color: Colors.black.withValues(alpha: 0.85)),
            const Spacer(),
            Padding(
              padding: const EdgeInsets.only(right: 24, bottom: 8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Text(
                    'Segurança',
                    style: TextStyle(
                      color: Colors.white.withValues(alpha: 0.75),
                      fontSize: 10,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 14,
                      vertical: 8,
                    ),
                    decoration: BoxDecoration(
                      color: Colors.white,
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Text(
                      showCvc,
                      style: const TextStyle(
                        color: Color(0xFF1A1A1A),
                        fontSize: 18,
                        fontWeight: FontWeight.w700,
                        letterSpacing: 3,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 0, 20, 14),
              child: Text(
                'Toque para voltar à frente',
                textAlign: TextAlign.center,
                style: TextStyle(
                  color: Colors.white.withValues(alpha: 0.65),
                  fontSize: 11,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Cartão com animação de giro (frente ↔ verso com CVC). Toque para virar.
class FlippableCreditCard extends StatefulWidget {
  const FlippableCreditCard({
    super.key,
    required this.holderName,
    required this.brandCode,
    required this.creditLimitCents,
    required this.cvc,
    required this.expiryDisplay,
    this.panDigits,
    required this.maskedFallback,
  });

  final String holderName;
  final String brandCode;
  final int creditLimitCents;
  final String cvc;

  /// Ex.: `08/28` ou `••/••` enquanto incompleto.
  final String expiryDisplay;

  /// Se preenchido, a frente mostra o número formatado; senão usa [maskedFallback].
  final String? panDigits;
  final String maskedFallback;

  String get displayPanLine {
    final p = panDigits?.replaceAll(RegExp(r'\D'), '') ?? '';
    if (p.length >= 13) {
      return CardNumberGenerator.formatPanDisplay(p, brandCode);
    }
    return maskedFallback;
  }

  @override
  State<FlippableCreditCard> createState() => _FlippableCreditCardState();
}

class _FlippableCreditCardState extends State<FlippableCreditCard>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 520),
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _toggle() {
    if (_controller.isCompleted) {
      _controller.reverse();
    } else {
      _controller.forward();
    }
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: _toggle,
      child: AnimatedBuilder(
        animation: _controller,
        builder: (context, child) {
          final t = Curves.easeInOutCubic.transform(_controller.value);
          final angle = t * math.pi;
          final isFront = angle < math.pi / 2;
          return AspectRatio(
            aspectRatio: 1.586,
            child: Transform(
              alignment: Alignment.center,
              transform: Matrix4.identity()
                ..setEntry(3, 2, 0.0012)
                ..rotateY(angle),
              child: isFront
                  ? CreditCardVisual(
                      holderName: widget.holderName,
                      panDisplay: widget.displayPanLine,
                      brandCode: widget.brandCode,
                      creditLimitCents: widget.creditLimitCents,
                      expiryDisplay: widget.expiryDisplay,
                    )
                  : Transform(
                      alignment: Alignment.center,
                      transform: Matrix4.identity()..rotateY(math.pi),
                      child: _CreditCardBack(
                        brandCode: widget.brandCode,
                        cvc: widget.cvc,
                      ),
                    ),
            ),
          );
        },
      ),
    );
  }
}
