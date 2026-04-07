# 배포 가이드

rhwp 프로젝트의 배포 대상과 절차를 정리한다.

---

## 배포 대상

| 대상 | 패키지명 | 배포 방식 | 트리거 |
|------|---------|----------|--------|
| GitHub Pages (데모) | — | CI/CD 자동 | main push 또는 태그 |
| npm WASM 코어 | @rhwp/core | CI/CD 자동 | GitHub Release 생성 |
| VSCode 익스텐션 | rhwp-vscode | 수동 스크립트 | `publish.sh` 실행 |
| npm 에디터 | @rhwp/editor | 수동 | `npm publish` 실행 |

---

## CI/CD 워크플로우 (GitHub Actions)

### 자동 실행되는 워크플로우

| 파일 | 트리거 | 역할 |
|------|--------|------|
| `.github/workflows/ci.yml` | push/PR (main, devel) | cargo build + test + clippy 검증 |
| `.github/workflows/deploy-pages.yml` | main push, 태그 | WASM 빌드 → rhwp-studio 빌드 → GitHub Pages 배포 |
| `.github/workflows/npm-publish.yml` | **GitHub Release 생성** | WASM 빌드 → @rhwp/core npm 배포 |

### CI/CD 자동 배포 흐름

```
코드 작업 완료
  ↓
devel push → CI 자동 실행 (build + test + clippy)
  ↓
main merge + push → GitHub Pages 자동 배포
  ↓
GitHub Release 생성 (태그) → npm @rhwp/core 자동 배포
  ↓
수동: VSCode 익스텐션 배포 (publish.sh)
수동: npm @rhwp/editor 배포 (npm publish)
```

> **중요**: npm @rhwp/core는 수동으로 `npm publish`하지 않는다. GitHub Release를 생성하면 자동 배포된다.

### GitHub Secrets 설정

GitHub Actions에서 사용하는 시크릿 (Settings → Secrets and variables → Actions):

| Secret 이름 | 용도 |
|------------|------|
| `NPM_TOKEN` | npm 배포 인증 (@rhwp/core 자동 배포) |

---

## 버전 관리

### 버전 번호 규칙 (Semantic Versioning)

```
v{MAJOR}.{MINOR}.{PATCH}
  │       │       └─ 버그 수정, README 보강, 문서 업데이트
  │       └───────── 기능 추가, 조판 개선, API 추가
  └─────────────────  호환성이 깨지는 변경 (v1.0.0 = 편집 엔진 정합성 확립)
```

### 버전 번호가 관리되는 파일

| 파일 | 패키지 | 예시 |
|------|--------|------|
| `Cargo.toml` | rhwp (Rust) + @rhwp/core 원본 | `version = "0.6.0"` |
| `rhwp-vscode/package.json` | VSCode 익스텐션 | `"version": "0.6.0"` |
| `npm/editor/package.json` | @rhwp/editor | `"version": "0.6.3"` |

> `pkg/package.json`은 직접 편집하지 않는다. `scripts/prepare-npm.sh`가 `Cargo.toml`에서 버전을 읽어 자동 생성한다.

### 버전 동기화 원칙

- **Cargo.toml이 기준**이다. MINOR 버전은 모든 패키지가 동일하게 맞춘다.
- @rhwp/core 는 Cargo.toml 버전을 그대로 따른다.
- VSCode 익스텐션은 Cargo.toml과 MINOR까지 동일하게 유지한다.
- @rhwp/editor 는 독자적으로 PATCH를 올릴 수 있다 (README 보강 등).
- npm은 한 번 배포한 버전을 덮어쓸 수 없으므로, README만 수정해도 PATCH를 올려야 한다.

### 버전 올리기 예시

**MINOR 릴리즈** (조판 개선, 새 기능):
```
Cargo.toml:               0.6.0 → 0.7.0
rhwp-vscode/package.json: 0.6.0 → 0.7.0
npm/editor/package.json:  0.6.3 → 0.7.0
```

**PATCH 릴리즈** (npm README 수정 등):
```
npm/editor/package.json:  0.6.1 → 0.6.2  (다른 파일 변경 없음)
```

### Git 태그

- 태그는 `v{MAJOR}.{MINOR}.{PATCH}` 형식 (예: `v0.6.0`)
- Cargo.toml 기준 MINOR 릴리즈마다 태그를 생성한다
- PATCH 전용 릴리즈(npm README 등)는 태그를 생성하지 않는다

---

## 배포 절차

### 1단계: 코드 검증

```bash
cargo build && cargo test        # 네이티브 빌드 + 783개 테스트
docker compose --env-file .env.docker run --rm wasm   # WASM 빌드
```

E2E 테스트:
```bash
cd rhwp-studio
CHROME_CDP=http://localhost:19222 node e2e/edit-pipeline.test.mjs --mode=host
# 16개 테스트 파일 순차 실행
```

### 2단계: 버전 업데이트 + CHANGELOG

**Cargo.toml** (Rust 패키지 + npm @rhwp/core 버전 원본):
```toml
version = "0.7.0"
```

**rhwp-vscode/package.json**:
```json
"version": "0.7.0"
```

**rhwp-vscode/CHANGELOG.md** 새 버전 항목 추가.

**npm/editor/package.json**:
```json
"version": "0.7.0"
```

### 3단계: README 점검

모든 배포 대상의 README에 다음 항목이 포함되어야 한다:

| 항목 | rhwp-vscode | npm/core | npm/editor |
|------|:---------:|:-------:|:---------:|
| 기능 목록 | O | O | O |
| 폰트 가이드 | — | O (CDN/셀프호스팅) | O (내장 폴백 안내) |
| Third-Party Licenses | O | O | O |
| Trademark 면책 조항 | O | O | O |
| Notice (한컴 공개 문서) | O | O | O |

