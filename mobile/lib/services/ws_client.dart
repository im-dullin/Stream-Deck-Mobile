import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/status.dart' as status;
import 'package:web_socket_channel/web_socket_channel.dart';

import '../protocol/messages.dart';

enum ConnectionState {
  idle,
  connecting,
  awaitingApproval,
  connected,
  error,
}

typedef PairTokenCallback = void Function(String token);

/// Manages the lifecycle of the WebSocket connection to the PC agent. Holds
/// the latest [Profile] received from the agent and exposes mutation methods
/// (button press, page change) that serialize to wire messages.
class WsClient extends ChangeNotifier {
  WebSocketChannel? _channel;
  StreamSubscription? _sub;

  ConnectionState _state = ConnectionState.idle;
  String? _errorMessage;
  Profile? _profile;
  String? _agentName;
  PairTokenCallback? _onTokenIssued;

  ConnectionState get state => _state;
  String? get errorMessage => _errorMessage;
  Profile? get profile => _profile;
  String? get agentName => _agentName;
  bool get isConnected => _state == ConnectionState.connected;

  /// Reconnect using an existing pairing token (returning client).
  Future<void> connect({
    required String host,
    required int port,
    required String token,
    required String deviceId,
    required String deviceName,
  }) async {
    await disconnect();
    _onTokenIssued = null;
    _setState(ConnectionState.connecting, error: null);

    final uri = Uri.parse('ws://$host:$port');
    try {
      final channel = WebSocketChannel.connect(uri);
      await channel.ready;
      _channel = channel;

      final hello = HelloMessage(
        protocolVersion: protocolVersion,
        deviceId: deviceId,
        deviceName: deviceName,
        token: token,
      );
      channel.sink.add(jsonEncode(hello.toJson()));

      _attachListener(channel);
    } catch (e) {
      _setState(ConnectionState.error, error: e.toString());
    }
  }

  /// Initial pairing flow for a new device. Caller provides [onTokenIssued]
  /// to persist the agent-issued token once the user approves on the PC.
  Future<void> requestPair({
    required String host,
    required int port,
    required String deviceId,
    required String deviceName,
    required PairTokenCallback onTokenIssued,
  }) async {
    await disconnect();
    _onTokenIssued = onTokenIssued;
    _setState(ConnectionState.connecting, error: null);

    final uri = Uri.parse('ws://$host:$port');
    try {
      final channel = WebSocketChannel.connect(uri);
      await channel.ready;
      _channel = channel;

      final req = PairRequestMessage(
        protocolVersion: protocolVersion,
        deviceId: deviceId,
        deviceName: deviceName,
      );
      channel.sink.add(jsonEncode(req.toJson()));

      _attachListener(channel);
    } catch (e) {
      _setState(ConnectionState.error, error: e.toString());
    }
  }

  Future<void> disconnect() async {
    await _sub?.cancel();
    _sub = null;
    await _channel?.sink.close(status.goingAway);
    _channel = null;
    _profile = null;
    _agentName = null;
    if (_state != ConnectionState.idle) {
      _setState(ConnectionState.idle);
    }
  }

  void pressButton({
    required String pageId,
    required int row,
    required int col,
  }) {
    final ch = _channel;
    if (ch == null || _state != ConnectionState.connected) return;
    final msg = ButtonPressMessage(pageId: pageId, row: row, col: col);
    ch.sink.add(jsonEncode(msg.toJson()));
  }

  void _attachListener(WebSocketChannel channel) {
    _sub = channel.stream.listen(
      _onMessage,
      onError: (Object e) {
        _setState(ConnectionState.error, error: e.toString());
      },
      onDone: () {
        if (_state == ConnectionState.connected ||
            _state == ConnectionState.awaitingApproval) {
          _setState(ConnectionState.idle);
        }
      },
      cancelOnError: true,
    );
  }

  void _onMessage(dynamic data) {
    final raw = data is String ? data : data.toString();
    final Map<String, dynamic> json;
    try {
      json = jsonDecode(raw) as Map<String, dynamic>;
    } catch (_) {
      return;
    }

    final ServerMessage msg;
    try {
      msg = ServerMessage.fromJson(json);
    } catch (_) {
      return;
    }

    switch (msg) {
      case WelcomeMessage(:final agentName, :final profile):
        _agentName = agentName;
        _profile = profile;
        _setState(ConnectionState.connected);
      case ProfileUpdateMessage(:final profile):
        _profile = profile;
        notifyListeners();
      case PingMessage():
        _channel?.sink.add(jsonEncode(const PongMessage().toJson()));
      case PairPendingMessage():
        _setState(ConnectionState.awaitingApproval);
      case PairAcceptedMessage(:final token):
        // Persist token via callback; Welcome will follow on the same socket
        // and transition us to connected.
        _onTokenIssued?.call(token);
      case PairRejectedMessage(:final reason):
        _setState(ConnectionState.error, error: 'Pairing $reason');
        disconnect();
      case ErrorMessage(:final code, :final message):
        _setState(ConnectionState.error, error: '[$code] $message');
        disconnect();
    }
  }

  void _setState(ConnectionState s, {String? error}) {
    _state = s;
    _errorMessage = error;
    notifyListeners();
  }

  @override
  void dispose() {
    _sub?.cancel();
    _channel?.sink.close(status.goingAway);
    super.dispose();
  }
}
