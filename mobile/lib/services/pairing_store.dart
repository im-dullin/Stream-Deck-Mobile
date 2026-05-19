import 'package:shared_preferences/shared_preferences.dart';

class Pairing {
  final String host;
  final int port;
  final String token;
  final String deviceId;

  const Pairing({
    required this.host,
    required this.port,
    required this.token,
    required this.deviceId,
  });
}

/// Persists a single pairing (MVP: one PC at a time).
class PairingStore {
  static const _kHost = 'pairing.host';
  static const _kPort = 'pairing.port';
  static const _kToken = 'pairing.token';
  static const _kDeviceId = 'pairing.deviceId';

  Future<Pairing?> load() async {
    final prefs = await SharedPreferences.getInstance();
    final host = prefs.getString(_kHost);
    final port = prefs.getInt(_kPort);
    final token = prefs.getString(_kToken);
    final deviceId = prefs.getString(_kDeviceId);
    if (host == null || port == null || token == null || deviceId == null) {
      return null;
    }
    return Pairing(host: host, port: port, token: token, deviceId: deviceId);
  }

  Future<void> save(Pairing p) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kHost, p.host);
    await prefs.setInt(_kPort, p.port);
    await prefs.setString(_kToken, p.token);
    await prefs.setString(_kDeviceId, p.deviceId);
  }

  Future<void> clear() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kHost);
    await prefs.remove(_kPort);
    await prefs.remove(_kToken);
    await prefs.remove(_kDeviceId);
  }
}

/// Per-install random ID, stable across restarts. Used to identify this device
/// to the agent (eventually for multi-device pairing).
Future<String> ensureDeviceId() async {
  final prefs = await SharedPreferences.getInstance();
  var id = prefs.getString('device.id');
  if (id == null) {
    id = _randomId();
    await prefs.setString('device.id', id);
  }
  return id;
}

String _randomId() {
  final rng = DateTime.now().microsecondsSinceEpoch;
  final hex = rng.toRadixString(16).padLeft(12, '0');
  return 'dev_$hex';
}
