# Task #91 구현계획서 리뷰 피드백

리뷰어: iOS 네이티브 + Rust FFI 시니어 리뷰어  
대상: Task #91 FFI 브릿지 고도화 구현계획서 (3단계)  
일자: 2026-04-10

---

## 1. FFI 설계

### 1-1. catch_unwind 적용 — 필수이나 전제조건 확인 필요

`catch_unwind`로 패닉 방어하는 방향은 올바르다. 다만 아래 사항을 확인해야 한다.

- **panic = "abort" 여부**: 현재 `Cargo.toml`에 `[profile.release]` panic 설정이 없으므로 기본값(`unwind`)이 적용되어 `catch_unwind`가 작동한다. 그러나 iOS 빌드용 `.cargo/config.toml`에서 `panic = "abort"`를 설정하는 경우가 있으므로, iOS 타깃 빌드 설정을 명시적으로 문서화할 것.
- **적용 범위**: `rhwp_open`, `rhwp_render_page_svg`, 새로 추가하는 `rhwp_document_info`, `rhwp_page_info`에는 반드시 적용해야 한다. 반면 `rhwp_page_count`처럼 단순 정수 반환 함수, `rhwp_free_string`, `rhwp_close`는 패닉 가능성이 극히 낮으므로 null 체크만으로 충분하다. 모든 함수에 일괄 적용하면 불필요한 오버헤드와 코드 노이즈가 생긴다.
- **catch_unwind 래퍼 매크로 권장**: 반복 보일러플레이트를 줄이기 위해 다음과 같은 매크로를 도입할 것을 권장한다.

```rust
macro_rules! ffi_guard {
    ($handle:expr, $default:expr, $body:expr) => {{
        if $handle.is_null() { return $default; }
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(_) => $default,
        }
    }};
}
```

### 1-2. last_error 패턴 — 구조 변경 필요

`RhwpHandle`에 `last_error: Option<CString>` 필드를 추가하는 방안은 **스레드 안전성 문제**가 있다.

- **문제점**: `RhwpHandle`은 `&*handle` (불변 참조)로 접근하는데, `last_error`를 갱신하려면 가변 접근이 필요하다. `&mut *handle`로 바꾸면 Rust의 앨리어싱 규칙을 위반할 수 있다.
- **권장안 A (단순)**: `last_error`를 `Cell<Option<CString>>` 또는 `RefCell<Option<CString>>`로 감싼다. 단일 스레드 접근을 가정하므로 `RefCell`이면 충분하다.
- **권장안 B (스레드 안전)**: `Mutex<Option<CString>>`을 사용한다. 향후 #93 네이티브 렌더러에서 백그라운드 렌더링을 도입할 가능성이 있으므로 `Mutex`가 더 안전하다.
- **권장안 C (가장 단순, M1 추천)**: `last_error`를 핸들에 넣지 말고, 에러를 반환하는 함수가 에러 시 null을 반환하는 현재 패턴을 유지한다. M1은 최소 뷰어이므로 에러 상세 메시지는 로그로 확인하면 된다. `rhwp_error_message` 함수는 M2 이후로 미루는 것을 권장한다.

### 1-3. rhwp_document_info / rhwp_page_info의 JSON 반환

- JSON 문자열 반환 방식 자체는 유연하고 확장성이 좋다. 다만 `serde_json` 의존성이 현재 `Cargo.toml`에 없다. 추가해야 하며, iOS staticlib 빌드 시 바이너리 크기 영향을 확인할 것.
- **대안 검토**: 페이지 크기/여백 정도라면 C 구조체로 직접 반환하는 것이 파싱 오버헤드 없이 더 간결하다. JSON은 필드가 10개 이상이거나 가변적일 때 의미가 있다.

```rust
#[repr(C)]
pub struct RhwpPageSize {
    width_pt: f64,
    height_pt: f64,
}
```

M1에서 필요한 메타데이터가 페이지 크기뿐이라면 C 구조체를, 향후 확장이 확실하다면 JSON을 선택할 것.

---

## 2. 메모리 관리

### 2-1. 문자열 소유권 이전 — 현재 패턴 유지

