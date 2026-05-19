import 'dart:convert';

import 'package:flutter/material.dart' hide Page;
import 'package:flutter/services.dart';

import '../protocol/messages.dart';
import '../services/ws_client.dart';

class DeckPage extends StatelessWidget {
  final WsClient client;
  final VoidCallback onDisconnect;

  const DeckPage({super.key, required this.client, required this.onDisconnect});

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: client,
      builder: (context, _) {
        final profile = client.profile;
        final page = profile?.pageById(profile.defaultPageId);

        return Scaffold(
          appBar: AppBar(
            title: Text(client.agentName ?? '가상 스트림덱'),
            actions: [
              IconButton(
                tooltip: '연결 종료',
                icon: const Icon(Icons.logout),
                onPressed: () async {
                  await client.disconnect();
                  onDisconnect();
                },
              ),
            ],
            bottom: PreferredSize(
              preferredSize: const Size.fromHeight(2),
              child: Container(
                height: 2,
                color: client.isConnected
                    ? Colors.green
                    : Theme.of(context).colorScheme.error,
              ),
            ),
          ),
          body: page == null
              ? const Center(child: Text('구성된 페이지가 없습니다'))
              : _Grid(client: client, page: page),
        );
      },
    );
  }
}

class _Grid extends StatelessWidget {
  final WsClient client;
  final Page page;

  const _Grid({required this.client, required this.page});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: GridView.builder(
        gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
          crossAxisCount: page.cols,
          mainAxisSpacing: 8,
          crossAxisSpacing: 8,
          childAspectRatio: 1,
        ),
        itemCount: page.rows * page.cols,
        itemBuilder: (context, index) {
          final row = index ~/ page.cols;
          final col = index % page.cols;
          final button = page.buttonAt(row, col);
          return _ButtonCell(
            label: button?.label,
            iconBytes: _decodeIcon(button?.iconBase64),
            empty: button == null,
            onTap: button == null
                ? null
                : () {
                    HapticFeedback.mediumImpact();
                    client.pressButton(
                      pageId: page.id,
                      row: row,
                      col: col,
                    );
                  },
          );
        },
      ),
    );
  }

  static Uint8List? _decodeIcon(String? base64Str) {
    if (base64Str == null || base64Str.isEmpty) return null;
    try {
      return base64Decode(base64Str);
    } catch (_) {
      return null;
    }
  }
}

class _ButtonCell extends StatelessWidget {
  final String? label;
  final Uint8List? iconBytes;
  final bool empty;
  final VoidCallback? onTap;

  const _ButtonCell({
    required this.label,
    required this.iconBytes,
    required this.empty,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    return Material(
      color: empty ? cs.surfaceContainerLowest : cs.surfaceContainerHighest,
      borderRadius: BorderRadius.circular(12),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: onTap,
        child: Container(
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: empty ? cs.outlineVariant : cs.outline,
              width: 1,
            ),
          ),
          padding: const EdgeInsets.all(8),
          alignment: Alignment.center,
          child: empty
              ? Icon(Icons.add, color: cs.outlineVariant, size: 18)
              : Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    if (iconBytes != null) ...[
                      Expanded(
                        child: Image.memory(
                          iconBytes!,
                          fit: BoxFit.contain,
                          gaplessPlayback: true,
                        ),
                      ),
                      const SizedBox(height: 4),
                    ],
                    Text(
                      label ?? '',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      textAlign: TextAlign.center,
                      style: TextStyle(
                        color: cs.onSurface,
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ],
                ),
        ),
      ),
    );
  }
}
