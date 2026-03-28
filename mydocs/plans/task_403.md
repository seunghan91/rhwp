# Task 403: VSCode 확장으로 rhwp 제공하기 설계

## 수행 목표

VSCode에서 HWP/HWPX 파일을 열어볼 수 있는 확장(Extension)의 아키텍처를 설계한다. 기존 WASM 렌더링 파이프라인을 최대한 재활용하여, rhwp-studio와 동일한 품질의 문서 뷰어를 VSCode 내에서 제공하는 것이 목표이다.

## 배경

- rhwp는 이미 WASM 빌드(`pkg/`)를 통해 웹 환경에서 HWP 렌더링을 지원한다.
- rhwp-studio는 Canvas 2D 기반 뷰어/에디터로, wasm-bridge.ts가 WASM API를 래핑한다.
- VSCode의 Custom Editor API는 Webview를 통해 바이너리 파일의 커스텀 뷰어를 제공할 수 있다.
- WASM 렌더러가 Canvas 2D, SVG, HTML 세 가지 출력을 지원하므로 Webview에서 재활용 가능하다.

## 현황 분석

### 재활용 가능 자산

| 자산 | 위치 | 용도 |
|------|------|------|
| WASM 바이너리 | `pkg/rhwp_bg.wasm` (3.3MB) | HWP 파싱 + 렌더링 엔진 |
| WASM JS 래퍼 | `pkg/rhwp.js` | wasm-bindgen 생성 인터페이스 |
| TypeScript 타입 | `pkg/rhwp.d.ts` | HwpDocument API 타입 정의 |
| WASM Bridge | `rhwp-studio/src/core/wasm-bridge.ts` | 150+ 메서드 래퍼 (참조용) |
| Canvas View | `rhwp-studio/src/view/canvas-view.ts` | 가상 스크롤 + 페이지 렌더링 |
| Page Renderer | `rhwp-studio/src/view/page-renderer.ts` | 단일 페이지 Canvas 렌더링 |

### VSCode Extension API 핵심

| API | 설명 |
|-----|------|
| `CustomReadonlyEditorProvider` | 읽기 전용 커스텀 에디터 (뷰어 단계에 적합) |
| `CustomEditorProvider` | 읽기/쓰기 커스텀 에디터 (편집 단계) |
| `WebviewPanel` | HTML/CSS/JS를 렌더링하는 샌드박스 환경 |
| `workspace.fs` | 파일 읽기/쓰기 API |

### 렌더링 파이프라인 재활용 전략

```
[VSCode에서 .hwp 파일 열기]
    ↓
Extension Host: 파일 바이너리 읽기 (workspace.fs)
    ↓
Webview: WASM 초기화 + HwpDocument 생성
    ↓
Webview: renderPageToCanvas() 또는 renderPageSvg()
    ↓
Canvas/SVG로 문서 표시
```

## 소스 관리 전략

### 디렉토리 구조

```
rhwp/                          # 모노레포 루트
├── src/                       # Rust 코어 (파서 + 렌더러)
├── pkg/                       # WASM 빌드 출력
├── rhwp-studio/               # 웹 뷰어/에디터 (기존)
├── rhwp-vscode/               # VSCode 확장 (신규, 독립 패키지)
│   ├── package.json           # 확장 매니페스트 + 의존성
│   ├── tsconfig.json
│   ├── src/
│   │   ├── extension.ts       # 확장 진입점
│   │   ├── hwp-editor-provider.ts
│   │   └── webview/           # Webview 전용 코드
│   ├── media/                 # 아이콘, 스타일시트
│   └── webpack.config.js      # 번들러 설정
└── ...
```

### 분리 원칙

- **`rhwp-vscode/`는 `rhwp-studio/`와 완전히 독립된 패키지**로 관리한다.
- 두 패키지 간 소스 코드 공유(import)는 하지 않는다.
- 공통 의존성은 `pkg/` (WASM 빌드 출력)뿐이며, 이를 각자의 방식으로 참조한다.
  - `rhwp-studio`: Vite를 통해 `pkg/` 참조
  - `rhwp-vscode`: webpack 번들 시 `pkg/`의 WASM 파일을 확장에 포함
- Webview 내부 코드(가상 스크롤, 페이지 렌더링 등)는 rhwp-studio를 **참조**하되, 복사 후 VSCode Webview 환경에 맞게 독립 구현한다.

## 설계 범위

### 포함 (v1 — 읽기 전용 뷰어)

- HWP/HWPX 파일 더블클릭 시 문서 뷰어로 열기
- 페이지 단위 렌더링 (Canvas 2D)
- 가상 스크롤 (대용량 문서 대응)
- 줌 인/아웃
- 페이지 네비게이션

### 향후 확장 (v2+)

- 텍스트 편집 (CustomEditorProvider로 전환)
- 텍스트 검색
- 아웃라인 뷰 (목차/제목 네비게이션)
- SVG/PDF 내보내기 명령

## 승인 요청

위 수행계획서를 검토 후 승인 부탁드립니다. 승인 후 구현 계획서를 작성하겠습니다.
