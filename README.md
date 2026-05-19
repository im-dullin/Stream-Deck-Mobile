# 가상 스트림덱

손에 든 폰을, PC의 컨트롤 패널로.

---

Elgato Stream Deck은 우아한 기기입니다. 하지만 비싸고, 책상을 차지하고, 버튼 수가 고정되어 있습니다. 한편 우리에겐 이미 매일 들고 다니는 고해상도 멀티터치 디스플레이가 있습니다 — 스마트폰.

가상 스트림덱은 그 디스플레이를 PC 위의 데크로 바꿉니다. 같은 와이파이, 1회 페어링, 1회 승인. 그 후로는 폰을 켜는 것만으로 PC와 연결됩니다.

---

## 한 버튼, 여러 동작

자주 쓰는 앱과 URL을 그리드에 매핑합니다. 한 버튼에 최대 10개의 동작을 시퀀셜로 묶을 수 있습니다. "작업 모드" 한 번에 Slack과 Notion이 열리고, 로파이 플레이리스트가 자동으로 재생됩니다.

## 필요한 것

PC 한 대. 폰 한 대. 같은 와이파이.

클라우드 없음. 계정 없음. 구독 없음. 모든 데이터는 두 디바이스 사이에서만 흐릅니다.

---

## 시작

[Releases](https://github.com/im-dullin/Stream-Deck-Mobile/releases)에서 OS에 맞는 인스톨러를 받습니다.

&nbsp;&nbsp;&nbsp;&nbsp;macOS&nbsp;&nbsp;·&nbsp;&nbsp;`.dmg`
&nbsp;&nbsp;&nbsp;&nbsp;Windows&nbsp;&nbsp;·&nbsp;&nbsp;`.msi`

설치 후 실행하면 작은 창이 열립니다. 거기 적힌 주소를 폰 브라우저에 입력하고, PC에서 "승인" 한 번. 그게 전부입니다.

폰의 "홈 화면에 추가" 를 누르면 진짜 앱처럼 보입니다.

상세 흐름은 [사용자 가이드](./training/03-developer-guide.md)에.

---

## 구조

```
       ┌──────────────────────┐        ┌──────────────────────────┐
       │   모바일              │        │   PC 에이전트             │
       │   Flutter Web        │        │   Tauri 2 (Rust + React) │
       │                      │        │                          │
       │   그리드·페어링       │◄──WS──►│   토큰 인증·액션 실행      │
       │                      │        │                          │
       │                      │◄─HTTP──│   모바일 웹 번들 호스팅   │
       └──────────────────────┘        └──────────────────────────┘
```

PC 에이전트가 모바일 웹 번들을 직접 호스팅합니다. 폰에는 아무것도 깔리지 않습니다. 단지 URL을 엽니다.

&nbsp;&nbsp;프로토콜&nbsp;&nbsp;·&nbsp;&nbsp;JSON over WebSocket
&nbsp;&nbsp;인증&nbsp;&nbsp;·&nbsp;&nbsp;디바이스별 UUID 토큰 · PC 1회 승인
&nbsp;&nbsp;범위&nbsp;&nbsp;·&nbsp;&nbsp;LAN 한정

---

## 디자인 노트

폰에 앱을 깔지 않는 것이 의도입니다. 앱스토어 심사도, 사이드로드도 없습니다. 같은 URL이 모든 OS에서 같이 동작합니다.

한 번 발급된 페어링 토큰은 영구합니다. 매번 묻지 않습니다.

한 버튼은 곧 하나의 작은 자동화입니다. 같은 패턴이 매크로·크롤링·카드뉴스 자동화로 그대로 확장됩니다.

---

## 빌드 · 확장

소스에서 빌드하거나 새 액션 타입을 만들고 싶다면, [개발자 가이드](./training/03-developer-guide.md)에 macOS · Windows · winget 없는 환경 각각의 단계가 있습니다.

---

## 만든 과정

이 저장소는 AI 코딩 에이전트와의 협업으로 처음부터 만들어졌습니다. 그 대화와 의사결정의 기록이 [`training/`](./training)에 그대로 남아있습니다. AX 교육 실습용으로 설계되어, 학생이 같은 흐름을 재현하거나 확장할 수 있습니다.

— [`01-conversation-log.md`](./training/01-conversation-log.md) · 워크플로우 회고와 AX 협업 9원칙
— [`02-project-spec.md`](./training/02-project-spec.md) · 프로젝트 아이템 정의서와 난이도별 챌린지
— [`03-developer-guide.md`](./training/03-developer-guide.md) · 사전 설치와 트러블슈팅

---

[MIT](./LICENSE)
