# Task #90: Rust aarch64-apple-ios 크로스 컴파일 파이프라인 — 수행계획서

## 목표

Rust rhwp 엔진을 iOS(aarch64-apple-ios) 타겟으로 크로스 컴파일하여 정적 라이브러리(.a)를 생성한다. Swift에서 호출할 수 있는 C-ABI 인터페이스를 포함한다.

## 배경

- 현재 rhwp는 네이티브(macOS/Linux) + WASM 두 가지 타겟으로 빌드
- iPad 앱(알한글)은 WKWebView가 아닌 **Rust 네이티브 렌더링**
- Rust 공식: `aarch64-apple-ios` 타겟 Tier 2 지원
- iOS Simulator: `aarch64-apple-ios-sim` 타겟

## 구현 단계

### 1단계: iOS 타겟 설치 + 크로스 컴파일 확인

- `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
- `cargo build --target aarch64-apple-ios --lib` 성공 여부 확인
- WASM 전용 의존성(`web-sys`, `wasm-bindgen`) 분리 필요성 파악

### 2단계: C-ABI FFI 인터페이스 설계

- `#[no_mangle] extern "C"` 함수로 iOS에서 호출할 API 정의
- 최소 API: HWP 파일 로드 → 페이지 수 → 페이지 SVG 렌더링
- cbindgen으로 C 헤더(.h) 자동 생성

### 3단계: Xcode 프로젝트 연동 테스트

- 생성된 .a 정적 라이브러리를 Xcode에서 링크
- Swift에서 Rust 함수 호출 확인
- iOS Simulator에서 동작 테스트

## 검증 기준

- `cargo build --target aarch64-apple-ios` 성공
- Swift에서 Rust 함수 호출하여 HWP 파일의 페이지 수 반환
