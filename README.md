<div align="center">

<img src="app-icon.png" width="128" height="128" alt="TextMacro" />

# TextMacro

**전역 단축키 한 번으로, 자주 쓰는 문구를 어디서나 즉시 붙여넣기**

자주 입력하는 인사말·서명·답변 템플릿을 매크로로 저장해 두고,
어느 앱에서든 단축키로 피커를 열어 골라 넣을 수 있는 가볍고 빠른 데스크톱 유틸리티입니다.

[![Release](https://img.shields.io/github/v/release/rainsound-k/text-macro?style=flat-square)](../../releases)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue?style=flat-square)

</div>

---

## ✨ 주요 기능

- **전역 단축키** — 어느 앱에서든 `Alt + Space`(기본값)로 매크로 피커 호출
- **빠른 선택** — `⌘1 ~ ⌘9`(macOS) / `Ctrl+1 ~ Ctrl+9`(Windows), 또는 방향키 + Enter, 마우스 클릭
- **자동 붙여넣기** — 매크로를 고르면 직전에 작업하던 앱으로 포커스가 돌아가 그 자리에 바로 입력
- **실시간 검색** — 피커에서 타이핑하면 제목·내용으로 즉시 필터링
- **본문 미리보기** — 목록에서 내용 최대 3줄까지 미리 표시
- **멀티 모니터 지원** — 단축키를 누른 화면(커서가 있는 모니터)에 피커가 표시
- **매크로 관리** — 추가 / 편집 / 삭제 / 순서 변경
- **가져오기 · 내보내기** — JSON 파일로 매크로 백업 및 이전
- **로그인 시 자동 실행** — 컴퓨터를 켜면 자동으로 백그라운드 상주
- **메뉴 막대 상주** — Dock 아이콘 없이 시스템 트레이/메뉴 막대에서 조용히 동작

---

## 📦 설치

[**Releases**](../../releases) 페이지에서 운영체제에 맞는 파일을 받으세요.

### macOS
| 파일 | 대상 |
|------|------|
| `TextMacro_macos-arm64.zip` | Apple Silicon (M1/M2/M3) |
| `TextMacro_macos-x64.zip` | Intel Mac |

1. ZIP을 내려받아 압축을 풉니다. (압축 해제 시에는 경고가 뜨지 않습니다.)
2. **TextMacro.app**을 `응용 프로그램` 폴더로 옮깁니다.
3. 서명되지 않은 빌드이므로 처음 실행 시 경고가 뜨면 앱을 **우클릭 → 열기**로 실행합니다.

> **왜 DMG가 아니라 ZIP인가요?** macOS Sequoia(15)부터는 공증되지 않은 DMG를 더블클릭하면
> Gatekeeper 경고를 우회하기 어렵습니다(우클릭 → 열기가 막힘). ZIP은 압축 해제 후 앱 실행
> 경고만 한 번 처리하면 되어 더 간단합니다. 경고를 완전히 없애려면 Apple 공증이 필요합니다.

> 자신의 칩을 모를 때: `Apple 메뉴 → 이 Mac에 관하여`에서 "Apple M…" 이면 Apple Silicon, "Intel" 이면 x64.

> 그래도 경고를 건너뛰고 싶다면 (파워 유저): `xattr -dr com.apple.quarantine /Applications/TextMacro.app`

### Windows
| 파일 | 형식 |
|------|------|
| `TextMacro_*_x64-setup.exe` | 설치 마법사 (권장) |
| `TextMacro_*_x64_en-US.msi` | MSI 인스톨러 |

처음 실행 시 SmartScreen 경고가 뜨면 **추가 정보 → 실행**을 선택하세요.

---

## 🔐 macOS 권한 설정 (필수)

자동 붙여넣기는 키 입력을 시뮬레이션하므로 **손쉬운 사용(Accessibility)** 권한이 필요합니다.
권한이 없으면 복사는 되지만 입력창에 붙여넣어지지 않습니다.

1. `시스템 설정` → `개인 정보 보호 및 보안` → `손쉬운 사용`
2. 목록에서 **TextMacro**를 켜기
   (앱 실행 후 설정 창 상단의 노란 배너에서 **"설정 열기"** 버튼으로 바로 이동 가능)

> 앱을 새 버전으로 교체한 뒤 입력이 안 되면, 손쉬운 사용 목록에서 TextMacro를 **껐다가 다시 켜** 주세요. macOS가 변경된 실행 파일의 권한을 다시 확인합니다.

---

## ⌨️ 사용법

1. 어디서든 `Alt + Space`를 눌러 피커를 엽니다.
2. 매크로를 선택합니다.
   - 위에서부터 9개는 `⌘1 ~ ⌘9` (Windows는 `Ctrl+1 ~ Ctrl+9`)
   - 또는 `↑` `↓`로 이동 후 `Enter`, 혹은 마우스 클릭
   - 검색창에 입력하면 제목·내용으로 필터링
3. 선택하는 순간 직전에 작업하던 앱의 커서 위치에 텍스트가 붙여넣어집니다.
4. `Esc`로 피커를 닫습니다.

매크로 추가·편집과 단축키·자동 실행 변경은 **메뉴 막대 아이콘 → 설정 열기**에서 할 수 있습니다.

---

## 🛠️ 개발

### 요구 사항
- [Node.js](https://nodejs.org/) **20.19+ 또는 22.12+** (Vite 7 요구사항)
- [Rust](https://rustup.rs/) (stable)
- macOS: Xcode Command Line Tools / Windows: Visual Studio C++ Build Tools

### 시작하기
```bash
# 의존성 설치
npm install

# 개발 모드 실행 (핫 리로드)
npm run tauri dev
```

### 프로덕션 빌드
```bash
# 현재 플랫폼용 설치 파일 생성 (.dmg / .exe / .msi)
npm run tauri build
```
빌드 결과물은 `src-tauri/target/release/bundle/` 에 생성됩니다.

### 아이콘 재생성
`app-icon.png`(1024×1024)를 수정한 뒤 아래 명령으로 전 플랫폼 아이콘을 다시 생성합니다.
```bash
npm run tauri icon app-icon.png
```

---

## 🚀 릴리스

`v*` 형식의 git 태그를 푸시하면 GitHub Actions가 macOS(arm64·x64)와 Windows(x64) 설치 파일을
빌드해 **Draft 릴리스**로 업로드합니다.

```bash
git tag v1.0.0
git push origin v1.0.0
```
이후 GitHub의 Releases에서 초안을 확인하고 게시하세요.

---

## 🔏 macOS 코드 서명 (권한 유지, 선택)

서명되지 않은 빌드는 매 업데이트마다 실행 파일 해시가 바뀌어, **새 버전을 설치할 때마다
손쉬운 사용 권한을 다시 요청**합니다. 같은 **self-signed 인증서**로 매번 서명하면 앱의
코드 식별자가 고정되어 권한이 업데이트 후에도 유지됩니다. (무료, Apple 개발자 계정 불필요.
단, Gatekeeper "미확인 개발자" 경고는 그대로이며 첫 실행만 우클릭 → 열기 필요.)

**설정 방법 (최초 1회):**

1. 인증서를 생성하고 시크릿 값을 출력합니다.
   ```bash
   bash scripts/create-macos-cert.sh
   ```
2. 출력된 4개 값을 GitHub 저장소 시크릿으로 등록합니다.
   `Settings → Secrets and variables → Actions → New repository secret`
   - `APPLE_SIGNING_IDENTITY`
   - `APPLE_CERTIFICATE` (base64 블록 전체)
   - `APPLE_CERTIFICATE_PASSWORD`
   - `KEYCHAIN_PASSWORD`
3. 다음 릴리스부터 자동으로 서명되어 빌드됩니다.

> 시크릿이 등록되지 않은 상태에서도 빌드는 정상 동작합니다(서명 없이 ad-hoc 빌드).
> 릴리스 워크플로우(`.github/workflows/release.yml`)가 시크릿 유무를 자동 판별합니다.

---

## 🧱 기술 스택

- **[Tauri 2](https://tauri.app/)** — 경량 Rust 기반 데스크톱 셸
- **React 19 + TypeScript + Vite** — 프런트엔드
- **Rust** — 전역 단축키, 클립보드, 키 입력 시뮬레이션, 트레이/창 관리
  - `tauri-plugin-global-shortcut`, `tauri-plugin-autostart`, `tauri-plugin-dialog`
  - `arboard`(클립보드), macOS는 `CoreGraphics` 이벤트로 직접 붙여넣기

---

## 📄 라이선스

MIT
