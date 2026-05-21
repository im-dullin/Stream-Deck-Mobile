# 개발자 실행 가이드

> 가상 Stream Deck 프로젝트를 본 머신에서 빌드·실행·시연하는 절차. macOS 기준.

---

## 1. 시스템 요구사항

- **OS**: macOS 11+ (Apple Silicon/Intel) 또는 **Windows 10 1803+ / 11** (x64)
- **램**: 8GB 이상 권장
- **디스크**: 약 4GB (Rust + Flutter + 빌드 캐시)
- **네트워크**: 폰과 같은 WiFi (LAN-only MVP)
- **Windows 전용 런타임**: **WebView2 Runtime** (Win11 기본 포함, Win10는 인스톨러가 자동 설치)

---

## 2. 사전 설치

### 2.A macOS

```bash
# Homebrew (없으면)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Toolchain
brew install rust node
brew install --cask flutter

# 확인
rustc --version && node --version && flutter --version
```

### 2.B Windows (PowerShell, 일반 권한)

**중요 순서**: Rust → MSVC Build Tools → Node → Flutter → 셸 재시작.

```powershell
# 1) Git
winget install -e --id Git.Git

# 2) Rust — rustup-init.exe 다운받아 실행 (https://rustup.rs)
#    설치 중 MSVC 미감지 시 자동으로 Visual Studio Build Tools 설치 안내
#    수동 설치: winget install -e --id Rustlang.Rustup
winget install -e --id Rustlang.Rustup

# 3) Visual Studio Build Tools (C++ 워크로드)
#    Tauri의 Rust 코드를 Windows 네이티브로 링크하려면 필수
winget install -e --id Microsoft.VisualStudio.2022.BuildTools
#    설치 후: Visual Studio Installer → Modify → "Desktop development with C++" 워크로드 체크 → Install

# 4) Node.js LTS
winget install -e --id OpenJS.NodeJS.LTS

# 5) Flutter — winget에 공식 패키지 없음, 수동 권장
#    https://docs.flutter.dev/get-started/install/windows → SDK zip 다운로드
#    C:\flutter 에 압축 해제 (경로에 공백 X)
#    제어판 → 시스템 → 고급 시스템 설정 → 환경변수 → Path에 C:\flutter\bin 추가
```

**셸 재시작 후 확인**:
```powershell
git --version          # git version 2.x
rustc --version        # rustc 1.95+
cargo --version        # cargo 1.95+
node --version         # v20.x
flutter --version      # Flutter 3.x
flutter doctor         # Windows toolchain + Chrome 만 ✓ 필요. Android/iOS는 ! 무시 가능
```

> **WebView2 Runtime**: 인스톨러로 배포된 .msi 는 WebView2 를 내장(`offlineInstaller` 모드)해서 자동 설치. 소스 빌드 / `npm run tauri dev` 환경은 Win10 1803+/Win11 의 기본 Edge가 제공하는 WebView2 사용. 그래도 깔려있지 않다는 에러가 뜨면 https://developer.microsoft.com/en-us/microsoft-edge/webview2/ 에서 "Evergreen Standalone Installer" 다운로드.

---

## 3. 프로젝트 초기 설정

```bash
cd /path/to/011_stream_deck
git status   # git 초기화돼 있어야 함

# 에이전트 (Rust + React) 종속성
cd agent
npm install

# 모바일 (Flutter) 종속성
cd ../mobile
flutter pub get
```

---

## 4. PC 에이전트 실행

### 4.1 개발 모드 (Tauri 윈도우 핫리로드)
```bash
cd agent
npm run tauri dev
```
- 첫 빌드: 5~10분 (Rust 의존성 컴파일)
- 이후 빌드: 5~30초 (incremental)
- 빌드 완료 후 "Stream Deck Virtual" 윈도우가 자동으로 뜬다

### 4.2 윈도우에서 확인할 것
- 상단 칩 3개: `Host` / `Port` / `Token` — 폰 페어링에 그대로 사용
- 가운데: 5×3 빈 그리드
- 우측 인스펙터는 셀 선택 시 활성화

---

## 5. 모바일 데크 실행 (웹)

본 MVP는 **폰 브라우저로 접속하는 웹** 형태로 진행. 네이티브 빌드(Android/iOS)는 학생 자율 확장 과제. ([02-project-spec.md 11장](02-project-spec.md) 의 ★★★ 챌린지 참고.)

### 5.1 배포용 — Tauri 에이전트에 임베드 (Python 서버 불필요)

빌드 시 모바일 웹이 에이전트 바이너리에 임베드됩니다. 따로 정적 서버를 띄울 필요 없음.

```bash
cd mobile
flutter build web --release       # 한 번 빌드해두면 에이전트가 자동 포함
```

