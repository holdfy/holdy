import 'package:flutter_test/flutter_test.dart';
import 'package:logistica_holdfy/src/app.dart';

void main() {
  testWidgets('LogisticaHoldFy app smoke', (WidgetTester tester) async {
    await tester.pumpWidget(const LogisticaHoldFyApp());
    expect(find.text('LogisticaHoldFy'), findsOneWidget);
  });
}
