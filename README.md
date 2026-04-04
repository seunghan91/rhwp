<p align="center">
  <img src="assets/logo/logo-256.png" alt="rhwp logo" width="128" />
</p>

<h1 align="center">rhwp</h1>

<p align="center">
  <strong>알(R), 모두의 한글</strong> — 알에서 시작하다<br/>
  <em>All HWP, Open for Everyone</em>
</p>

<p align="center">
  <a href="https://github.com/edwardkim/rhwp/actions/workflows/ci.yml"><img src="https://github.com/edwardkim/rhwp/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://edwardkim.github.io/rhwp/"><img src="https://img.shields.io/badge/Demo-GitHub%20Pages-blue" alt="Demo" /></a>
  <a href="https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode"><img src="https://img.shields.io/badge/VS%20Code-Marketplace-007ACC" alt="VS Code" /></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT" /></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75%2B-orange.svg" alt="Rust" /></a>
  <a href="https://webassembly.org/"><img src="https://img.shields.io/badge/WebAssembly-Ready-blue.svg" alt="WASM" /></a>
</p>

---

HWP 파일을 **어디서든** 열어보세요. 무료, 설치 없이.

rhwp는 Rust + WebAssembly 기반의 오픈소스 HWP/HWPX 뷰어/에디터입니다. 닫힌 포맷의 벽을 깨고, 모든 사람, 모든 AI, 모든 플랫폼에서 한글 문서를 자유롭게 읽고 쓸 수 있게 합니다.

