# 가상 Stream Deck — AI 협업 워크플로우 회고

> 한 세션 동안 AI 코딩 에이전트(Claude Code, Opus 4.7)와 함께 "가상 Stream Deck" MVP를 빈 디렉토리에서 완성한 과정의 기록.
> 사용자가 무엇을 어떤 순서로 지시했고, 어떤 의사결정 지점이 있었고, 어디서 막혔는지를 시간순으로 정리한다.

---

## 0. 출발점

- 작업 디렉토리: `011_stream_deck/` (비어 있음)
- 호스트: macOS, Node 24.x, Homebrew 설치됨
- 사전 설치 상태: Rust ✗, Flutter ✗
- 사용자 의도: Elgato Stream Deck 의 동작방식을 차용한 가상 데크 MVP

---

## 1. 전략 협의 단계 (코드 0줄)

### 사용자 프롬프트 #1

> 현재 디렉토리에 신규 프로젝트를 하나 만들거야. 엘가토 스트림덱 제품은 개별 LCD 버튼에 ... 어떻게 프로젝트 구현하고 서비스 형태는 어떻게 하고 기술스택과 최종 페이지 구성안까지 전략 같이 논의해보자.

**AI의 첫 동작**: 코드 작성 대신 **3개의 트레이드오프 질문**을 던졌다.

- 서비스 형태 (데스크톱 단독 / 데스크톱+모바일 컴패니언 / 웹+로컬 에이전트)
- 타겟 사용자 (개발자·파워유저 / 스트리머·크리에이터 / 일반)
- MVP 범위 (최소 / 표준 / 풀스펙)

각 보기에는 ASCII 다이어그램 preview를 붙여 시각적 비교를 가능하게 했다.

> **AX 관점 교훈 ①** — 전략적 결정은 코드보다 먼저 좁힌다. 다지선다 + preview 시각화로 의사결정 비용을 줄인다.

### 사용자 프롬프트 #2 (재포커싱)

> 모바일에서 덱 화면과 세팅된 버튼을 클릭하면 PC에서 열리도록 하고싶고, 그 버튼 세팅은 웹이나 피씨 로컬에서 하면 좋겠는데, elgato stream deck 제품의 동작방식을 참고해서 기획해주고 협의해보자.

사용자가 form factor를 직접 명시 — **모바일 = 덱, PC = 두뇌, 에디터 = 리모컨**.

AI는 즉시:
- Elgato Stream Deck Mobile + Stream Deck SW 구조를 차용한 다이어그램 제시
- "가져올 기능 vs 버릴 기능" 비교표 작성
- 후속 분기점 3개 제시:

| 분기 | 사용자 선택 |
|---|---|
| 모바일↔PC 네트워크 | **LAN 전용** (서버비 0, 외부망 미지원) |
| 에디터 1차 진입점 | **네이티브 (Tauri 윈도우)** |
| 모바일 배포 형태 | **Flutter 단일 코드베이스** |

이어서 MVP 범위와 수익모델 분기:

| 분기 | 사용자 선택 |
|---|---|
| 수익모델 | 미정 — MVP 무료, 구조적으로 추가 가능하게 |
| 액션 타입 | **로컬앱 실행 단 1개** |

> **AX 관점 교훈 ②** — 사용자가 결정 못 한 부분은 "미정"으로 명확히. AI가 추정으로 채우면 추정 위에 코드가 쌓인다.

이 시점에 AI는 결정사항을 **프로젝트 메모리**에 저장(LAN-only, Tauri+Flutter, app-launch MVP). 다음 세션에서 같은 질문을 다시 받지 않도록.

---

## 2. 실행 모드 전환

### 사용자 프롬프트 #3

> 그냥 최대한 빨리 결과물 볼 수 있게 만들어주고, 급하다해도 코드 품질을 프로덕션급으로 유지해야해.

**해석**: 코드 시작. 단 빠름과 품질 양립.

AI의 응답 구조:
- **3가지를 처음부터 못 박음**:
  - 모노레포 (`agent/` + `mobile/` + `schema/`)
  - 와이어 프로토콜 = 단일 진실 소스 (`schema/protocol.ts`)
  - 액션 = 다형성 (`Action` discriminated union, 새 액션 추가 시 한 곳 수정)
- 툴체인 확인 → Rust/Flutter 미설치 발견
- **시스템 전역 설치는 사용자 동의 필요** → AskUserQuestion으로 확인:
  - "Rust+Flutter 설치 (단, 스캐폴딩 병행)" 선택
- 설치 둘 다 백그라운드 시작 + 비차단 작업(.gitignore, 와이어 프로토콜) 병행 작성

> **AX 관점 교훈 ③** — "그냥 해줘"라도 시스템 변경(brew install)은 한 번 더 합의. 설치 대기 시간(20~30분)에 의미 있는 작업을 병행.