`CString::into_raw()` → Swift에서 사용 → `rhwp_free_string()`으로 해제하는 패턴은 올바르다. 새로 추가하는 `rhwp_document_info`, `rhwp_page_info`도 동일 패턴을 적용하면 된다.

### 2-2. rhwp_error_message 반환 방식 주의

계획서에서 `rhwp_error_message`가 `*const c_char`(불변 포인터)를 반환한다고 되어 있다. 이 경우:

- Swift 측에서 `rhwp_free_string`을 호출하면 **안 된다** (소유권이 핸들에 남아있으므로).
- 하지만 `*const`와 `*mut`의 구분을 Swift는 인식하지 못한다 — 둘 다 `UnsafeMutablePointer?`로 보인다.
- **해제 규칙이 함수마다 다르면 반드시 버그가 발생한다.** 통일해야 한다.
- **권장**: 에러 메시지도 `*mut c_char`로 반환하고 `rhwp_free_string`으로 해제하게 하거나, 앞서 권장한 대로 이 함수 자체를 M1에서 제외한다.

### 2-3. Swift Data.withUnsafeBytes 범위 밖에서의 핸들 사용

현재 ContentView의 `loadSampleHWP()`에서 `data.withUnsafeBytes` 클로저 내부에서 `rhwp_open`을 호출하고 `defer { rhwp_close(handle) }`로 즉시 닫고 있다. 계획서의 `RhwpDocument` 래퍼에서 핸들을 유지하려면:

- `rhwp_open`이 내부적으로 데이터를 복사하여 소유하고 있으므로 (파싱 후 IR로 변환), `withUnsafeBytes` 밖에서 핸들을 사용해도 안전하다. 이 점을 계획서에 명시적으로 기술할 것. 향후 zero-copy 파싱을 도입하면 이 가정이 깨지므로 주석을 남겨두는 것이 좋다.

---

## 3. Swift 래퍼 설계

### 3-1. class vs struct — class가 올바른 선택

`RhwpDocument`를 `class`로 구현하는 것은 올바르다. `deinit`에서 `rhwp_close`를 호출해야 하므로 참조 타입이 필수다. 값 타입(`struct`)은 복사 시 double-free가 발생한다.

### 3-2. @StateObject 사용 — ObservableObject 채택 필요

`@StateObject`를 사용하려면 `RhwpDocument`가 `ObservableObject`를 채택해야 한다. 계획서에 이 부분이 누락되어 있다.

```swift
class RhwpDocument: ObservableObject {
    @Published var pageCount: Int = 0
    // ...
}
```

그러나 `RhwpDocument`를 `ObservableObject`로 만드는 것이 적절한지 재검토가 필요하다.

- **RhwpDocument는 뷰 모델이 아니라 데이터 모델이다.** 읽기 전용 문서를 표현하므로 상태 변화가 거의 없다.
- **권장 구조**: `RhwpDocument`는 순수 래퍼로 유지하고, 별도의 `DocumentViewModel: ObservableObject`를 만들어 @StateObject로 사용한다.

```swift
// 데이터 모델 (FFI 래퍼)
class RhwpDocument {
    private var handle: OpaquePointer
    // ...
}

// 뷰 모델
class DocumentViewModel: ObservableObject {
    @Published var document: RhwpDocument?
    @Published var currentPage: Int = 0
    @Published var svgContent: String = ""
    @Published var errorMessage: String?
}
```

이 구조가 #92 다중 페이지 탐색에서 `currentPage` 상태 관리에 자연스럽게 확장된다.

### 3-3. Sendable 준수

`RhwpDocument`는 내부에 `OpaquePointer`를 가지므로 기본적으로 `Sendable`이 아니다. Swift 6 strict concurrency 환경에서 `@MainActor` 외 컨텍스트로 전달 시 컴파일 에러가 발생한다.

- M1 단계에서는 `@MainActor`로 제한하여 메인 스레드에서만 접근하도록 하면 충분하다.
- `@unchecked Sendable` 채택은 M3 이후 백그라운드 렌더링 도입 시에 검토한다.

### 3-4. init? (failable initializer) vs throws

