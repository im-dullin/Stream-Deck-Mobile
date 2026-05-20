# 프로젝트 아이템 정의서 — 가상 Stream Deck

> AX 교육 실습용 프로젝트. 학생들이 이 정의서를 입력 삼아 AI 코딩 에이전트와 함께 직접 구현해보거나, 본 레포지토리를 기반으로 기능을 확장하는 용도.

---

## 1. 한 줄 정의

**물리장치 없이 스마트폰을 Elgato Stream Deck처럼 사용해 PC를 제어하는 데스크톱+모바일 시스템.**

---

## 2. 배경 / 문제 정의

엘가토 Stream Deck은 자주 쓰는 매크로·앱 실행·OBS 컨트롤을 LCD 버튼에 매핑해 워크플로우를 가속하는 인기 디바이스다. 하지만:

- **하드웨어 비용** — 6~32버튼 모델이 ~15~25만원
- **버튼 수 제한** — 페이지·폴더로 우회하지만 물리적 그리드 크기 고정
- **휴대성** — USB 케이블 + 책상 점유

이미 모든 사용자가 들고 다니는 스마트폰을 Stream Deck 표면으로 쓰면:

- 비용 0원
- 무한 페이지/그리드 크기
- WiFi가 닿는 모든 곳에서 사용

본 프로젝트는 이 가설을 검증하는 MVP다.

---

## 3. 사용자 & 이해관계자

| 페르소나 | 동기 |
|---|---|
| 개발자/파워유저 | 자주 쓰는 앱·터미널·IDE 빠른 접근 |
| 스트리머/크리에이터 | OBS·사운드보드 매크로 (Stream Deck 직접 대체) |
| 일반 사무직 | Notion·Slack·Mail 등 자주 쓰는 앱의 진입장벽 0 |

MVP 단계 1순위 페르소나: **개발자/파워유저**.

---

## 4. MVP 범위

### 4.1 포함 (In-Scope)

- PC 에이전트가 시스템 트레이/창에 상주, LAN의 WebSocket 서버로 작동
- 모바일이 같은 LAN에서 접속, 그리드 형태로 버튼 표시, 탭→PC에서 액션 실행
- **액션 타입 3종**:
  - `launch_app` — 설치된 로컬 앱 실행 (예: `/Applications/Slack.app`)
  - `open_url` — OS 기본 브라우저로 URL 열기 (YouTube 플레이리스트 자동재생, Gmail/Notion/Slack 웹 등)
  - `run_command` — 임의 프로세스 스폰 (Python/Node/bash 스크립트). 카드뉴스 생성, 크롤링 같은 사용자 자동화 트리거. `~/` 자동 확장. 파이프·리다이렉트는 셸 스크립트로 감싸야 동작.
- PC 측 에디터로 버튼 매핑 구성 (그리드, 셀 선택, 인스펙터, 앱 피커)
- 페어링: **호스트/포트만 입력 + PC 측 1회 승인 플로우**. 토큰 입력 불필요 (PC가 발급해 폰에 자동 전달, `pairings.json` 영구 저장)
- 폰 클라이언트는 **웹** (브라우저로 접속). 네이티브 빌드는 학생 자율 과제로 이관
- 페어 디바이스: 다중 디바이스 페어링, 디바이스별 토큰 영구 저장·회수 가능 (디바이스 관리 UI는 v1.1)
- **복합 액션 (multi-action)**: 한 버튼에 최대 10개의 sub-action 매핑 가능, 순차 실행. 한 액션이 실패해도 다음 액션 계속.
- 설정 저장: 로컬 JSON 파일, 변경 시 모든 페어된 모바일에 라이브 푸시
- 앱 아이콘 자동 추출·표시 (macOS)

### 4.2 제외 (Out-of-Scope, v1.1+)

- 멀티액션의 고급 기능 (액션 간 지연, 조건부 실행, 액션 트리 편집) — 기본 시퀀셜 실행은 MVP에 포함
- 단축키/텍스트 입력/시스템 액션(볼륨/잠금/슬립)
- 컨텍스트 자동 프로파일 전환 (활성 앱에 따라 덱 자동 전환)
- 플러그인 SDK
- 클라우드 동기화/외부망 릴레이
- **mDNS / Bonjour 자동 디스커버리 (네이티브 모바일 앱)** — 프로토콜·저장은 미리 설계되어 있음. 학생이 직접 `flutter build apk` / `flutter build ios` + `nsd` 패키지 통합으로 확장 가능
- QR 페어링
- 다중 PC 페어링 (디스커버리)
- Windows/Linux 앱 디스커버리

---

## 5. 시스템 아키텍처

```
[Mobile · Flutter]          [PC · Tauri Agent]
┌────────────────┐  WS over LAN  ┌──────────────────────────┐
│ Deck Surface   │ ◄──────────► │ WS Server (tokio)        │
│  - Pairing UI  │   JSON msgs   │  - Hello/Welcome auth    │
│  - Grid View   │               │  - Profile broadcast     │
│  - Button tap  │ ──button───► │  - Ping/Pong keepalive   │
└────────────────┘   _press      ├──────────────────────────┤
                                  │ Action Runner            │
                                  │  - launch_app / open_url │
                                  ├──────────────────────────┤
                                  │ Editor (Tauri window)    │
                                  │  - Grid + Inspector      │
                                  │  - App Picker (sips)     │
                                  ├──────────────────────────┤
                                  │ Profile Store            │
                                  │  - JSON, atomic write    │
                                  └──────────────────────────┘
```

