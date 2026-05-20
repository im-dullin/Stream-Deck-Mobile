// Wire protocol mirror of `schema/protocol.ts` and
// `agent/src-tauri/src/protocol.rs`. Keep in sync.

const int protocolVersion = 1;

// =====================================================================
// Domain
// =====================================================================

class Profile {
  final String id;
  final String name;
  final String defaultPageId;
  final List<Page> pages;

  const Profile({
    required this.id,
    required this.name,
    required this.defaultPageId,
    required this.pages,
  });

  factory Profile.fromJson(Map<String, dynamic> json) => Profile(
        id: json['id'] as String,
        name: json['name'] as String,
        defaultPageId: json['defaultPageId'] as String,
        pages: (json['pages'] as List<dynamic>)
            .map((e) => Page.fromJson(e as Map<String, dynamic>))
            .toList(growable: false),
      );

  Page? pageById(String id) {
    for (final p in pages) {
      if (p.id == id) return p;
    }
    return null;
  }
}

class Page {
  final String id;
  final String name;
  final int rows;
  final int cols;
  final List<Button> buttons;

  const Page({
    required this.id,
    required this.name,
    required this.rows,
    required this.cols,
    required this.buttons,
  });

  factory Page.fromJson(Map<String, dynamic> json) => Page(
        id: json['id'] as String,
        name: json['name'] as String,
        rows: json['rows'] as int,
        cols: json['cols'] as int,
        buttons: (json['buttons'] as List<dynamic>)
            .map((e) => Button.fromJson(e as Map<String, dynamic>))
            .toList(growable: false),
      );

  Button? buttonAt(int row, int col) {
    for (final b in buttons) {
      if (b.row == row && b.col == col) return b;
    }
    return null;
  }
}

class Button {
  final int row;
  final int col;
  final String? label;
  final String? iconBase64;
  final Action action;

  const Button({
    required this.row,
    required this.col,
    this.label,
    this.iconBase64,
    required this.action,
  });

  factory Button.fromJson(Map<String, dynamic> json) => Button(
        row: json['row'] as int,
        col: json['col'] as int,
        label: json['label'] as String?,
        iconBase64: json['iconBase64'] as String?,
        action: Action.fromJson(json['action'] as Map<String, dynamic>),
      );
}

sealed class Action {
  const Action();
  factory Action.fromJson(Map<String, dynamic> json) {
    switch (json['type']) {
      case 'launch_app':
        return LaunchAppAction(
          appPath: json['appPath'] as String,
          appName: json['appName'] as String,
        );
      case 'open_url':
        return OpenUrlAction(
          url: json['url'] as String,
          displayName: json['displayName'] as String?,
        );
      case 'open_folder':
        return OpenFolderAction(
          path: json['path'] as String,
          displayName: json['displayName'] as String?,
        );
      case 'multi_action':
        return MultiAction(
          actions: (json['actions'] as List<dynamic>)
              .map((e) => Action.fromJson(e as Map<String, dynamic>))
              .toList(growable: false),
        );
      default:
        throw FormatException('unknown action type: ${json['type']}');
    }
  }
}

class LaunchAppAction extends Action {
  final String appPath;
  final String appName;
  const LaunchAppAction({required this.appPath, required this.appName});
}

class OpenUrlAction extends Action {
  final String url;
  final String? displayName;
  const OpenUrlAction({required this.url, this.displayName});
}

class OpenFolderAction extends Action {
  final String path;
  final String? displayName;
  const OpenFolderAction({required this.path, this.displayName});
}

class MultiAction extends Action {
  final List<Action> actions;
  const MultiAction({required this.actions});
}

// =====================================================================
// Client -> Server
// =====================================================================

sealed class ClientMessage {
  const ClientMessage();
  Map<String, dynamic> toJson();
}

class HelloMessage extends ClientMessage {
  final int protocolVersion;
  final String deviceId;
  final String deviceName;
  final String token;

  const HelloMessage({
    required this.protocolVersion,
    required this.deviceId,
    required this.deviceName,
    required this.token,
  });

  @override
  Map<String, dynamic> toJson() => {
        'type': 'hello',
        'protocolVersion': protocolVersion,
        'deviceId': deviceId,
        'deviceName': deviceName,
        'token': token,
      };
}

class PairRequestMessage extends ClientMessage {
  final int protocolVersion;
  final String deviceId;
  final String deviceName;

  const PairRequestMessage({
    required this.protocolVersion,
    required this.deviceId,
    required this.deviceName,
  });

  @override
  Map<String, dynamic> toJson() => {
        'type': 'pair_request',
        'protocolVersion': protocolVersion,
        'deviceId': deviceId,
        'deviceName': deviceName,
      };
}

class ButtonPressMessage extends ClientMessage {
  final String pageId;
  final int row;
  final int col;

  const ButtonPressMessage({
    required this.pageId,
    required this.row,
    required this.col,
  });

  @override
  Map<String, dynamic> toJson() => {
        'type': 'button_press',
        'pageId': pageId,
        'row': row,
        'col': col,
      };
}

class PageChangeMessage extends ClientMessage {
  final String pageId;
  const PageChangeMessage({required this.pageId});

  @override
  Map<String, dynamic> toJson() => {'type': 'page_change', 'pageId': pageId};
}

class PongMessage extends ClientMessage {
  const PongMessage();
  @override
  Map<String, dynamic> toJson() => {'type': 'pong'};
}

// =====================================================================
// Server -> Client
// =====================================================================

sealed class ServerMessage {
  const ServerMessage();

  factory ServerMessage.fromJson(Map<String, dynamic> json) {
    switch (json['type']) {
      case 'welcome':
        return WelcomeMessage(
          protocolVersion: json['protocolVersion'] as int,
          agentName: json['agentName'] as String,
          profile: Profile.fromJson(json['profile'] as Map<String, dynamic>),
        );
      case 'profile_update':
        return ProfileUpdateMessage(
          profile: Profile.fromJson(json['profile'] as Map<String, dynamic>),
        );
      case 'pair_pending':
        return PairPendingMessage(requestId: json['requestId'] as String);
      case 'pair_accepted':
        return PairAcceptedMessage(token: json['token'] as String);
      case 'pair_rejected':
        return PairRejectedMessage(reason: json['reason'] as String);
      case 'ping':
        return const PingMessage();
      case 'error':
        return ErrorMessage(
          code: json['code'] as String,
          message: json['message'] as String,
        );
      default:
        throw FormatException('unknown server message type: ${json['type']}');
    }
  }
}

class WelcomeMessage extends ServerMessage {
  final int protocolVersion;
  final String agentName;
  final Profile profile;
  const WelcomeMessage({
    required this.protocolVersion,
    required this.agentName,
    required this.profile,
  });
}

class ProfileUpdateMessage extends ServerMessage {
  final Profile profile;
  const ProfileUpdateMessage({required this.profile});
}

class PairPendingMessage extends ServerMessage {
  final String requestId;
  const PairPendingMessage({required this.requestId});
}

class PairAcceptedMessage extends ServerMessage {
  final String token;
  const PairAcceptedMessage({required this.token});
}

class PairRejectedMessage extends ServerMessage {
  final String reason;
  const PairRejectedMessage({required this.reason});
}

class PingMessage extends ServerMessage {
  const PingMessage();
}

class ErrorMessage extends ServerMessage {
  final String code;
  final String message;
  const ErrorMessage({required this.code, required this.message});
}
