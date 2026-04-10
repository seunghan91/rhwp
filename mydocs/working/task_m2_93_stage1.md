# Task #93 — 1단계 완료보고서

## Rust serde 도입 + 렌더 트리 직렬화 FFI ✅

### 작업 내용

1. **serde + serde_json 의존성 추가** — `Cargo.toml`에 serde(전체), serde_json(네이티브 전용)
2. **35개 타입에 Serialize derive 추가** — 렌더 트리 전체 직렬화 가능
3. **FFI 함수 2개 추가** — `rhwp_render_page_tree`, `rhwp_image_data`
4. **DocumentCore 메서드 추가** — `build_page_render_tree`, `get_bin_data`
5. **C 헤더 갱신**

### Serialize 적용 타입 (35개)

| 파일 | 타입 수 | 주요 타입 |
|------|---------|-----------|
| `src/renderer/render_tree.rs` | 20 | RenderNode, RenderNodeType, PageNode, TextRunNode, TableNode 등 |
| `src/renderer/mod.rs` | 12 | TextStyle(44필드), ShapeStyle, LineStyle, PathCommand 등 |
| `src/model/style.rs` | 2 | UnderlineType, ImageFillMode |
| `src/model/control.rs` | 1 | FormType |
| `src/renderer/composer.rs` | 1 | CharOverlapInfo |
| `src/renderer/layout.rs` | 2 | CellContext, CellPathEntry |
| `src/renderer/equation/` | 3 | LayoutBox, LayoutKind, MatrixStyle, DecoKind, FontStyleKind |

- ImageNode.data, PageBackgroundImage.data: `#[serde(skip)]` (이미지는 별도 FFI)

### FFI API (신규)

```c
// 렌더 트리 JSON 반환 (rhwp_free_string으로 해제)
char *rhwp_render_page_tree(const RhwpHandle *handle, uint32_t page);

// 이미지 바이너리 참조 (bin_data_id는 1-indexed, 핸들 유효 동안만 사용)
const uint8_t *rhwp_image_data(const RhwpHandle *handle, uint16_t bin_data_id, size_t *out_len);
```

### 검증 결과

- `cargo build --target aarch64-apple-ios-sim --lib --release`: ✅ 성공
- `cargo test`: 785 passed, 0 failed (회귀 없음)
- SVG 내보내기: ✅ 기존 기능 정상 동작