### 5.1 모듈 책임 분리

**Rust (agent/src-tauri/src/)**
- `protocol.rs` — 와이어 메시지 타입 (서버↔클라 양방향 enum)
- `ws_server.rs` — TcpListener + tungstenite, 페어링 인증, 메시지 디스패치
- `actions/` — 액션 trait + 구현. 확장 시 한 곳에 모듈 추가 + match 한 줄
- `app_discovery.rs` — macOS `.app` 스캔 + `sips` 동시 아이콘 추출
- `config.rs` — JSON 로드/저장, tmp→rename 원자적
- `lib.rs` — Tauri 빌더 + 커맨드 4종

**React (agent/src/)**
- `App.tsx` — 그리드/인스펙터/스테이터스바 컴포지션
- `components/` — Grid, Inspector, AppPicker, StatusBar
- `api/tauri.ts` — 타입드 invoke 래퍼
- `types/protocol.ts` — Rust와 미러되는 TS 타입

**Flutter (mobile/lib/)**
- `protocol/messages.dart` — sealed class 미러
- `services/ws_client.dart` — WS 연결 라이프사이클 + Profile 보관
- `services/pairing_store.dart` — SharedPreferences로 페어링 저장
- `pages/pairing_page.dart` / `deck_page.dart`
- `main.dart` — 라우팅 (페어링 ↔ 데크)

---

## 6. 와이어 프로토콜 (요약)

JSON over WebSocket. `type` 필드가 식별자. 양쪽이 같은 스키마를 미러.

```typescript
// 클라(폰) → 서버(PC)
type ClientMessage =
  | { type: "hello"; protocolVersion: 1; deviceId; deviceName; token }
  | { type: "pair_request"; protocolVersion: 1; deviceId; deviceName }
  | { type: "button_press"; pageId; row; col }
  | { type: "page_change"; pageId }
  | { type: "pong" };

// 서버(PC) → 클라(폰)
type ServerMessage =
  | { type: "welcome"; protocolVersion: 1; agentName; profile: Profile }
  | { type: "profile_update"; profile: Profile }
  | { type: "pair_pending"; requestId }
  | { type: "pair_accepted"; token }
  | { type: "pair_rejected"; reason }
  | { type: "ping" }
  | { type: "error"; code; message };

type Profile = {
  id; name; defaultPageId;
  pages: Array<{ id; name; rows; cols;
    buttons: Array<{ row; col; label?; iconBase64?;
      action: Action }> }>
}

type Action =
  | { type: "launch_app"; appPath; appName }
  | { type: "open_url"; url; displayName? }
  | { type: "run_command"; program; args: string[]; workingDir?; displayName? }
  | { type: "multi_action"; actions: Action[] };  // 최대 10, 순차 실행
```

흐름 (재연결, 페어 완료된 디바이스):
1. 클라 connect → `Hello { token }` 전송 (5초 안에)
2. 서버: `PairingDb`에서 `(deviceId, token)` 매칭 검증
3. 서버 → `Welcome` + 현재 프로필
4. 이후 양방향 메시지 + 30초 ping 키프얼라이브

흐름 (신규 페어링):
1. 클라 connect → `PairRequest` 전송 (토큰 없음)
2. 서버: pending request 등록, PC 에디터에 `pair_requested` 이벤트 emit
3. 서버 → `PairPending { requestId }`
4. PC 사용자가 에디터의 승인 배너에서 Approve/Reject 클릭 (60초 타임아웃)
5. 승인 시: 새 토큰 발급 → 디스크 저장 (`pairings.json`) → `PairAccepted { token }` 전송 → `Welcome` 후속
6. 거절/타임아웃 시: `PairRejected { reason }` 후 연결 종료

---

## 7. 기술 스택

| 레이어 | 선택 | 사유 |
|---|---|---|
| PC 코어 | Rust (Tauri 2.x) | OS API, 트레이/자동시작, 메모리 효율, Electron 대비 가벼움 |
| PC UI | React + Vite + TS | 타입 안전, 빠른 빌드, Tauri 내장 친화 |
| 모바일 | Flutter 3.x | iOS/Android 단일 코드베이스, 햅틱/메모리 이미지 자연스러움 |
| 통신 | WebSocket (LAN) | 양방향 저지연, ws/wss 자유, mDNS 친화 (v1.1) |
| 저장 | JSON 파일 (`~/Library/Application Support/StreamDeckVirtual/profile.json`) | MVP 단순, SQLite로 무중단 마이그레이션 가능 |
| 보안 | 디바이스별 UUID 토큰 + PC 1회 승인, TLS는 v1.1 | LAN 전제, MVP 적정 |

---

## 8. 비기능 요구사항

