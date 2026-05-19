# 가상 스트림덱 (Stream Deck Virtual)

> 스마트폰을 Elgato Stream Deck처럼 사용해 PC를 제어하는 LAN 전용 데스크톱+모바일 시스템. 물리장치 없이 무한 버튼, 한 번 페어링 후 자동 재연결, 한 버튼에 여러 동작 매핑.

![macOS](https://img.shields.io/badge/macOS-supported-blue) ![Windows](https://img.shields.io/badge/Windows-supported-blue) ![License](https://img.shields.io/badge/license-MIT-green)

---

## 이게 뭐예요?

- **PC 에이전트** (Tauri, Rust+React): 백그라운드 상주, LAN의 WebSocket 서버 + 모바일 웹 호스팅
- **모바일 데크** (Flutter web): 폰 브라우저로 접속, 그리드 버튼 탭 → PC에서 액션 실행
- **액션 타입**: 로컬 앱 실행 / URL 열기 / 한 버튼에 최대 10개 시퀀셜 (복합)
- **페어링**: 호스트·포트 입력 → PC에서 1회 승인 → 이후 영구 자동 재연결

---

# 🟢 트랙 1 — 그냥 써보고 싶은 사용자 (비개발자 OK)

설치 도구·빌드 환경 전혀 필요 없음.

## A. PC에 에이전트 설치

1. https://github.com/im-dullin/Stream-Deck-Mobile/releases 접속
2. 본인 OS 맞는 파일 다운로드:
   - **macOS**: `가상 스트림덱_x.y.z_aarch64.dmg` (M1/M2/M3) 또는 `_x64.dmg` (Intel)
   - **Windows**: `가상 스트림덱_x.y.z_x64_ko-KR.msi`
3. 더블클릭으로 설치

> **⚠️ Releases가 비어있다면**: 아직 첫 릴리즈 전입니다. 강사가 `git tag v0.1.0 && git push origin v0.1.0` 한 번 실행하면 GitHub Actions 가 자동 빌드해서 ~10분 후 Releases에 첨부됩니다. 그동안은 아래 **트랙 2** 로 진행.

### macOS 첫 실행

- **Launchpad** 또는 **Spotlight** (`Cmd+Space`) → "가상 스트림덱" 검색 → 실행
- "확인되지 않은 개발자" 경고 시: **시스템 설정 → 개인정보 보호 및 보안** → "그래도 열기"
- 네트워크 권한 팝업 → 허용

### Windows 첫 실행

- 시작 메뉴 → "가상 스트림덱" → 실행
- "Windows의 PC 보호" 경고 시: **"추가 정보"** → **"실행"** 클릭 (코드사이닝 없는 오픈소스 SW의 표준)
- Windows Defender 방화벽 팝업 → **"개인 네트워크"** 체크 → "액세스 허용"

## B. 폰에서 사용

1. PC IP 주소 확인
   - **macOS**: 시스템 설정 → 네트워크 → Wi-Fi → 세부 사항 → "IP 주소"
   - **Windows**: 시작 → "cmd" → `ipconfig` → "IPv4 주소" 값 (예: `192.168.0.7`)
2. **폰 브라우저** (Safari/Chrome) 주소창 → `http://192.168.0.7:8090` (위 IP 사용)
3. "PC 페어링" 화면 → 같은 IP + 포트 `41234` 입력 → **페어링 요청**
4. PC 화면 상단 배너 → **승인**
5. 폰 화면이 그리드로 자동 전환 → 끝
6. **(중요) 홈 화면에 추가**:
   - Safari: 공유 버튼 → "홈 화면에 추가"
   - Chrome: ⋮ → "앱 설치" 또는 "홈 화면에 추가"
   - 그 화면에서 한 번 더 페어링·승인 필요 (iOS PWA 컨텍스트 분리), 이후 영구 자동 재연결

## C. 버튼 설정

PC 화면에서:
1. 빈 셀 클릭 → 우측에 인스펙터 열림
2. **`+ 앱`** → 설치된 앱 목록에서 선택 (아이콘 자동)
3. **`+ URL`** → 주소 입력 (예: `https://www.youtube.com/playlist?list=...` → 자동 재생됨)
4. 한 셀에 최대 10개까지 추가 → 폰에서 한 번 탭하면 순서대로 다 실행

---

# 🛠️ 트랙 2 — 소스에서 빌드 + 확장 (개발자/AX 학습자)

## A. 도구 설치 — macOS

```bash
# Homebrew (없으면 먼저)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

brew install rust node git
brew install --cask flutter
```

확인:
```bash
rustc --version && node --version && flutter --version
```

## B. 도구 설치 — Windows

> winget이 동작하면 가장 간편하지만, **회사·학교 정책이나 Win10 N/KN 에디션** 에서는 안 깔려 있을 수 있습니다. 우선 PowerShell에서 `winget --version` 으로 확인. 안 나오면 **방법 2** 의 직접 다운로드 링크 사용.

### 방법 1 — winget이 동작하는 경우 (편함)

PowerShell (일반 권한):
```powershell
winget install -e --id Git.Git
winget install -e --id Rustlang.Rustup
winget install -e --id Microsoft.VisualStudio.2022.BuildTools
winget install -e --id OpenJS.NodeJS.LTS
```

설치 후 **시작 메뉴 → Visual Studio Installer → 위 항목 'Modify'** → **`Desktop development with C++` 워크로드 체크** → Install. (Rust가 Windows에서 컴파일하려면 이 C++ 빌드 도구가 반드시 필요.)

Flutter는 winget 공식 패키지가 없어서 다음 단계의 수동 설치를 따라야 함.

### 방법 2 — 직접 다운로드 (winget 없어도 OK, 비개발자도 가능)

각 링크 클릭 → 안내에 따라 설치:

| 도구 | 다운로드 링크 | 설치 메모 |
|---|---|---|
| **Git** | https://git-scm.com/download/win | 기본 옵션으로 Next 계속 |
| **Rust** | https://win.rustup.rs/x86_64 (rustup-init.exe) | 더블클릭 → `1` 입력 (default) → MSVC 안내가 뜨면 그에 따라 VS Build Tools 설치 |
| **Visual Studio 2022 Build Tools** | https://aka.ms/vs/17/release/vs_BuildTools.exe | 설치 화면에서 **`Desktop development with C++`** 워크로드 체크 → Install |
| **Node.js LTS** | https://nodejs.org/ko/download | "Windows Installer (.msi)" 다운로드 → 기본 옵션으로 설치 |
| **Flutter SDK** | https://docs.flutter.dev/get-started/install/windows | "Download Flutter SDK" zip 다운 → `C:\flutter` 에 압축 풀기 (경로에 **공백/한글 X**) |
| **Flutter PATH 설정** | (수동) | 시작 → "환경 변수" 검색 → "사용자 변수" → `Path` 선택 → 편집 → 새로 만들기 → `C:\flutter\bin` 추가 → 확인 |
| **WebView2 Runtime** *(트랙 1 인스톨러 사용 시 자동, 트랙 2 소스 빌드 시에만 필요)* | https://developer.microsoft.com/en-us/microsoft-edge/webview2/ | "Evergreen Standalone Installer" 다운로드 → 실행 (Win11/최신 Win10는 보통 이미 있음) |

**PowerShell 새 창 열고** 확인 (셸 안 닫고 그대로 진행하면 PATH 갱신 안 됨):
```powershell
git --version
rustc --version
cargo --version
node --version
flutter --version
flutter doctor       # Windows toolchain ✓, Chrome ✓ 면 OK. Android/iOS는 ! 무시
```

## C. 코드 받고 빌드

```bash
git clone https://github.com/im-dullin/Stream-Deck-Mobile.git
cd Stream-Deck-Mobile

# 1) 모바일 웹 빌드 (에이전트에 임베드됨)
cd mobile
flutter pub get
flutter build web --release

# 2) 에이전트 의존성 + 실행
cd ../agent          # Windows는 cd ..\agent
npm install
npm run tauri dev    # 첫 빌드 5~10분, 이후 30초
```

### 배포용 인스톨러 만들기

```bash
cd agent
npm run tauri build
# macOS:   src-tauri/target/release/bundle/dmg/가상 스트림덱_0.1.0_aarch64.dmg
# Windows: src-tauri\target\release\bundle\msi\가상 스트림덱_0.1.0_x64_ko-KR.msi
```

이 인스톨러를 다른 학생에게 주면 트랙 1처럼 사용 가능.

---

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
- 모든 액션 데이터는 PC `pairings.json` / `profile.json` 에 저장.

## 보안 주의

- 같은 와이파이의 누구나 페어링을 시도할 수 있음 (대신 PC에서 매번 명시 승인 요구)
- 에이전트를 **관리자 권한으로 실행하지 말 것**
- 카페·공용 와이파이에서는 사용 비권장 (LAN 전제)

## 라이선스

MIT. 자유롭게 사용·수정·배포 가능. [LICENSE](./LICENSE).

## 기여

PR 환영. 큰 변경은 issue로 먼저 논의해주세요.

---

## AX 교육용 자료

본 레포지토리는 AX(AI Transformation) 교육 실습용으로 설계되었습니다. [`training/`](./training):

- [`01-conversation-log.md`](./training/01-conversation-log.md) — AI와 함께 이 프로젝트를 만든 실제 워크플로우 회고 + AX 협업 9원칙
- [`02-project-spec.md`](./training/02-project-spec.md) — 학생이 본인의 AI 에이전트에 입력으로 줄 수 있는 공식 spec + 난이도별 확장 과제
- [`03-developer-guide.md`](./training/03-developer-guide.md) — 사전설치 → 실행 → 트러블슈팅 상세
