import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:logistica_holdfy/src/core/api_config.dart';

class TrackingEvent {
  TrackingEvent({
    required this.id,
    required this.status,
    required this.description,
    this.location,
    required this.occurredAt,
    this.presetKey,
  });

  final String id;
  final String status;
  final String description;
  final String? location;
  final DateTime occurredAt;
  final String? presetKey;

  factory TrackingEvent.fromJson(Map<String, dynamic> j) {
    return TrackingEvent(
      id: j['id'] as String? ?? '',
      status: j['status'] as String? ?? 'unknown',
      description: j['description'] as String? ?? '',
      location: j['location'] as String?,
      occurredAt: DateTime.tryParse(j['occurred_at'] as String? ?? '') ??
          DateTime.now(),
      presetKey: j['preset_key'] as String?,
    );
  }
}

class Tracker {
  Tracker({
    required this.id,
    required this.trackingCode,
    this.description,
    this.orderId,
    this.originCity,
    this.destinationCity,
    required this.events,
    required this.createdAt,
    required this.nextPresetIndex,
  });

  final String id;
  final String trackingCode;
  final String? description;
  final String? orderId;
  final String? originCity;
  final String? destinationCity;
  final List<TrackingEvent> events;
  final DateTime createdAt;
  final int nextPresetIndex;

  String get currentStatus =>
      events.isNotEmpty ? events.last.status : 'pending';

  factory Tracker.fromJson(Map<String, dynamic> j) {
    final rawEvents = j['events'] as List<dynamic>? ?? [];
    return Tracker(
      id: j['id'] as String? ?? '',
      trackingCode: j['tracking_code'] as String? ?? '',
      description: j['description'] as String?,
      orderId: j['order_id'] as String?,
      originCity: j['origin_city'] as String?,
      destinationCity: j['destination_city'] as String?,
      events: rawEvents
          .map((e) => TrackingEvent.fromJson(e as Map<String, dynamic>))
          .toList(),
      createdAt: DateTime.tryParse(j['created_at'] as String? ?? '') ??
          DateTime.now(),
      nextPresetIndex: (j['next_preset_index'] as num?)?.toInt() ??
          countPresetEvents(rawEvents),
    );
  }
}

class PresetStep {
  PresetStep({
    required this.index,
    required this.key,
    required this.label,
    required this.description,
    required this.status,
  });

  final int index;
  final String key;
  final String label;
  final String description;
  final String status;

  factory PresetStep.fromJson(Map<String, dynamic> j) {
    return PresetStep(
      index: (j['index'] as num?)?.toInt() ?? 0,
      key: j['key'] as String? ?? '',
      label: j['label'] as String? ?? '',
      description: j['description'] as String? ?? '',
      status: j['status'] as String? ?? 'in_transit',
    );
  }
}

class ApiClient {
  ApiClient({String? baseUrl}) : _baseUrlOverride = baseUrl;

  final String? _baseUrlOverride;
  static const _timeout = Duration(seconds: 20);

  String get baseUrl => _baseUrlOverride ?? ApiConfig.baseUrl;

  Future<List<Tracker>> listTrackers() async {
    final json = await _getJson('/trackers');
    final items = json as List<dynamic>? ?? [];
    return items
        .map((e) => Tracker.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<Tracker> createTracker({
    String? description,
    String? orderId,
    String? originCity,
    String? destinationCity,
  }) async {
    final body = <String, dynamic>{
      if (description != null && description.isNotEmpty)
        'description': description,
      if (orderId != null && orderId.isNotEmpty) 'order_id': orderId,
      if (originCity != null && originCity.isNotEmpty)
        'origin_city': originCity,
      if (destinationCity != null && destinationCity.isNotEmpty)
        'destination_city': destinationCity,
    };
    final json = await _postJson('/trackers', body);
    return Tracker.fromJson(json);
  }

  Future<Tracker> getTracker(String code) async {
    final json = await _getJson('/trackers/${Uri.encodeComponent(code)}');
    return Tracker.fromJson(json as Map<String, dynamic>);
  }

  Future<List<PresetStep>> listPresets() async {
    final json = await _getJson('/presets');
    final items = json as List<dynamic>? ?? [];
    return items
        .map((e) => PresetStep.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<Tracker> addPresetStep(String code, String presetKey) async {
    final json = await _postJson(
      '/trackers/${Uri.encodeComponent(code)}/presets',
      {'preset_key': presetKey},
    );
    return Tracker.fromJson(json);
  }

  Future<Map<String, dynamic>> health() async {
    final json = await _getJson('/health');
    return json as Map<String, dynamic>;
  }

  Future<dynamic> _getJson(String path) async {
    final uri = Uri.parse('$baseUrl$path');
    final resp = await http.get(uri).timeout(_timeout);
    return _decode(resp);
  }

  Future<Map<String, dynamic>> _postJson(
    String path,
    Map<String, dynamic> body,
  ) async {
    final uri = Uri.parse('$baseUrl$path');
    final resp = await http
        .post(
          uri,
          headers: {'Content-Type': 'application/json'},
          body: jsonEncode(body),
        )
        .timeout(_timeout);
    return _decode(resp) as Map<String, dynamic>;
  }

  dynamic _decode(http.Response resp) {
    final body = resp.body.isEmpty ? '{}' : resp.body;
    final decoded = jsonDecode(body);
    if (resp.statusCode >= 400) {
      final msg = decoded is Map && decoded['error'] != null
          ? decoded['error'].toString()
          : 'HTTP ${resp.statusCode}';
      throw ApiException(msg, resp.statusCode);
    }
    return decoded;
  }
}

class ApiException implements Exception {
  ApiException(this.message, this.statusCode);
  final String message;
  final int statusCode;

  @override
  String toString() => message;
}

int countPresetEvents(List<dynamic> rawEvents) {
  return rawEvents.where((e) {
    if (e is! Map) return false;
    final key = e['preset_key'];
    return key != null && key.toString().isNotEmpty;
  }).length;
}
