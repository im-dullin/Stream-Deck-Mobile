import 'package:flutter/material.dart' hide ConnectionState;

import '../services/pairing_store.dart';
import '../services/ws_client.dart';

class PairingPage extends StatefulWidget {
  final WsClient client;
  final PairingStore store;
  final VoidCallback onPaired;

  const PairingPage({
    super.key,
    required this.client,
    required this.store,
    required this.onPaired,
  });

  @override
  State<PairingPage> createState() => _PairingPageState();
}

class _PairingPageState extends State<PairingPage> {
  final _formKey = GlobalKey<FormState>();
  final _hostCtrl = TextEditingController();
  final _portCtrl = TextEditingController(text: '41234');

  @override
  void dispose() {
    _hostCtrl.dispose();
    _portCtrl.dispose();
    super.dispose();
  }

  Future<void> _connect() async {
    if (!_formKey.currentState!.validate()) return;
    final host = _hostCtrl.text.trim();
    final port = int.parse(_portCtrl.text.trim());
    final deviceId = await ensureDeviceId();

    await widget.client.requestPair(
      host: host,
      port: port,
      deviceId: deviceId,
      deviceName: 'Mobile Deck',
      onTokenIssued: (token) async {
        await widget.store.save(Pairing(
          host: host,
          port: port,
          token: token,
          deviceId: deviceId,
        ));
        widget.onPaired();
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('PC 페어링')),
      body: ListenableBuilder(
        listenable: widget.client,
        builder: (context, _) {
          final state = widget.client.state;
          final error = widget.client.errorMessage;
          final busy = state == ConnectionState.connecting ||
              state == ConnectionState.awaitingApproval;

          return Padding(
            padding: const EdgeInsets.all(24),
            child: Form(
              key: _formKey,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  TextFormField(
                    controller: _hostCtrl,
                    decoration: const InputDecoration(
                      labelText: 'PC 주소',
                      hintText: '예: 192.168.1.5  또는  my-mac.local',
                    ),
                    autocorrect: false,
                    enabled: !busy,
                    validator: (v) =>
                        (v == null || v.trim().isEmpty) ? '필수 입력' : null,
                  ),
                  const SizedBox(height: 12),
                  TextFormField(
                    controller: _portCtrl,
                    decoration: const InputDecoration(labelText: '포트'),
                    keyboardType: TextInputType.number,
                    enabled: !busy,
                    validator: (v) {
                      final n = int.tryParse(v?.trim() ?? '');
                      if (n == null || n <= 0 || n > 65535) {
                        return '올바르지 않은 포트';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 24),
                  if (state == ConnectionState.awaitingApproval)
                    const _AwaitingApprovalIndicator()
                  else if (error != null && state == ConnectionState.error)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: Text(
                        error,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    ),
                  FilledButton(
                    onPressed: busy ? null : _connect,
                    child: busy
                        ? const SizedBox(
                            width: 18,
                            height: 18,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Text('페어링 요청'),
                  ),
                  const SizedBox(height: 16),
                  Text(
                    '※ 한 번 페어링하면 다음부터는 자동으로 연결됩니다.\n'
                    '홈 화면에 추가했다면 그 화면에서 한 번 더 페어링이 필요할 수 있어요.',
                    style: TextStyle(
                      color: Theme.of(context).hintColor,
                      fontSize: 12,
                    ),
                    textAlign: TextAlign.center,
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

class _AwaitingApprovalIndicator extends StatelessWidget {
  const _AwaitingApprovalIndicator();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        children: [
          const SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
          const SizedBox(height: 8),
          Text(
            'PC에서 승인 대기 중…',
            style: TextStyle(color: Theme.of(context).hintColor),
          ),
        ],
      ),
    );
  }
}