### 4단계: Git 커밋 + devel/main push

```bash
# 변경사항 커밋
git add -A
git commit -m "v0.7.0 릴리즈 준비"

# devel → main merge
git checkout devel && git merge local/devel && git push origin devel
git checkout main && git merge devel && git push origin main
```

> main push 시 CI/CD가 자동 실행된다:
> - `ci.yml` → build + test + clippy 검증
> - `deploy-pages.yml` → GitHub Pages 데모 사이트 자동 배포

### 5단계: GitHub Release 생성 → npm @rhwp/core 자동 배포

```bash
git tag v0.7.0
git push origin v0.7.0
gh release create v0.7.0 --title "v0.7.0 — 제목" --notes "릴리즈 노트"
```

> **Release 생성 시 `npm-publish.yml` 자동 실행:**
> 1. WASM 빌드
> 2. `scripts/prepare-npm.sh` 실행
> 3. `npm publish --access public --provenance`
>
> 수동으로 `cd pkg && npm publish`를 실행하지 않는다.

### 6단계: VSCode 익스텐션 수동 배포

```bash
cd rhwp-vscode
bash publish.sh
```

`publish.sh` 수행 작업:
1. `assets/logo/logo-128.png` → `media/icon.png` 복사
2. `pkg/rhwp_bg.wasm`, `pkg/rhwp.js` → `media/` 복사
3. webpack 빌드
4. VS Code Marketplace 배포 (`npx vsce publish`)
5. Open VSX 배포 (`npx ovsx publish`)

사전 조건 — `.env` 파일에 토큰 설정:
```
VSCE_PAT=<Azure DevOps Personal Access Token>
OVSX_PAT=<Open VSX Access Token>
```

### 7단계: npm @rhwp/editor 수동 배포

```bash
cd npm/editor
npm publish --access public
```

사전 조건 — `~/.npmrc`에 npm 토큰 설정:
```bash
echo "//registry.npmjs.org/:_authToken=<npm_token>" > ~/.npmrc
```

### 8단계: 배포 확인

| 대상 | 확인 URL |
|------|---------|
| GitHub Pages | https://edwardkim.github.io/rhwp/ |
| VS Code Marketplace | https://marketplace.visualstudio.com/items?itemName=edwardkim.rhwp-vscode |
| Open VSX | https://open-vsx.org/extension/edwardkim/rhwp-vscode |
| npm @rhwp/core | https://www.npmjs.com/package/@rhwp/core |
| npm @rhwp/editor | https://www.npmjs.com/package/@rhwp/editor |

---

## 토큰 관리

### 로컬 배포용 (`.env`)

| 토큰 | 발급처 | 용도 |
|------|--------|------|
| VSCE_PAT | [Azure DevOps](https://dev.azure.com) → Personal Access Tokens | VSCode 익스텐션 배포 |
| OVSX_PAT | [open-vsx.org](https://open-vsx.org) → Access Tokens | Open VSX 배포 |
| npm_token | [npmjs.com](https://www.npmjs.com) → Access Tokens | @rhwp/editor 수동 배포 |

### CI/CD 자동 배포용 (GitHub Secrets)

| Secret | 용도 |
|--------|------|
| NPM_TOKEN | @rhwp/core 자동 배포 (npm-publish.yml) |

> GitHub Secrets 설정: Settings → Secrets and variables → Actions → New repository secret

---

## 배포 체크리스트

### 배포 전

- [ ] `cargo build` + `cargo test` 통과
- [ ] WASM 빌드 완료 (`pkg/`)
- [ ] E2E 테스트 통과
- [ ] 저작권 폰트가 포함되지 않았는지 확인
- [ ] Cargo.toml, package.json 버전 업데이트
- [ ] CHANGELOG.md 작성
- [ ] README 현행화 (기능, 폰트 가이드, 라이선스, 상표)

### 배포 순서

- [ ] devel push → CI 통과 확인
- [ ] main merge + push → GitHub Pages 배포 확인
- [ ] GitHub Release 생성 → npm @rhwp/core 자동 배포 확인
- [ ] `publish.sh` → VSCode 익스텐션 배포 확인
- [ ] `npm publish` → @rhwp/editor 배포 확인
- [ ] 5곳 URL 모두 최신 버전 확인

---

## 트러블슈팅

### VSCE_PAT 오류

```
❌ VSCE_PAT가 .env에 설정되지 않았습니다
```

- `.env` 파일에서 `VSCE_PAT=` 줄 앞에 개행이 있는지 확인
- Windows 줄바꿈(`\r`)이 포함되었을 수 있음: `cat -A .env`로 확인

### npm publish 버전 충돌

```
You cannot publish over the previously published versions
```

- 이미 배포된 버전. package.json 버전을 올려야 함 (예: 0.6.0 → 0.6.1)
- npm은 한 번 배포된 버전을 덮어쓸 수 없음
- CI/CD 자동 배포와 수동 배포가 충돌한 경우 패치 버전을 올려서 수동 배포

### pkg/ 권한 오류

```
Permission denied: pkg/package.json
```

- Docker 빌드로 `pkg/`가 root 소유로 생성된 경우
- `sudo chown -R $(whoami) pkg/` 로 소유권 변경 후 재시도

### GitHub Actions npm 배포 실패

- GitHub Secrets에 `NPM_TOKEN`이 설정되어 있는지 확인
- 토큰 만료 여부 확인 (npmjs.com에서 재발급)
- Actions 탭에서 `npm-publish.yml` 실행 로그 확인

### Open VSX 배포 실패

- OVSX_PAT 토큰 만료 확인 (open-vsx.org에서 재발급)
- `npx ovsx publish` 수동 실행으로 에러 메시지 확인
