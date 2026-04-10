# Task #90 — 최종 완료보고서

## Rust aarch64-apple-ios 크로스 컴파일 파이프라인

### 목표

Rust rhwp 엔진을 iOS 타겟으로 크로스 컴파일하여 iPad에서 HWP 문서를 네이티브 렌더링한다.

### 단계별 결과

| 단계 | 내용 | 결과 |
|------|------|------|
| 1 | iOS 타겟 크로스 컴파일 | ✅ `librhwp.a` 47MB (실기기 + Simulator)2 |
| 2 | C-ABI FFI 인터페이스 | ✅ 5개 함수 + cbindgen 헤더 |
| 3 | Xcode 프로젝트 + iPad Simulator | ✅ HWP 로드 + SVG 렌더링 성공 |

### 생성된 산출물

| 파일 | 역할 |
|------|------|
| `Cargo.toml` | `staticlib` 추가 |
| `src/ios_ffi.rs` | C-ABI FFI 함수 5개 |
| `src/lib.rs` | `ios_ffi` 모듈 등록 |
| `cbindgen.toml` | C 헤더 자동 생성 설정 |
| `rhwp-ios/project.yml` | xcodegen 프로젝트 스펙 |
| `rhwp-ios/Sources/` | SwiftUI 앱 (ContentView + FFI 호출) |
| `rhwp-ios/Resources/` | 테스트용 샘플 HWP 파일 |

### FFI API

```c
RhwpHandle *rhwp_open(const uint8_t *data, size_t len);
uint32_t rhwp_page_count(const RhwpHandle *handle);
char *rhwp_render_page_svg(const RhwpHandle *handle, uint32_t page);
void rhwp_free_string(char *ptr);
void rhwp_close(RhwpHandle *handle);
```

### 검증 결과

- iPad Simulator (iPad Pro 11-inch M4): HWP 파일 로드 + SVG 렌더링 ✅
- 페이지 수 표시 ✅
- `cargo test`: 785 passed, 0 failed (기존 회귀 없음)

### 해결한 문제

| 문제 | 해결 |
|------|------|
| cbindgen 전체 헤더 컴파일 오류 | 최소 FFI 헤더 직접 작성 |
| 스킴명 한글 빌드 오류 | 타겟명 영문, PRODUCT_NAME만 한글 |
| staticlib 미갱신 심볼 누락 | cargo build 재실행 |
| 리소스 번들 미포함 | 앱 번들 수동 복사 (빌드 자동화 보류) |

### 라이브러리 크기 분석 (47MB)

정적 라이브러리(`.a`)는 모든 의존 크레이트의 오브젝트 파일을 포함하므로 크기가 크다. 크레이트별 상위 기여:

| 크레이트 | 크기 | 비고 |
|----------|------|------|
| std + core + compiler_builtins | 13.7MB (29%) | Rust 런타임 (정적 링크 필수) |
| write_fonts + read_fonts + skrifa | 8.2MB (17%) | 폰트 서브셋/파싱 |
| rhwp | 6.5MB (14%) | 프로젝트 코드 |
| image | 1.9MB (4%) | 이미지 디코딩 |
| pxfm + usvg | 2.7MB (6%) | SVG 렌더링 |
| 기타 (922개 오브젝트) | 14MB (30%) | 나머지 의존성 |

최종 앱 바이너리에서는 링커의 dead code elimination이 적용되어 실제 포함 크기는 상당히 줄어든다. 추가 최적화 방안:

- 폰트 서브셋 기능(`subsetter`, `write_fonts`)을 feature flag로 분리하여 iOS 빌드 시 제외
- `codegen-units = 1` + LTO 적용으로 추가 최적화 가능

### 후속 작업 (M1 남은 이슈)

- #91: Swift UI + Rust FFI 브릿지 고도화
- #92: iOS 최소 뷰어 앱 (파일 선택 + 다중 페이지)