> **[온라인 데모](https://edwardkim.github.io/rhwp/)** | **[VS Code 확장](https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode)** | **[Open VSX](https://open-vsx.org/extension/edwardkim/rhwp-vscode)**

<p align="center">
  <img src="assets/screenshots/render-example-1.png" alt="rhwp 렌더링 예시 — KTX 노선도" width="700" />
</p>

## 로드맵

혼자 뼈대를 세우고, 함께 살을 붙이고, 모두의 것으로 완성한다.

```
0.5 ──── 1.0 ──── 2.0 ──── 3.0
뼈대      조판      협업      완성
```

| 단계 | 방향 | 전략 |
|------|------|------|
| **0.5 → 1.0** | 읽기/쓰기 기반 위에 조판 엔진 체계화 | 핵심 아키텍처를 혼자 견고하게 |
| **1.0 → 2.0** | AI 조판 파이프라인 위에 커뮤니티 참여 개방 | 기여 진입 장벽을 낮추는 구조 |
| **2.0 → 3.0** | 커뮤니티가 채운 기능 위에 공공 자산화 | 한컴 대등 수준 달성 |

> 0.5.0까지 혼자 뼈대를 완성하고 공개하는 이유 — 커뮤니티가 붙었을 때 방향이 흔들리지 않으려면 핵심 아키텍처가 먼저 견고해야 합니다.

## 이정표

### v0.5.0 — 뼈대 (현재)

> 역공학 완성, 읽기/쓰기 기반 구축

- HWP 5.0 / HWPX 파서, 문단·표·수식·이미지·차트 렌더링
- 페이지네이션 (다단 분할, 표 행 분할), 머리말/꼬리말/바탕쪽/각주
- SVG 내보내기 (CLI) + Canvas 렌더링 (WASM/Web)
- 웹 에디터 + hwpctl 호환 API (30 Actions, Field API)
- 755+ 테스트

### v1.0.0 — 조판 엔진

> AI 조판 파이프라인, 뼈대 완성

- 편집 시 동적 재조판 체계화 (LINE_SEG 재계산 + 페이지네이션 연동)
- AI 기반 문서 생성/편집 파이프라인
- 문서 조판 품질 한컴 뷰어 수준 도달

### v2.0.0 — 협업

> 커뮤니티가 기능을 채워가는 단계, 살 붙이기

- 플러그인/확장 아키텍처, 실시간 협업 편집
- 다양한 출력 포맷 (PDF, DOCX 등)

### v3.0.0 — 완성

> 한컴과 대등한 수준, 완전한 공공 자산

- 전체 HWP 기능 커버리지, 접근성(a11y), 모바일 대응
- 공공기관 실무 투입 가능 수준

자세한 내용은 [로드맵 문서](mydocs/report/rhwp-milestone.md)를 참조하세요.

---

## Features

### Parsing (파싱)
- HWP 5.0 binary format (OLE2 Compound File)
- HWPX (Open XML-based format)
- Sections, paragraphs, tables, textboxes, images, equations, charts
- Header/footer, master pages, footnotes/endnotes

### Rendering (렌더링)
- **Paragraph layout**: line spacing, indentation, alignment, tab stops
- **Tables**: cell merging, border styles (solid/double/triple/dotted), cell formula calculation
- **Multi-column layout** (2-column, 3-column, etc.)
- **Paragraph numbering/bullets**
- **Vertical text** (영문 눕힘/세움)
- **Header/footer** (odd/even page separation)
- **Master pages** (Both/Odd/Even, is_extension/overlap)
- **Object placement**: TopAndBottom, treat-as-char (TAC), in-front-of/behind text

### Equation (수식)
- Fractions (OVER), square roots (SQRT/ROOT), subscript/superscript
- Matrices: MATRIX, PMATRIX, BMATRIX, DMATRIX
- Cases, alignment (EQALIGN), stacking (PILE/LPILE/RPILE)
- Large operators: INT, DINT, TINT, OINT, SUM, PROD
- Relations (REL/BUILDREL), limits (lim), long division (LONGDIV)
- 15 text decorations, full Greek alphabet, 100+ math symbols

### Pagination (페이지 분할)
- Multi-column document column/page splitting
- Table row-level page splitting (PartialTable)
- shape_reserved handling for TopAndBottom objects
- vpos-based paragraph position correction

### Output (출력)
- SVG export (CLI)
- Canvas rendering (WASM/Web)
- Debug overlay (paragraph/table boundaries + indices + y-coordinates)

### Web Editor (웹 에디터)
- Text editing (insert, delete, undo/redo)
- Character/paragraph formatting dialogs
- Table creation, row/column insert/delete, cell formula
- hwpctl-compatible API layer (한컴 웹기안기 호환)

### hwpctl Compatibility (한컴 호환 레이어)
- 30 Actions: TableCreate, InsertText, CharShape, ParagraphShape, etc.
- ParameterSet/ParameterArray API
- Field API: GetFieldList, PutFieldText, GetFieldText
- Template data binding support

## Quick Start

처음 프로젝트에 참여하는 개발자는 [온보딩 가이드](mydocs/manual/onboarding_guide.md)를 먼저 읽어보세요. 프로젝트 아키텍처, 디버깅 도구, 개발 워크플로우를 한눈에 파악할 수 있습니다.

### Requirements
- Rust 1.75+
- Docker (for WASM build)
- Node.js 18+ (for web editor)

### Native Build

```bash
cargo build                    # Development build
cargo build --release          # Release build
cargo test                     # Run tests (755+ tests)
```

### WASM Build

WASM 빌드는 Docker를 사용합니다. 플랫폼에 관계없이 동일한 `wasm-pack` + Rust 툴체인 환경을 보장하기 위함입니다.

```bash
cp .env.docker.example .env.docker   # 최초 1회: 환경변수 템플릿 복사
docker compose --env-file .env.docker run --rm wasm
```

빌드 결과물은 `pkg/` 디렉토리에 생성됩니다.

### Web Editor

```bash
cd rhwp-studio
npm install
npx vite --host 0.0.0.0 --port 7700
```

Open `http://localhost:7700` in your browser.

## CLI Usage

### SVG Export

```bash
rhwp export-svg sample.hwp                         # Export to output/
rhwp export-svg sample.hwp -o my_dir/              # Export to custom directory
rhwp export-svg sample.hwp -p 0                    # Export specific page (0-indexed)
rhwp export-svg sample.hwp --debug-overlay         # Debug overlay (paragraph/table boundaries)
```

### Document Inspection

```bash
rhwp dump sample.hwp                  # Full IR dump
rhwp dump sample.hwp -s 2 -p 45      # Section 2, paragraph 45 only
rhwp dump-pages sample.hwp -p 15     # Page 16 layout items
rhwp info sample.hwp                  # File info (size, version, sections, fonts)
```

### Debugging Workflow

1. `export-svg --debug-overlay` → Identify paragraphs/tables by `s{section}:pi={index} y={coord}`
2. `dump-pages -p N` → Check paragraph layout list and heights
3. `dump -s N -p M` → Inspect ParaShape, LINE_SEG, table properties

No code modification needed for the entire debugging process.

## Project Structure

```
src/
├── main.rs                    # CLI entry point
├── parser/                    # HWP/HWPX file parser
├── model/                     # HWP document model
├── document_core/             # Document core (CQRS: commands + queries)
│   ├── commands/              # Edit commands (text, formatting, tables)
│   ├── queries/               # Queries (rendering data, pagination)
│   └── table_calc/            # Table formula engine (SUM, AVG, PRODUCT, etc.)
├── renderer/                  # Rendering engine
│   ├── layout/                # Layout (paragraph, table, shapes, cells)
│   ├── pagination/            # Pagination engine
│   ├── equation/              # Equation parser/layout/renderer
│   ├── svg.rs                 # SVG output
│   └── web_canvas.rs          # Canvas output
├── serializer/                # HWP file serializer (save)
└── wasm_api.rs                # WASM bindings

rhwp-studio/                   # Web editor (TypeScript + Vite)
├── src/
│   ├── core/                  # Core (WASM bridge, types)
│   ├── engine/                # Input handlers
│   ├── hwpctl/                # hwpctl compatibility layer
│   ├── ui/                    # UI (menus, toolbars, dialogs)
│   └── view/                  # Views (ruler, status bar, canvas)
├── e2e/                       # E2E tests (Puppeteer + Chrome CDP)
│   └── helpers.mjs            # Test helpers (headless/host modes)

mydocs/                        # Project documentation (Korean)
├── orders/                    # Daily task tracking
├── plans/                     # Task plans and implementation specs
├── feedback/                  # Code review feedback
├── tech/                      # Technical documents
└── manual/                    # Manuals and guides

scripts/                       # Build & quality tools
├── metrics.sh                 # Code quality metrics collection
└── dashboard.html             # Quality dashboard with trend tracking
```

## Development

이 프로젝트는 **Claude Code** (Anthropic AI 코딩 에이전트)를 페어 프로그래밍 파트너로 사용하여 개발합니다.

### Git 워크플로우

```
local/task{N}  ──커밋──커밋──┐
                              ├─→ devel merge (관련 타스크 묶어서)
                              ├─→ main merge + 태그 (릴리즈 시점)
```

| 브랜치 | 용도 |
|--------|------|
| `main` | 릴리즈 (태그: v0.5.0 등) |
| `devel` | 개발 통합 |
| `local/task{N}` | GitHub Issue 번호 기반 타스크 브랜치 |

### 타스크 관리

- **GitHub Issues**로 타스크 번호 자동 채번 — 중복 방지
- **GitHub Milestones**로 타스크 그룹화
- 마일스톤 표기: `M{버전}` (예: M100=v1.0.0, M05x=v0.5.x)
- 오늘할일: `mydocs/orders/yyyymmdd.md` — `M100 #1` 형식으로 참조
- 커밋 메시지: `Task #1: 내용` — `closes #1`로 Issue 자동 종료

### 타스크 진행 절차

1. `gh issue create` → GitHub Issue 등록 (마일스톤 지정)
2. `local/task{issue번호}` 브랜치 생성
3. 수행계획서 작성 → 승인 → 구현 → 테스트
4. devel merge → `closes #{번호}`

### 디버깅 프로토콜

1. `export-svg --debug-overlay` → 문단/표 식별
2. `dump-pages -p N` → 배치 목록과 높이
3. `dump -s N -p M` → ParaShape, LINE_SEG 상세

> `mydocs/`의 문서는 AI 기반 소프트웨어 개발의 교육 자료로 활용됩니다.

### 문서 생성 규칙

모든 문서는 **한국어**로 작성합니다.

```
mydocs/
├── orders/           # 오늘 할일 (yyyymmdd.md)
├── plans/            # 수행 계획서, 구현 계획서
│   └── archives/     # 완료된 계획서 보관
├── working/          # 단계별 완료 보고서
├── report/           # 기본 보고서
├── feedback/         # 코드 리뷰 피드백
├── tech/             # 기술 사항 정리 문서
├── manual/           # 매뉴얼, 가이드 문서
└── troubleshootings/ # 트러블슈팅 관련 문서
```

| 문서 유형 | 위치 | 파일명 규칙 |
|----------|------|------------|
| 오늘 할일 | `orders/` | `yyyymmdd.md` — 마일스톤(M100)+Issue(#1) 형식 |
| 수행 계획서 | `plans/` | Issue 번호 참조 |
| 완료 보고서 | `working/` | Issue 번호 참조 |
| 기술 문서 | `tech/` | 주제별 자유 명명 |

## Architecture

```mermaid
graph TB
    HWP[HWP/HWPX File] --> Parser
    Parser --> Model[Document Model]
    Model --> DocumentCore
    DocumentCore --> |Commands| Edit[Edit Operations]
    DocumentCore --> |Queries| Render[Rendering Pipeline]
    Render --> Pagination
    Pagination --> Layout
    Layout --> SVG[SVG Output]
    Layout --> Canvas[Canvas Output]
    DocumentCore --> WASM[WASM API]
    WASM --> Studio[rhwp-studio Web Editor]
    Studio --> hwpctl[hwpctl Compatibility Layer]
```

## HWPUNIT

- 1 inch = 7,200 HWPUNIT
- 1 inch = 25.4 mm
- 1 HWPUNIT ≈ 0.00353 mm

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

[MIT License](LICENSE) — Copyright (c) 2025-2026 Edward Kim
