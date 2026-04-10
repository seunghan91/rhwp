// rhwp iOS FFI — 자동 생성 헤더 (cbindgen)

#ifndef RHWP_H
#define RHWP_H

// 이 파일은 cbindgen으로 자동 생성됩니다. 수동 편집 금지.

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * FileHeader 크기 (바이트)
 */
#define FILE_HEADER_SIZE 256

/**
 * 태그 시작 기준값
 */
#define HWPTAG_BEGIN 16

/**
 * 문서 속성 (섹션 수, 시작 번호 등)
 */
#define HWPTAG_DOCUMENT_PROPERTIES HWPTAG_BEGIN

/**
 * ID 매핑 테이블 (각 타입별 개수)
 */
#define HWPTAG_ID_MAPPINGS (HWPTAG_BEGIN + 1)

/**
 * 바이너리 데이터 참조
 */
#define HWPTAG_BIN_DATA (HWPTAG_BEGIN + 2)

/**
 * 글꼴 이름
 */
#define HWPTAG_FACE_NAME (HWPTAG_BEGIN + 3)

/**
 * 테두리/채우기
 */
#define HWPTAG_BORDER_FILL (HWPTAG_BEGIN + 4)

/**
 * 글자 모양
 */
#define HWPTAG_CHAR_SHAPE (HWPTAG_BEGIN + 5)

/**
 * 탭 정의
 */
#define HWPTAG_TAB_DEF (HWPTAG_BEGIN + 6)

/**
 * 번호 매기기
 */
#define HWPTAG_NUMBERING (HWPTAG_BEGIN + 7)

/**
 * 글머리표
 */
#define HWPTAG_BULLET (HWPTAG_BEGIN + 8)

/**
 * 문단 모양
 */
#define HWPTAG_PARA_SHAPE (HWPTAG_BEGIN + 9)

/**
 * 스타일
 */
#define HWPTAG_STYLE (HWPTAG_BEGIN + 10)

/**
 * 문서 데이터
 */
#define HWPTAG_DOC_DATA (HWPTAG_BEGIN + 11)

/**
 * 배포용 문서 데이터 (복호화 시드)
 */
#define HWPTAG_DISTRIBUTE_DOC_DATA (HWPTAG_BEGIN + 12)

/**
 * 호환 문서
 */
#define HWPTAG_COMPATIBLE_DOCUMENT (HWPTAG_BEGIN + 14)

/**
 * 레이아웃 호환성
 */
#define HWPTAG_LAYOUT_COMPATIBILITY (HWPTAG_BEGIN + 15)

/**
 * 변경 추적
 */
#define HWPTAG_TRACKCHANGE (HWPTAG_BEGIN + 16)

/**
 * 문단 헤더
 */
#define HWPTAG_PARA_HEADER (HWPTAG_BEGIN + 50)

/**
 * 문단 텍스트 (UTF-16LE)
 */
#define HWPTAG_PARA_TEXT (HWPTAG_BEGIN + 51)

/**
 * 문단 글자 모양 참조
 */
#define HWPTAG_PARA_CHAR_SHAPE (HWPTAG_BEGIN + 52)

/**
 * 문단 줄 세그먼트
 */
#define HWPTAG_PARA_LINE_SEG (HWPTAG_BEGIN + 53)

/**
 * 문단 범위 태그
 */
#define HWPTAG_PARA_RANGE_TAG (HWPTAG_BEGIN + 54)

/**
 * 컨트롤 헤더
 */
#define HWPTAG_CTRL_HEADER (HWPTAG_BEGIN + 55)

/**
 * 리스트 헤더 (셀, 머리말/꼬리말 등의 문단 목록)
 */
#define HWPTAG_LIST_HEADER (HWPTAG_BEGIN + 56)

/**
 * 용지 설정
 */
#define HWPTAG_PAGE_DEF (HWPTAG_BEGIN + 57)

/**
 * 각주/미주 모양
 */
#define HWPTAG_FOOTNOTE_SHAPE (HWPTAG_BEGIN + 58)

/**
 * 쪽 테두리/배경
 */
#define HWPTAG_PAGE_BORDER_FILL (HWPTAG_BEGIN + 59)

/**
 * 그리기 개체 속성
 */
#define HWPTAG_SHAPE_COMPONENT (HWPTAG_BEGIN + 60)

/**
 * 표 속성
 */
#define HWPTAG_TABLE (HWPTAG_BEGIN + 61)

/**
 * 직선
 */
#define HWPTAG_SHAPE_COMPONENT_LINE (HWPTAG_BEGIN + 62)

/**
 * 사각형
 */
#define HWPTAG_SHAPE_COMPONENT_RECTANGLE (HWPTAG_BEGIN + 63)