---

## 3. 스캐폴딩 + 핵심 코드 작성

설치 진행 중 작성된 파일:
- `.gitignore`, `git init` (1일차부터 git)
- `schema/protocol.ts` — TypeScript로 메시지/도메인 타입 정의 (Rust·Dart가 이걸 미러)

Rust 설치 완료 후:
```bash
npm create tauri-app@latest agent -- --template react-ts --manager npm --identifier com.streamdeckvirtual.agent -y
```
- `Cargo.toml` 에 tokio, tokio-tungstenite, anyhow, tracing, local-ip-address, dirs 추가
- 모듈 6개 일괄 작성:
  - `protocol.rs` — 와이어 타입 + 라운드트립 테스트 6종
  - `actions/{mod, launch_app}.rs` — 액션 trait + macOS/Win/Linux 분기
  - `app_discovery.rs` — `/Applications` 스캔
  - `config.rs` — 원자적 저장 (tmp + rename)
  - `ws_server.rs` — tokio-tungstenite 기반 WS 서버 + Hello/Welcome/Ping/Error 흐름
  - `lib.rs` — Tauri 부팅 + 커맨드 4개

Flutter 설치 완료 후:
```bash
flutter create --platforms=web,ios,android --org com.streamdeckvirtual --project-name streamdeck_mobile mobile
```
- `pubspec.yaml` 에 web_socket_channel, shared_preferences 추가
- 모듈 일괄 작성:
  - `protocol/messages.dart` — Dart sealed class 미러
  - `services/{ws_client, pairing_store}.dart` — WS 연결 관리 + SharedPreferences 페어링 저장
  - `pages/{pairing, deck}_page.dart`
  - `main.dart` — 페어링/덱 라우팅

> **AX 관점 교훈 ④** — 큰 변화량은 한 턴에 묶어서 작성하고, 작성 직후 일괄 검증으로 들어간다. 작은 변경마다 멈추면 컨텍스트 비용이 누적된다.

---

## 4. 검증 — 4중 자동 체크 병행

작성 후 백그라운드로 동시 실행:

| 도구 | 대상 | 결과 |
|---|---|---|
| `cargo check` | Rust 컴파일 | 1차 실패 → 수정 → 그린 |
| `cargo test` | 프로토콜 라운드트립 6종 | 6/6 통과 |
| `tsc --noEmit` | TypeScript 타입 검사 | 통과 |
| `flutter analyze` | Dart 정적 분석 | 1차 5건 실패 → 수정 → 그린 |
| `npm run build` | Vite 프로덕션 번들 | 통과 |

### 막힌 곳 3선

**1. Tauri capabilities 미정합**
- `tauri-plugin-opener` 를 `Cargo.toml` 에서 제거했더니 `capabilities/default.json` 의 `opener:default` 권한이 미해결.
- 해결: `capabilities/default.json` 에서 권한 줄 제거.

**2. `tokio::process` feature gate**
- `tokio::process::Command` 가 "the item is gated behind the `process` feature" 에러.
- 해결: `Cargo.toml` 의 tokio features에 `"process"` 추가.

**3. Flutter ambiguous import**
- `Page` 가 `flutter/material` 의 Page 위젯과 내 protocol의 Page 양쪽에 존재. `ConnectionState` 도 동일.
- 해결: `import 'package:flutter/material.dart' hide Page;` / `hide ConnectionState;`.

> **AX 관점 교훈 ⑤** — 빌드 검증은 작업 끝이 아니라 작업 중간에 계속 돌린다. 5분 안에 도는 피드백 루프 > 한 번에 모아서 디버깅.

---

## 5. 첫 시연

```bash
# 터미널 1 — PC 에이전트
cd agent && npm run tauri dev

# 터미널 2 — 모바일 데크 (브라우저로 폰 접속)
cd mobile && flutter run -d chrome --web-port 8090 --web-hostname 0.0.0.0
```

Tauri 윈도우 표시:
- 상단 페어링 칩 3개 (Host / Port / Token) — 클릭으로 복사
- 가운데 5×3 그리드
- 우측 인스펙터 (셀 선택 시 펼침)

→ 빈 셀 클릭 → "Pick an app" → 설치된 앱 선택 → 자동 저장.

---

## 6. 배포 함정 — 폰에서 흰 화면

### 사용자 프롬프트 #4

> http://<맥-LAN-IP>:8090 주소를 모바일에서 접속하면 빈 흰색 화면만 나오는데?

**원인 진단**:
- HTML은 정상 수신 (= 방화벽/네트워크 OK)
- JS 부팅 실패 = `flutter run -d chrome --web-hostname 0.0.0.0` 의 디버그 서비스는 핫리로드용 WebSocket이 `127.0.0.1` 에 묶여 있어 외부 디바이스에서 부팅 못 함

