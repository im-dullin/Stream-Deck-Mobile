# 가상 스트림덱 (Stream Deck Virtual)

> 스마트폰을 Elgato Stream Deck처럼 사용해 PC를 제어하는 LAN-only 데스크톱+모바일 시스템. 물리장치 없이 무한 버튼, 한 번 페어링 후 자동 재연결, 한 버튼에 여러 동작 매핑.

![macOS](https://img.shields.io/badge/macOS-supported-blue) ![Windows](https://img.shields.io/badge/Windows-supported-blue) ![License](https://img.shields.io/badge/license-MIT-green)

---

## 이게 뭐예요?

- **PC 에이전트** (Tauri, Rust+React): 백그라운드 상주, LAN의 WebSocket 서버 + 모바일 웹 호스팅
- **모바일 데크** (Flutter web): 폰 브라우저로 접속, 그리드 버튼 탭 → PC에서 액션 실행
- **액션 타입**: 로컬 앱 실행 / URL 열기 / 한 버튼에 최대 10개 시퀀셜 (복합)
- **페어링**: 호스트·포트 입력 → PC에서 1회 승인 → 이후 영구 자동 재연결

## 빠른 시작 (설치형 사용)

1. [Releases](https://github.com/YOUR_ORG/streamdeck-virtual/releases) 에서 OS별 설치 파일 다운로드
   - macOS: `.dmg`
   - Windows: `.msi`
2. 설치 후 "가상 스트림덱" 실행 → 시스템이 네트워크 접근 권한 요청 → 허용
3. 폰 브라우저에서 `http://<PC의 LAN IP>:8090` 접속
4. 페어링 요청 → PC 윈도우 상단 배너에서 **승인**
5. 그리드의 빈 셀 클릭 → `+ 앱` 또는 `+ URL` 로 액션 추가

## 소스에서 빌드

### 사전 설치

| 도구 | macOS | Windows |
|---|---|---|
| Rust | `brew install rust` | `winget install Rustlang.Rustup` |
| Node 20+ | `brew install node` | `winget install OpenJS.NodeJS.LTS` |
| Flutter 3.41+ | `brew install --cask flutter` | `winget install Flutter.Flutter` |

확인:
```bash
rustc --version && node --version && flutter --version
```

### 빌드 + 실행

```bash
# 1) 모바일 웹 빌드 (에이전트에 임베드됨)
cd mobile
flutter pub get
flutter build web --release

# 2) 에이전트 의존성
cd ../agent
npm install

# 3) 개발 모드 (윈도우 열리고 핫리로드)
npm run tauri dev

# 또는 배포용 빌드 (설치 파일 생성)
npm run tauri build
# → src-tauri/target/release/bundle/ 에 .dmg / .msi 생성
```

## 디렉토리 구조

```
streamdeck-virtual/
├── schema/protocol.ts            # 와이어 프로토콜 단일 진실 소스
├── agent/                        # PC 에이전트 (Tauri 2.x)
│   ├── src-tauri/src/            # Rust: WS 서버, 액션 실행, 앱 디스커버리, 임베드 HTTP
│   └── src/                      # React 에디터 UI
├── mobile/                       # 모바일 데크 (Flutter Web)
│   └── lib/                      # Dart: 페어링·재연결·데크 UI
├── training/                     # AX 교육 자료 (한국어)
│   ├── 01-conversation-log.md    # AI 협업 워크플로우 회고
│   ├── 02-project-spec.md        # 프로젝트 아이템 정의서
│   └── 03-developer-guide.md     # 실행 가이드 + 트러블슈팅
└── .github/workflows/release.yml # 태그 push → 인스톨러 자동 빌드·릴리즈
```

## 아키텍처 핵심

```
[Mobile · Flutter Web]          [PC · Tauri Agent]
┌────────────────┐  WS :41234   ┌──────────────────────────┐
│ Deck Surface   │ ◄──────────► │ WS Server                │
└────────────────┘   JSON       │  - 페어링 승인 플로우     │
        ▲                       │  - 액션 디스패치          │
        │ HTTP :8090            ├──────────────────────────┤
        └────  serves  ◄────────│ Embedded Static Server   │
              build/web         │  - Flutter 웹 번들 임베드 │
                                ├──────────────────────────┤
                                │ Editor Window (Tauri)    │
                                │  - 그리드·인스펙터·피커  │
                                └──────────────────────────┘
```

- LAN 전용. 외부망 안 통함.
- 디바이스별 UUID 토큰 + PC 1회 승인 = 페어링.
- 폰 측 모든 액션 데이터는 PC `pairings.json` / `profile.json` 에 저장.

## 보안 주의

- 페어링은 같은 와이파이에 있는 누구나 시도 가능 (대신 PC에서 매번 명시 승인)
- 에이전트를 **관리자 권한으로 실행하지 마세요**
- 카페·공용 와이파이 환경에서는 사용 비권장 (LAN 전제)

## 라이선스

MIT. 자유롭게 사용·수정·배포 가능. 자세한 내용은 [LICENSE](./LICENSE).

## 기여

PR 환영. 큰 변경은 issue로 먼저 논의해주세요.

---

## AX 교육용 자료

본 레포지토리는 AX(AI Transformation) 교육 실습용으로 설계되었습니다. [`training/`](./training) 폴더의 자료 참고:

- [`01-conversation-log.md`](./training/01-conversation-log.md) — AI와 함께 이 프로젝트를 만든 실제 워크플로우 회고 + AX 협업 9원칙
- [`02-project-spec.md`](./training/02-project-spec.md) — 학생이 본인의 AI 에이전트에 입력으로 줄 수 있는 공식 spec + 난이도별 확장 과제
- [`03-developer-guide.md`](./training/03-developer-guide.md) — 사전설치 → 실행 → 트러블슈팅
