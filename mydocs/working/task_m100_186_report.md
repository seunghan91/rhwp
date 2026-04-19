# 최종 결과 보고서 — Task M100 #186
# section.xml 완전 동적화 (그림·표·다중 CharShape·BinData)

## 완료 일자
2026-04-18

## 작업 브랜치
`local/task186`

## 구현 범위

| Stage | 내용 | 테스트 |
|-------|------|--------|
| Stage 1 | secPr/pagePr 동적화 | 4개 |
| Stage 2 | 다중 CharShape run 분리 | 2개 |
| Stage 3 | 표(Control::Table) 직렬화 | 4개 |
| Stage 4 | 그림(Control::Picture) + BinData ZIP | 3개 |
| Stage 5 | ref 샘플 라운드트립 통합 검증 | 3개 |

## 변경 파일 요약

| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/section.rs` | render_paragraph_runs 재설계 (3-버퍼 + emit_run!), 인라인 컨트롤 후처리 |
| `src/serializer/hwpx/table.rs` | **신규**: write_table, write_cell, write_cell_para |
| `src/serializer/hwpx/picture.rs` | **신규**: write_picture, bin_data_path, media_type |
| `src/serializer/hwpx/mod.rs` | collect_bin_data, 모듈 선언, 단계별 테스트 16개 |

## 최종 테스트 결과

**46 passed** (기존 pre-existing 실패 14개 제외)

| 테스트 그룹 | 수 | 결과 |
|------------|-----|------|
| Stage 1 pagePr 동적화 | 4 | ✅ |
| Stage 2 다중 run | 2 | ✅ |
| Stage 3 표 직렬화 | 4 | ✅ |
| Stage 4 그림 + BinData | 3 | ✅ |
| Stage 5 라운드트립 | 3 | ✅ |
| 이전 Stage 0 기본 | 30 | ✅ |

## 주요 설계 결정

### 3-버퍼 run 빌딩
- `ctrl_section`: 인라인 개체 XML (`<hp:t>` 이전에 위치)
- `t_content`: 탭/줄바꿈/텍스트 내용
- `char_buf`: xml_escape 대기 문자
- `emit_run!()` 매크로: 세 버퍼를 조합하여 `<hp:run>` 출력

### HWPX 파서 `\u{0002}` 필터링 대응
HWPX 파서는 `para.text`에서 `\u{0002}` 제어문자를 제거한다. 직렬화 시
텍스트 루프 후 미소비 인라인 컨트롤을 별도 run으로 후출력하여 라운드트립 보장.

### `<hp:tbl>` 위치
`<hp:run>` 내 `<hp:t>` 이전에 배치 (ref_table.hwpx 분석 기반).

### BinData 수집 전략
문서 전체 문단+셀 재귀 순회 → Picture 컨트롤 발견 시 `bin_data_id` →
`doc.bin_data_content`에서 바이트 탐색 → ZIP `BinData/image{id}.{ext}` 저장.
