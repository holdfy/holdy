import 'package:flutter/material.dart';

const _accent = Color(0xFF14B8A6);
const _surface = Color(0xFF0F1419);
const _card = Color(0xFF1A2332);
const _border = Color(0xFF2A3544);

ThemeData buildDarkTheme() {
  const scheme = ColorScheme.dark(
    primary: _accent,
    onPrimary: Color(0xFF042F2E),
    secondary: Color(0xFF38BDF8),
    surface: _surface,
    onSurface: Color(0xFFE2E8F0),
    error: Color(0xFFF87171),
  );

  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: scheme,
    scaffoldBackgroundColor: _surface,
    appBarTheme: const AppBarTheme(
      backgroundColor: _surface,
      foregroundColor: Color(0xFFF1F5F9),
      elevation: 0,
      centerTitle: false,
    ),
    cardTheme: CardThemeData(
      color: _card,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: const BorderSide(color: _border),
      ),
    ),
    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: _card,
      indicatorColor: _accent.withValues(alpha: 0.2),
      labelTextStyle: WidgetStateProperty.resolveWith((states) {
        final selected = states.contains(WidgetState.selected);
        return TextStyle(
          fontSize: 12,
          fontWeight: selected ? FontWeight.w600 : FontWeight.w500,
          color: selected ? _accent : const Color(0xFF94A3B8),
        );
      }),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        backgroundColor: _accent,
        foregroundColor: const Color(0xFF042F2E),
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: const Color(0xFF111827),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: _border),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: _border),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: _accent, width: 1.5),
      ),
      labelStyle: const TextStyle(color: Color(0xFF94A3B8)),
      hintStyle: const TextStyle(color: Color(0xFF64748B)),
    ),
    dividerColor: _border,
    snackBarTheme: SnackBarThemeData(
      backgroundColor: _card,
      contentTextStyle: const TextStyle(color: Color(0xFFE2E8F0)),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      behavior: SnackBarBehavior.floating,
    ),
  );
}

Color get holdfyAccent => _accent;