`init?(data: Data)`보다 `init(data: Data) throws`를 권장한다. 실패 원인(파일 손상, 지원하지 않는 버전 등)을 에러 타입으로 전달할 수 있어 UX가 좋아진다.

```swift
enum RhwpError: LocalizedError {
    case parseFailure(String)
    case invalidData
}
```

---

## 4. 누락된 고려사항

### 4-1. #92 다중 페이지와의 호환성

현재 계획서의 API는 `renderPageSVG(at:)`로 개별 페이지를 렌더링한다. #92에서 페이지 간 스크롤을 구현하려면:

- 인접 페이지 프리페치가 필요하다. 현재 API로 가능하나, **렌더링 결과 캐싱**이 필요하다. `RhwpDocument` 또는 `DocumentViewModel`에 SVG 캐시(`[Int: String]`)를 두는 것을 권장한다.
- 계획서에서 캐싱에 대한 언급이 없다. 1단계나 3단계에 캐시 설계를 포함하거나, 최소한 "캐싱은 #92에서 구현" 이라고 명시할 것.

### 4-2. #93 네이티브 렌더러와의 호환성

#93에서 SVG 대신 Core Graphics로 직접 렌더링할 경우, `rhwp_render_page_svg` 대신 렌더 커맨드 배열을 반환하는 FFI가 필요하다. 현재 계획서는 SVG 기반이므로 #93과 충돌하지 않지만, `RhwpDocument` 래퍼에 `renderPage` 메서드를 추가할 때 SVG 전용 네이밍(`renderPageSVG`)을 사용한 점은 적절하다. 향후 `renderPageCommands`를 별도로 추가할 수 있다.

### 4-3. 파일 포맷 구분

현재 `rhwp_open`은 바이트 배열을 받아 HWP/HWPX를 자동 감지한다. 그러나 Swift 측에서 파일 확장자 기반으로 UTI를 처리해야 하는 상황(Document Browser, Share Extension 등)이 올 수 있으므로, `rhwp_document_info`의 JSON에 `format` 필드("hwp" 또는 "hwpx")를 포함하는 것을 권장한다.

---

## 5. 과잉 설계 여부

### 5-1. rhwp_document_info — M1에서 필요한가?

M1 최소 뷰어에서 문서 메타데이터(제목, 작성자 등)를 표시하는 UI가 없다면 불필요하다. 페이지 수는 이미 `rhwp_page_count`로 얻고 있다. **문서 정보 표시 UI가 M1 범위에 포함되는지 확인 후 결정할 것.**

### 5-2. rhwp_error_message — M1에서 불필요

앞서 기술한 대로, M1에서는 null 반환으로 에러를 판단하고 Xcode 콘솔의 로그로 디버깅하면 충분하다. last_error 패턴은 스레드 안전성, 소유권 문제를 동반하므로 M2 이후 도입을 권장한다.

### 5-3. rhwp_page_info — 유지 권장

페이지 크기/여백은 SVG 뷰포트 설정, 페이지 간 간격 계산에 필요하므로 M1에서도 유효하다. 다만 JSON이 아닌 C 구조체 반환을 권장한다 (1-3항 참조).

---

## 6. 종합 권장사항

| 항목 | 권장 |
|------|------|
| catch_unwind | 패닉 가능성 있는 함수에만 선별 적용. 매크로로 보일러플레이트 제거 |
| last_error / rhwp_error_message | M1에서 제외. M2에서 Mutex 기반으로 도입 |
| rhwp_document_info | M1 UI 범위에 문서 정보 표시가 없으면 제외 |
| rhwp_page_info | 유지. C 구조체 반환 방식 검토 |
| RhwpDocument | class 유지, throws init, ObservableObject 분리 |
| ViewModel | DocumentViewModel 별도 분리하여 @StateObject 적용 |
| Sendable | M1은 @MainActor 제한, M3 이후 검토 |
| SVG 캐시 | 설계에 언급하거나 #92로 명시적 이관 |

**결론**: 전체 방향은 적절하나, M1 범위에서 last_error/document_info는 과잉이다. 핵심인 Swift 래퍼 + 핸들 수명 관리 + 페이지 크기 조회에 집중하고, 에러 처리 고도화는 M2로 미룰 것을 권장한다.
