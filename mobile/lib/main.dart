import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

import 'pages/deck_page.dart';
import 'pages/pairing_page.dart';
import 'pages/reconnecting_page.dart';
import 'services/pairing_store.dart';
import 'services/ws_client.dart';

void main() {
  runApp(const StreamDeckApp());
}

class StreamDeckApp extends StatefulWidget {
  const StreamDeckApp({super.key});

  @override
  State<StreamDeckApp> createState() => _StreamDeckAppState();
}

class _StreamDeckAppState extends State<StreamDeckApp> {
  final WsClient _client = WsClient();
  final PairingStore _store = PairingStore();
  Pairing? _savedPairing;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _restorePairing();
  }

  Future<void> _restorePairing() async {
    final pairing = await _store.load();
    if (!mounted) return;
    setState(() {
      _savedPairing = pairing;
      _loading = false;
    });
    if (pairing != null) {
      _client.connect(
        host: pairing.host,
        port: pairing.port,
        token: pairing.token,
        deviceId: pairing.deviceId,
        deviceName: 'Mobile Deck',
      );
    }
  }

  /// Try connecting again with the saved pairing.
  Future<void> _retry() async {
    final pairing = _savedPairing ?? await _store.load();
    if (pairing == null) {
      if (!mounted) return;
      setState(() => _savedPairing = null);
      return;
    }
    _client.connect(
      host: pairing.host,
      port: pairing.port,
      token: pairing.token,
      deviceId: pairing.deviceId,
      deviceName: 'Mobile Deck',
    );
  }

  /// Drop the saved pairing entirely and return to the pairing screen.
  Future<void> _forget() async {
    await _client.disconnect();
    await _store.clear();
    if (!mounted) return;
    setState(() => _savedPairing = null);
  }

  /// Called by deck page when user taps "disconnect". Closes the live WS but
  /// keeps the saved pairing so the next launch auto-reconnects.
  Future<void> _disconnect() async {
    await _client.disconnect();
    if (!mounted) return;
    setState(() {});
  }

  Future<void> _onPaired() async {
    // Reload saved pairing after a successful pair request.
    final pairing = await _store.load();
    if (!mounted) return;
    setState(() => _savedPairing = pairing);
  }

  @override
  void dispose() {
    _client.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final base = ThemeData(
      colorScheme: ColorScheme.fromSeed(
        seedColor: Colors.indigo,
        brightness: Brightness.dark,
      ),
      useMaterial3: true,
    );
    final theme = base.copyWith(
      textTheme: GoogleFonts.notoSansKrTextTheme(base.textTheme),
    );

    return MaterialApp(
      title: '가상 스트림덱',
      debugShowCheckedModeBanner: false,
      theme: theme,
      home: _loading
          ? const Scaffold(body: Center(child: CircularProgressIndicator()))
          : ListenableBuilder(
              listenable: _client,
              builder: (context, _) {
                if (_client.isConnected) {
                  return DeckPage(
                    client: _client,
                    onDisconnect: _disconnect,
                  );
                }
                // Have a saved pairing → reconnect screen rather than the
                // pairing form (so the token isn't requested again).
                if (_savedPairing != null) {
                  return ReconnectingPage(
                    client: _client,
                    host:
                        '${_savedPairing!.host}:${_savedPairing!.port}',
                    onRetry: _retry,
                    onForget: _forget,
                  );
                }
                return PairingPage(
                  client: _client,
                  store: _store,
                  onPaired: _onPaired,
                );
              },
            ),
    );
  }
}