/**
 * 타원
 */
#define HWPTAG_SHAPE_COMPONENT_ELLIPSE (HWPTAG_BEGIN + 64)

/**
 * 호
 */
#define HWPTAG_SHAPE_COMPONENT_ARC (HWPTAG_BEGIN + 65)

/**
 * 다각형
 */
#define HWPTAG_SHAPE_COMPONENT_POLYGON (HWPTAG_BEGIN + 66)

/**
 * 곡선
 */
#define HWPTAG_SHAPE_COMPONENT_CURVE (HWPTAG_BEGIN + 67)

/**
 * OLE 개체
 */
#define HWPTAG_SHAPE_COMPONENT_OLE (HWPTAG_BEGIN + 68)

/**
 * 그림
 */
#define HWPTAG_SHAPE_COMPONENT_PICTURE (HWPTAG_BEGIN + 69)

/**
 * 컨테이너 (그리기 묶음)
 */
#define HWPTAG_SHAPE_COMPONENT_CONTAINER (HWPTAG_BEGIN + 70)

/**
 * 컨트롤 데이터
 */
#define HWPTAG_CTRL_DATA (HWPTAG_BEGIN + 71)

/**
 * 수식
 */
#define HWPTAG_EQEDIT (HWPTAG_BEGIN + 72)

/**
 * 글맵시
 */
#define HWPTAG_SHAPE_COMPONENT_TEXTART (HWPTAG_BEGIN + 74)

/**
 * 양식 개체
 */
#define HWPTAG_FORM_OBJECT (HWPTAG_BEGIN + 75)

/**
 * 메모 모양
 */
#define HWPTAG_MEMO_SHAPE (HWPTAG_BEGIN + 76)

/**
 * 메모 리스트
 */
#define HWPTAG_MEMO_LIST (HWPTAG_BEGIN + 77)

/**
 * 금칙 문자
 */
#define HWPTAG_FORBIDDEN_CHAR (HWPTAG_BEGIN + 78)

/**
 * 차트 데이터
 */
#define HWPTAG_CHART_DATA (HWPTAG_BEGIN + 79)

/**
 * 구역/단 정의 컨트롤
 */
#define CHAR_SECTION_COLUMN_DEF 2

/**
 * 필드 시작
 */
#define CHAR_FIELD_BEGIN 3

/**
 * 필드 끝
 */
#define CHAR_FIELD_END 4

/**
 * 인라인 컨트롤 (비텍스트)
 */
#define CHAR_INLINE_NON_TEXT 8

/**
 * 컨트롤 삽입 위치 (표, 도형, 그림 등)
 */
#define CHAR_EXTENDED_CTRL 11

/**
 * 문단 끝/나눔
 */
#define CHAR_LINE_BREAK 10

/**
 * 문단 나눔
 */
#define CHAR_PARA_BREAK 13

/**
 * 하이픈
 */
#define CHAR_HYPHEN 30

/**
 * 고정폭 공백
 */
#define CHAR_NBSPACE 24

/**
 * 고정폭 하이픈
 */
#define CHAR_FIXED_WIDTH_SPACE 25

/**
 * 고정폭 빈칸 (스펙 코드 31)
 */
#define CHAR_FIXED_WIDTH_SPACE_31 31

/**
 * HWPUNIT → 픽셀 변환 (96 DPI 기준)
 */
#define DEFAULT_DPI 96.0

#define HWPUNIT_PER_INCH 7200.0

/**
 * HWP 언어 카테고리 수 (한국어, 영어, 한자, 일본어, 기타, 기호, 사용자)
 */
#define LANG_COUNT 7

/**
 * 불투명 핸들 (Swift에서 포인터로 전달)
 */
typedef struct RhwpHandle RhwpHandle;

extern double js_measure_text_width(const str *font, const str *text);

/**
 * HWP 파일 데이터를 로드하여 핸들을 반환한다.
 * 실패 시 null 반환.
 */
struct RhwpHandle *rhwp_open(const uint8_t *data, uintptr_t len);

/**
 * 문서의 페이지 수를 반환한다.
 */
uint32_t rhwp_page_count(const struct RhwpHandle *handle);

/**
 * 특정 페이지를 SVG 문자열로 렌더링한다.
 * 호출자는 반환된 문자열을 `rhwp_free_string`으로 해제해야 한다.
 * 실패 시 null 반환.
 */
char *rhwp_render_page_svg(const struct RhwpHandle *handle, uint32_t page);

/**
 * `rhwp_render_page_svg`가 반환한 문자열을 해제한다.
 */
void rhwp_free_string(char *ptr);

/**
 * 문서 핸들을 해제한다.
 */
void rhwp_close(struct RhwpHandle *handle);

#endif  /* RHWP_H */