**해결**: 정적 빌드 + 단순 정적 서버로 교체.

```bash
flutter build web --release
python3 -m http.server 8090 --bind 0.0.0.0 --directory build/web
```

디버그 서비스 의존성 0, 폰 시연 안정성 ↑.

> **AX 관점 교훈 ⑥** — "내 머신 브라우저에서 됨" ≠ "다른 디바이스에서 됨". 디버그 모드 vs 정적 배포의 차이를 미리 안다.

---

## 7. 폴리시 — 앱 아이콘

### 사용자 프롬프트 #5

> Pick an application에서 응용 프로그램들의 아이콘을 같이 볼 수는 없어?

**구현 선택**:
- 옵션 A: 순수 Rust 디코더 (`icns` 크레이트) — 의존성 추가, 변형 처리 까다로움
- 옵션 B: macOS 내장 `sips` 서브프로세스 — 신뢰성↑, ~50ms/app, 의존성 0

→ **옵션 B 선택**: `sips -s format png -Z 128 input.icns --out output.png`. `tokio::process` + 세마포어로 동시성 8 제한, 100개 앱 ~600ms.

확장:
- `InstalledApp.iconBase64` (Rust) → JSON으로 React 전달 → 그리드/인스펙터/모바일 데크 모두 렌더
- 모바일은 `Image.memory(base64Decode(...))` 로 표시 — 같은 base64가 PC↔폰 일관

---

## 8. Iteration 2 — 1회 승인 페어링 (토큰 입력 제거)

### 사용자 프롬프트 #6
> 지금은 사용하려면 컴퓨터에서 토큰을 확인하고 접속해야 하는데 모바일에서 같은 네트워크에 접속해있는 PC 기기를 찾아 바로 연결할 수 있도록 할 수 있을까? 같이 협의해보자.

**제약 인식**: 브라우저(웹 폰 클라이언트)는 LAN 멀티캐스트/mDNS가 불가능. 자동 디스커버리를 하려면 **네이티브 앱**이 필요. AI는 이 제약을 명시하고 3가지 옵션 제시:

| 옵션 | 변경 규모 | UX |
|---|---|---|
| A. 웹 유지 + 토큰만 제거 (PC 1회 승인) | 작음 | 부분 자동 |
| B. 네이티브 + mDNS (Plex 스타일) | 중간 | 완전 자동 |
| C. 단계적 (A → B) | 분산 | — |

사용자 1차 선택: **B**.

### 구현 (1차)

**Rust 백엔드**
- `pairings.rs` — 디바이스별 토큰 저장(`pairings.json`, atomic write)
- 프로토콜 확장: `PairRequest`, `PairPending`, `PairAccepted`, `PairRejected`
- `ws_server.rs` — 첫 메시지가 `Hello`면 토큰 검증, `PairRequest`면 pending 등록+이벤트 emit+승인 대기(60s)
- Tauri 커맨드: `approve_pair`, `reject_pair`, `list_pairings`, `revoke_pairing`
- 이벤트 채널: ws_server → mpsc → lib.rs forwarder → Tauri Event Bus → React
- (mDNS 광고: `mdns-sd` 크레이트로 `_streamdeck._tcp.local.`)

**React 에디터**
- `PairingBanner` 컴포넌트가 `pair_requested` 이벤트 listen
- 동시 여러 페어 요청 처리(배너 스택)
- `StatusBar` 에서 토큰 칩 제거 (더 이상 사용자가 입력 안 함)

**Flutter** (네이티브 1차 구현)
- `nsd` 패키지로 `_streamdeck._tcp` browse + 신규 `DiscoveryPage`
- Android `CHANGE_WIFI_MULTICAST_STATE` / iOS `NSLocalNetworkUsageDescription`·`NSBonjourServices` 권한
- `kIsWeb` 분기로 웹 폴백 → `PairingPage`
- `WsClient.requestPair()` — `PairRequest` → `PairPending` → 승인 대기 → `PairAccepted` (token 콜백) → `Welcome`

### 사용자 프롬프트 #7 — 스코프 재정의

> 실제 sdk와 앱으로 전향하는건 제가하고 웹 폰에서 플로우 검증하는 것까지만 프로젝트를 제안하자. 이것만으로도 토이 프로젝트 실습으로는 최적일 것 같아. 불필요한 코드 제거해주고.

**판단**: 토이 프로젝트 학습 가치는 (1) 프로토콜 설계, (2) WS + 승인 플로우, (3) 모노레포 + 검증 루프 — 여기까지로 충분. 네이티브 빌드는 OS·서명·SDK·entitlement 학습으로 별개의 깊이가 필요해서 다음 차시 과제로 분리.

