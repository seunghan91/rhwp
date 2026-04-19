# Stage 5 완료 보고서 — Task M100 #186
# 통합 검증 (ref 샘플 라운드트립)

## 완료 일자
2026-04-18

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/section.rs` | render_paragraph_runs: 소비되지 않은 인라인 컨트롤 후처리 추가 |
| `src/serializer/hwpx/mod.rs` | Stage 5 통합 테스트 3개 추가 |

## 구현 요약

### 핵심 버그 수정 (section.rs)

**문제**: HWPX 파서가 `\u{0002}` 제어문자를 `para.text`에서 필터링(line 299-305):
```rust
para.text = text_parts.iter()
    .filter(|s| *s != "\u{0002}")  // ← 필터링
    .cloned().collect::<Vec<_>>().join("");
```

결과: HWPX 파싱 후 직렬화 시 `render_paragraph_runs`가 `\u{0002}`를 만나지 못해
표/그림 컨트롤을 전혀 출력하지 않음.

**수정**: 텍스트 루프 종료 후 `ctrl_idx` 이후 남은 인라인 컨트롤을 별도 run으로 출력:

```rust
// 마지막 run 닫기
emit_run!();

// HWPX 파서는 \u{0002}를 para.text에서 필터링하므로, ctrl_idx가 소비되지 않은
// 인라인 컨트롤(Table/Picture)을 별도 run으로 후출력한다.
for ctrl in &controls[ctrl_idx..] {
    match ctrl {
        Control::Table(tbl) => {
            write_table(&mut ctrl_section, tbl, default_tab_width);
            emit_run!();
        }
        Control::Picture(pic) => {
            write_picture(&mut ctrl_section, pic);
            emit_run!();
        }
        _ => {}
    }
}
```

이 수정은 기존 단위 테스트(수동 `\u{0002}` 설정)와 충돌 없음:
- `ctrl_idx`가 이미 소비된 컨트롤은 건너뜀
- 텍스트 루프 내 `\u{0002}` 처리 경로는 그대로 유지

### 통합 테스트 3개 (mod.rs)

| 테스트 | 검증 내용 |
|--------|----------|
| `ref_table_roundtrip` | ref_table.hwpx → 직렬화 → 재파싱 → row_count/col_count/borderFillIDRef 보존 |
| `ref_mixed_roundtrip` | ref_mixed.hwpx → 직렬화 → 재파싱 → 문단 수 및 문단 텍스트 보존 |
| `ref_text_roundtrip` | ref_text.hwpx → 직렬화 → 재파싱 → 섹션 수 및 문단 텍스트 보존 |

## 테스트 결과

| 테스트 | 결과 |
|--------|------|
| `ref_table_roundtrip` | ✅ row/col/borderFill 라운드트립 |
| `ref_mixed_roundtrip` | ✅ 문단 텍스트 보존 |
| `ref_text_roundtrip` | ✅ 문단 텍스트 보존 |
| 기존 43개 | ✅ 회귀 없음 |
| **합계** | **46 passed** |

기존 실패 14개(cfb_writer 2개, wasm_api 12개)는 이 작업 이전부터 실패 중인 pre-existing 항목.