그 다음 에이전트 (`npm run tauri dev` 또는 인스톨러로 설치한 앱) 실행 → 폰 브라우저에서 바로:

```
http://<맥-LAN-IP>:8090
```

### 5.2 (참고) 모바일 핫리로드 개발

Flutter UI 자체를 수정 중일 땐 Tauri 임베드 우회하고 직접:

```bash
cd mobile
flutter run -d chrome --web-port 8091   # 다른 포트로
```

(맥 자신의 Chrome에서만 보임. 폰에서는 5.1 방식으로 접속.)

---

## 6. 페어링 (1회 승인 플로우)

1. 폰 브라우저 → `http://<맥-LAN-IP>:8090`
2. PC host (IP 또는 `<hostname>.local`) + Port(`41234`) 입력 → **Request pairing** 탭
3. **맥 Tauri 윈도우 상단에 페어 요청 배너 자동 등장**: `📱 Mobile Deck wants to pair  [Reject]  [Approve]`
4. **Approve** 클릭 → 폰이 자동으로 데크 화면으로 전환 (토큰은 PC가 발급해 폰에 자동 저장)
5. 폰 새로고침 후에도 저장된 토큰으로 자동 재연결 (재승인 불필요)

### 6.x 페어 해제 (토큰 회수)
- 폰 측: 데크 화면 우상단 로그아웃 아이콘 → 폰의 페어링 캐시 삭제
- PC 측 (전체 초기화): `~/Library/Application Support/StreamDeckVirtual/pairings.json` 삭제 후 에이전트 재시작

---

## 7. 첫 버튼 설정 → 시연

1. 맥 Tauri 윈도우에서 **빈 셀 하나 클릭**
2. 우측 인스펙터 하단 `+ App` 또는 `+ URL` 버튼 선택
   - **앱 액션**: 설치된 앱 목록(아이콘 포함)에서 선택
   - **URL 액션**: URL 입력 (예: `https://www.youtube.com/playlist?list=...` ← 자동 재생됨)
3. 셀에 라벨이 자동 입력됨 (앱 이름 또는 URL 호스트명)
4. **여러 액션 추가**: 같은 셀에 계속 추가하면 최대 10개까지 묶임. 순차 실행됨.
   - 예: "Lofi 시작" 버튼 = [YouTube 플레이리스트 열기] + [Slack DND 켜기 (앱)] + ...
5. 폰에서도 같은 셀이 즉시 갱신 (라이브 푸시)
6. **폰에서 그 셀을 탭** → 햅틱 → PC에서 모든 액션 순차 실행 ✨

---

## 8. 트러블슈팅

### 8.0 Windows 자주 부딪히는 문제

| 증상 | 원인 | 해결 |
|---|---|---|
| `rustc: command not found` (PowerShell 새로 띄워도) | PATH 갱신 안 됨 | 로그아웃·재로그인. 또는 `$env:Path` 직접 확인 |
| `link.exe not found` 빌드 에러 | MSVC Build Tools 없음 | VS Build Tools 설치 + "Desktop development with C++" 워크로드 + 재부팅 |
| `flutter doctor` 에서 Visual Studio 미감지 | C++ 워크로드 누락 | VS Installer → Modify → "Desktop development with C++" 추가 |
| 폰에서 빈 화면 / 연결 시 timeout | Windows Defender 방화벽 차단 | 방화벽 팝업이 떴을 때 "개인 네트워크"+"액세스 허용" 체크. 놓쳤으면 제어판 → Windows Defender 방화벽 → 인바운드 규칙에서 "가상 스트림덱" 찾아 허용 |
| Chrome 다운로드가 `확인되지 않음 ...crdownload` 로 멈춤 | Chrome Safe Browsing 이 미서명 .msi 차단 | `Ctrl+J` → 해당 항목 → "위험한 파일이지만 계속" / "보관" 클릭. Edge 도 유사. Firefox 는 보통 한 번에 받아짐 |
| 인스톨러 실행 시 "Windows의 PC 보호" 경고 | 코드사이닝 없음 (오픈소스 SW 정상) | "추가 정보" → "실행" 클릭 |
| `npm run tauri dev` 가 5분 넘게 안 끝남 | 첫 의존성 다운로드 중 (Rust 크레이트 ~500개 컴파일) | 10분 인내. `target/` 캐시 생긴 이후로는 빠름. 중간에 끊으면 다시 받음 |
| WebView2 missing 에러 (Win10 구버전) | WebView2 Runtime 부재 | https://developer.microsoft.com/en-us/microsoft-edge/webview2/ → Evergreen 다운로드 |
| 응용 프로그램 선택 모달이 빈 목록 | PowerShell ExecutionPolicy 차단 또는 시작메뉴 비어있음 | `Get-ExecutionPolicy` 확인. `Restricted`면 `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser` |
| V3·알약 등 국내 AV가 .exe 차단 | 미서명 Rust 바이너리 휴리스틱 false-positive | AV 예외 추가 (학생 본인 책임) 또는 소스 빌드로 본인 머신에서 컴파일 |