### 정리 (제거됨)

- Rust: `mdns-sd` 의존성, `mdns.rs` 모듈, lib.rs의 광고 호출
- Flutter: `nsd` 패키지, `discovery_page.dart`, `kIsWeb` 분기
- 모바일 네이티브 설정: Android `CHANGE_WIFI_MULTICAST_STATE`, iOS `NSLocalNetworkUsageDescription` + `NSBonjourServices`

### 유지된 것 (학생 확장의 토대)

- 페어링 프로토콜 (`PairRequest` / `PairPending` / `PairAccepted` / `PairRejected`) — **다음 차시에 mDNS만 얹으면 그대로 재사용**
- `pairings.json` 디바이스별 토큰 저장 — 다중 디바이스 페어링 그대로 지원
- 승인 UI (PC 측 배너) — 디스커버리든 수동이든 진입 방식만 다르고 결말은 동일

### 막힌 곳

1. **Tauri 비동기 커맨드 반환 타입** — `async fn get_agent_status -> AgentStatus` 컴파일 실패. 비동기 Tauri 커맨드는 반드시 `Result<T, E>` 반환. → 수정.
2. **nsd 패키지 버전** — `^2.5.0` 으로 시작했으나 그 버전 없음. pub.dev 권고대로 `^5.0.1` 적용 (이후 제거).
3. **Flutter `Page` / `ConnectionState` 충돌** — 도메인 모델과 Flutter 위젯 이름 충돌. `hide` 임포트로 해결.

### 검증 결과 (최종)

| 검증 | 결과 |
|---|---|
| `cargo check` | ✅ |
| `cargo test --lib` | ✅ 8/8 (페어 메시지 라운드트립 2종 추가) |
| `npm run build` | ✅ |
| `flutter analyze` | ✅ no issues |
| `flutter build web --release` | ✅ |

> **AX 관점 교훈 ⑧** — 열린 질문("할 수 있을까?")은 **제약부터** 짚는다. 옵션 비용을 알아야 의미 있는 선택.
>
> **AX 관점 교훈 ⑨** — **만들 줄 알면 멈출 줄도 알아야**. 토이/학습 프로젝트에선 더더욱. 한 번 만든 코드를 다시 빼는 결정은 처음부터 안 만드는 결정만큼 중요하고, 학생의 학습 부담을 결정한다.

---

## 9. AX 협업에서 추출한 9가지 원칙

1. **전략부터, 코드는 마지막** — 한 줄 쓰기 전에 form factor·네트워크·액션 범위를 못 박는다.
2. **트레이드오프를 시각화하라** — preview 곁들인 다지선다는 의사결정 비용을 1/10로 줄인다.
3. **"아직 미정"을 인정하라** — 사용자가 모르는 답을 AI가 추정으로 채우면, 그 추정 위에 코드가 쌓인다.
4. **시스템 변경은 확인** — `brew install` 같은 글로벌 변경은 한 번 더 합의.
5. **대기 시간을 일하는 시간으로** — 설치/빌드 동안 비차단 작업을 병행.
6. **검증 루프를 짧게** — `cargo check`, `tsc`, `flutter analyze` 가 작업 진행 동안 계속 돌게 둔다.
7. **개발과 배포의 함정을 미리 안다** — 자기 머신 ≠ 사용자 머신, 디버그 모드 ≠ 프로덕션.
8. **"할 수 있을까?"는 제약부터** — 열린 질문에 무조건 "Yes" 로 답하지 않는다. 브라우저의 LAN 제약처럼 근본적 한계를 먼저 제시하고 옵션을 펼친다.
9. **만들 줄 알면 멈출 줄도 알아야** — 한 번 만든 코드를 다시 빼는 결정은 처음부터 안 만드는 결정만큼 중요. 토이/학습 프로젝트일수록 스코프 디시플린이 학습 효율을 결정한다.

---

## 10. 산출물 요약

```
011_stream_deck/
├── schema/protocol.ts            # 와이어 프로토콜 단일 진실 소스
├── agent/                        # Tauri PC 에이전트 (Rust + React)
│   ├── src-tauri/src/            # protocol / actions / app_discovery / config / ws_server / lib
│   └── src/                      # 에디터 UI (Grid / Inspector / AppPicker / StatusBar)
├── mobile/                       # Flutter 데크 (iOS/Android/Web)
│   └── lib/                      # protocol / services / pages
└── training/                     # 이 문서들 (AX 교육용)
```

- Rust 단위 테스트 6개 (프로토콜 라운드트립)
- TypeScript 타입 검사 clean
- Flutter analyze clean
- 1차 시연 = "폰에서 버튼 누르면 PC에서 앱 실행" 통과

소요 시간 (실효): 약 2시간 (툴체인 설치 30분 포함).