| 항목 | 목표치 |
|---|---|
| 버튼 탭 ↔ PC 실행 지연 | < 100ms (LAN) |
| 에이전트 메모리 | < 80MB (idle) |
| 에이전트 부팅 | < 2초 |
| 아이콘 스캔 (100개 앱) | < 1초 (sips 동시 8) |
| WS 클라이언트 동시 연결 | 8개 이상 (다중 페어링 대비) |
| 프로토콜 변경 시 호환성 | 메이저 버전 mismatch → 친절한 에러 메시지 |

---

## 9. 인수 기준 (Acceptance Criteria)

MVP가 다음을 모두 만족해야 한다:

- [ ] `cd agent && npm run tauri dev` 로 PC 에디터 윈도우 열림
- [ ] 윈도우 상단에 호스트/포트 + 페어 디바이스 수 표시
- [ ] 빈 셀 클릭 → 인스펙터 → "Pick an app" → 설치된 macOS 앱 목록 + 아이콘 표시
- [ ] 앱 선택 시 셀에 라벨·아이콘 표시, 자동 저장
- [ ] `flutter build web --release && python3 -m http.server` 로 폰 브라우저에서 페어링 화면 접속
- [ ] 호스트(IP 또는 `<hostname>.local`) + 포트(`41234`) 입력 → "Request pairing" → **PC 에디터에 페어 요청 배너 자동 표시**
- [ ] PC에서 Approve 클릭 → 폰이 자동으로 데크 화면으로 전환 (토큰은 PC가 발급, 폰에 자동 저장)
- [ ] 폰 새로고침 후에도 저장된 토큰으로 자동 재연결
- [ ] 폰에서 버튼 탭 → 햅틱 → PC에서 해당 앱 실행
- [ ] `cargo test` 8/8 통과 (프로토콜 + 페어 메시지 라운드트립)
- [ ] `flutter analyze` issues 0
- [ ] `tsc --noEmit` errors 0
- [ ] `flutter build web --release` 통과

---

## 10. 교육 학습 목표 (AX 관점)

학생이 이 실습을 마쳤을 때 다음을 할 수 있다:

1. **전략 협의를 AI와 진행** — 코드 작성 전에 form factor·네트워크 모델·MVP 범위를 트레이드오프 기반으로 좁힌다.
2. **AI에게 "프로덕션 품질 + 빠른 결과"를 동시에 요청** — 모노레포·단일 진실 소스·atomic write 같은 품질 토대를 처음에 깐다.
3. **AI의 산출물을 검증** — 빌드/테스트/정적분석으로 매 단계 신뢰도를 측정한다.
4. **AI와 배포 함정을 함께 진단** — 디버그 모드 vs 정적 배포, "내 머신 ≠ 사용자 머신" 같은 함정을 명확한 가설 → 수정으로 해결한다.
5. **AI 협업의 컨텍스트 관리** — 결정 사항을 memory/문서로 외부화해 다음 세션에서도 일관된 흐름을 유지한다.

---

## 11. 확장 과제 (학생 도전 옵션)

기본 MVP를 완료한 학생에게 권장:

### 난이도 ★

- **단축키 액션** — `Cmd+Shift+4` 같은 키스트로크 전송. `enigo` 크레이트 활용.
- **특정 브라우저 지정 URL 액션** — 현재는 OS 기본 브라우저로 열림. Chrome/Firefox 등 지정 옵션 추가 (`open -a "Google Chrome" <url>`).

### 난이도 ★★

- **앱 아이콘 캐시** — 매 호출마다 sips 돌지 않게 디스크 캐시 추가.
- **다중 페이지 + 스와이프** — 폰에서 `PageView` 좌우 스와이프, PC 에디터에 페이지 트리 사이드바.
- **다크/라이트 테마** — 에이전트 OS 테마 따라가기.
- **페어 디바이스 관리 UI** — `list_pairings` / `revoke_pairing` 커맨드는 이미 존재. 에디터에 페어된 디바이스 목록 + 회수 버튼 추가.

### 난이도 ★★★

- **네이티브 모바일 + mDNS 자동 디스커버리** (대표 챌린지) — Flutter `flutter build apk` / `flutter build ios` 파이프라인, `nsd` 패키지 통합, Android `CHANGE_WIFI_MULTICAST_STATE` / iOS `NSLocalNetworkUsageDescription`·`NSBonjourServices` 권한. Rust 측은 `mdns-sd` 크레이트로 `_streamdeck._tcp.local.` 광고 추가. **프로토콜·승인 플로우는 그대로 재사용**.
- **QR 페어링** — 에이전트가 QR(IP+포트 인코딩) 표시, 폰이 카메라로 스캔 (`mobile_scanner` 패키지). 승인 플로우 이전 단계 자동화.
- **고급 멀티액션** — sub-action 간 지연(`Delay { ms }`), 조건부 분기, 액션 트리 편집. MVP는 flat 시퀀셜 실행만 제공.

### 난이도 ★★★★

- **Windows/Linux 앱 디스커버리** — 시작메뉴 `.lnk` 파싱 / `.desktop` 파일 스캔.
- **플러그인 SDK** — 액션을 동적 라이브러리/스크립트로 로드.
