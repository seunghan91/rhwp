---
# Stage 1 완료 보고서 — Task M100 #171
# XML 기반 구조 + fontfaces 직렬화
---

## 완료 일시
2026-04-18

## 작업 내용

### 변경 파일
| 파일 | 변경 내용 |
|------|----------|
| `src/serializer/hwpx/header.rs` | `EMPTY_HEADER_XML` include_str 제거, 동적 생성으로 전면 교체 |

### 구현 내용

1. **`write_header(doc: &Document)`** — 동적 XML 생성
   - XML 선언 + `<hh:head>` 열기 (14개 네임스페이스 + `version="1.2"`)
   - `<hh:beginNum>` 고정값
   - `<hh:refList>` 내부 7개 섹션 생성

2. **`write_fontfaces(out, font_faces)`** — IR → XML 동적 생성
   - 항상 7개 언어 그룹 (`HANGUL` ~ `USER`) 출력
   - `font_faces[i]` 가 비어 있으면 `fontCnt="0"`
   - 글꼴 이름 XML escape 처리
   - `<hh:typeInfo>` 고정 기본값

3. **Placeholder 함수들 (Stage 2~4에서 교체 예정)**
   - `border_fills_placeholder` — ID 1, 2 (최소)
   - `char_properties_placeholder` — ID 0 (최소)
   - `tab_properties_placeholder` — ID 0 (최소)
   - `para_properties_placeholder` — ID 0 (최소)
   - `styles_placeholder` — 바탕글 1개
   - numberings: `itemCnt="0"` 빈 목록

4. **단위 테스트 4개** (모두 통과)
   - `empty_doc_has_seven_fontfaces` — 빈 Document → 7개 lang 그룹 확인
   - `font_name_appears_in_fontfaces` — 글꼴 이름/fontCnt 확인
   - `font_face_name_xml_escaped` — `A&B<C` → `A&amp;B&lt;C` escape 확인
   - `font_faces_roundtrip` — serialize → parse_hwpx_header → IR 동등성 확인

## 테스트 결과

```
running 14 tests (serializer::hwpx 전체)
... 14 passed; 0 failed
```

기존 HWPX 직렬화 테스트 14개 모두 회귀 없음.

## 비고
- `concat!` 매크로 내 `"#` 파싱 오류 → `fn` 방식으로 전환
- pre-existing 실패(cfb_writer, wasm_api) 14개는 `\` 경로 문자 문제로 본 작업과 무관

---

> **Stage 1 완료 승인 요청**: 위 내용 확인 후 승인해 주시면 **Stage 2 (charPr 직렬화)** 를 시작하겠습니다.