### 8.1 폰에서 흰 화면만 보임
**원인**: `flutter run -d chrome` 디버그 모드는 외부 디바이스에서 동작 안 함.
**해결**: 위 5.1 정적 빌드 방식으로 교체.

### 8.2 Connect 시 "not_paired" 또는 무응답
- 폰이 이전에 페어된 적 있는데 PC의 `pairings.json` 이 초기화되었을 가능성 → 폰 앱에서 disconnect 후 재페어
- 포트 `41234` 인지 확인 (기본값)
- 맥 방화벽: 시스템 설정 → 네트워크 → 방화벽 → OFF 또는 Python/Tauri 허용


### 8.3 같은 LAN인데 폰에서 PC IP가 안 보임
- 공유기의 AP 격리(클라이언트 분리) 옵션 OFF 확인
- 게스트 WiFi에 폰만 붙어 있지 않은지 확인
- 카페·공용 WiFi면 핫스팟으로 대체

### 8.4 Tauri 빌드 실패 — `Permission opener:default not found`
- `agent/src-tauri/capabilities/default.json` 의 `permissions` 배열에서 `opener:default` 가 남아있다면 제거.

### 8.5 Rust 빌드 실패 — `tokio::process` feature gated
- `Cargo.toml` 의 `tokio` features에 `"process"` 가 포함됐는지 확인.

### 8.6 Flutter analyze — `Page is ambiguous`
- `import 'package:flutter/material.dart' hide Page;` 가 적용됐는지 확인.

### 8.7 앱 아이콘이 안 뜨는 앱
- 일부 시스템 앱은 `.icns` 가 비표준 위치에 있거나 없을 수 있음. 라벨만 표시되는 게 정상 동작.

---

## 9. 디렉토리 구조

```
011_stream_deck/
├── schema/protocol.ts              # 와이어 프로토콜 (단일 진실 소스)
├── agent/                          # PC 에이전트 (Tauri)
│   ├── src-tauri/
│   │   ├── Cargo.toml
│   │   ├── capabilities/default.json
│   │   ├── tauri.conf.json
│   │   └── src/
│   │       ├── lib.rs              # Tauri 빌더 + 커맨드
│   │       ├── protocol.rs         # 와이어 타입 + 6종 단위테스트
│   │       ├── ws_server.rs        # WebSocket 서버
│   │       ├── actions/            # 액션 trait + launch_app
│   │       ├── app_discovery.rs    # macOS 앱 스캔 + sips 아이콘
│   │       └── config.rs           # JSON 저장
│   └── src/                        # React 에디터
│       ├── App.tsx
│       ├── App.css
│       ├── api/tauri.ts
│       ├── components/             # Grid, Inspector, AppPicker, StatusBar
│       └── types/protocol.ts       # Rust와 미러
├── mobile/                         # Flutter 데크
│   └── lib/
│       ├── main.dart
│       ├── protocol/messages.dart
│       ├── services/               # ws_client, pairing_store
│       └── pages/                  # pairing_page, deck_page
└── training/                       # AX 교육자료
    ├── 01-conversation-log.md
    ├── 02-project-spec.md
    └── 03-developer-guide.md
```

---

## 10. 자주 쓰는 명령 모음

```bash
# Rust 측 검증
cd agent/src-tauri
cargo check
cargo test --lib

# TypeScript 빌드
cd agent
npm run build           # tsc + vite

# Flutter 정적분석
cd mobile
flutter analyze

# Flutter 단위테스트 (있을 경우)
flutter test

# 데모 풀스택 실행 (모바일 웹은 에이전트에 임베드되므로 단일 명령)
cd mobile && flutter build web --release         # 1회 (Flutter 변경 시 재실행)
cd ../agent && npm run tauri dev                 # 에이전트 = WS + 임베드 HTTP 동시 제공
```

---

## 11. 환경 변수

- `STREAMDECK_TOKEN` — 에이전트 부팅 시 사용할 페어링 토큰 (지정 안 하면 UUID 자동 생성). 교육 현장에서 모든 학생이 같은 토큰을 쓰고 싶을 때 유용.
- `RUST_LOG` — 트레이싱 로그 레벨. 기본 `info,agent_lib=debug`.

예:
```bash
STREAMDECK_TOKEN=demo-class-2026 RUST_LOG=debug npm run tauri dev
```
