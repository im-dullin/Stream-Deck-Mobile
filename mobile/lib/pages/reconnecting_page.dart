import 'package:flutter/material.dart' hide ConnectionState;

import '../services/ws_client.dart';

class ReconnectingPage extends StatefulWidget {
  final WsClient client;
  final String? host;
  final Future<void> Function() onRetry;
  final Future<void> Function() onForget;

  const ReconnectingPage({
    super.key,
    required this.client,
    required this.host,
    required this.onRetry,
    required this.onForget,
  });

  @override
  State<ReconnectingPage> createState() => _ReconnectingPageState();
}

class _ReconnectingPageState extends State<ReconnectingPage> {
  bool _retried = false;

  @override
  void initState() {
    super.initState();
    // If app just opened with state = idle (haven't tried yet), kick off retry.
    final state = widget.client.state;
    if (state == ConnectionState.idle) {
      _retried = true;
      Future.microtask(() {
        if (mounted) widget.onRetry();
      });
    }
  }

  String _humanError(String? raw) {
    if (raw == null) return '연결에 실패했습니다.';
    if (raw.contains('not_paired')) {
      return 'PC에 등록된 페어링이 없습니다. 페어링을 다시 설정해주세요.';
    }
    if (raw.contains('version_mismatch')) {
      return '프로토콜 버전이 다릅니다. 앱을 업데이트해주세요.';
    }
    if (raw.contains('Connection refused') ||
        raw.contains('Failed') ||
        raw.contains('SocketException') ||
        raw.contains('NetworkException') ||
        raw.contains('TimeoutException')) {
      return 'PC에 연결할 수 없습니다. 같은 Wi-Fi인지, 에이전트가 실행 중인지 확인하세요.';
    }
    return raw;
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.client,
      builder: (context, _) {
        final state = widget.client.state;
        final isConnecting = state == ConnectionState.connecting;
        final isAwaiting = state == ConnectionState.awaitingApproval;
        final hasError = state == ConnectionState.error;
        final cs = Theme.of(context).colorScheme;

        return Scaffold(
          appBar: AppBar(title: const Text('재연결')),
          body: Padding(
            padding: const EdgeInsets.all(24),
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (isConnecting || isAwaiting || (!hasError && !_retried)) ...[
                    const CircularProgressIndicator(),
                    const SizedBox(height: 18),
                    Text(
                      isAwaiting ? 'PC에서 승인 대기 중…' : '재연결 중…',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    if (widget.host != null) ...[
                      const SizedBox(height: 8),
                      Text(
                        widget.host!,
                        style: TextStyle(color: cs.outline),
                      ),
                    ],
                  ] else if (hasError) ...[
                    Icon(Icons.wifi_off, size: 48, color: cs.error),
                    const SizedBox(height: 16),
                    Text(
                      '연결할 수 없습니다',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      _humanError(widget.client.errorMessage),
                      textAlign: TextAlign.center,
                      style: TextStyle(color: cs.outline),
                    ),
                    const SizedBox(height: 24),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        OutlinedButton(
                          onPressed: () async {
                            await widget.onForget();
                          },
                          child: const Text('페어링 삭제'),
                        ),
                        const SizedBox(width: 12),
                        FilledButton(
                          onPressed: () async {
                            await widget.onRetry();
                          },
                          child: const Text('재시도'),
                        ),
                      ],
                    ),
                  ],
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}
