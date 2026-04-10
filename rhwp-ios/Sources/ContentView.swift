import SwiftUI
import WebKit

struct ContentView: View {
    @State private var pageCount: Int = 0
    @State private var svgContent: String = ""
    @State private var statusMessage: String = "샘플 파일 로딩 중..."

    var body: some View {
        VStack(spacing: 0) {
            // 상단 바
            HStack {
                Text("알한글")
                    .font(.headline)
                    .fontWeight(.bold)
                Spacer()
                Text("\(pageCount)페이지")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .background(Color(UIColor.systemBackground))

            // SVG 뷰어
            if !svgContent.isEmpty {
                SVGWebView(svgContent: svgContent)
                    .edgesIgnoringSafeArea(.bottom)
            } else {
                VStack {
                    Spacer()
                    Text(statusMessage)
                        .foregroundColor(.secondary)
                    Spacer()
                }
            }
        }
        .onAppear {
            loadSampleHWP()
        }
    }

    private func loadSampleHWP() {
        // 번들에 포함된 샘플 HWP 파일 로드
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "hwpx"),
              let data = try? Data(contentsOf: url) else {
            statusMessage = "샘플 파일을 찾을 수 없습니다."
            return
        }

        // Rust FFI 호출
        let result = data.withUnsafeBytes { (ptr: UnsafeRawBufferPointer) -> Bool in
            guard let baseAddress = ptr.baseAddress else { return false }
            let handle = rhwp_open(baseAddress.assumingMemoryBound(to: UInt8.self), data.count)
            guard handle != nil else {
                statusMessage = "HWP 파싱 실패"
                return false
            }
            defer { rhwp_close(handle) }

            let count = rhwp_page_count(handle)
            pageCount = Int(count)

            // 첫 페이지 SVG 렌더링
            if let svgPtr = rhwp_render_page_svg(handle, 0) {
                svgContent = String(cString: svgPtr)
                rhwp_free_string(svgPtr)
            }

            statusMessage = ""
            return true
        }

        if !result {
            statusMessage = "파일 로드 실패"
        }
    }
}

// WKWebView로 SVG 렌더링
struct SVGWebView: UIViewRepresentable {
    let svgContent: String

    func makeUIView(context: Context) -> WKWebView {
        let webView = WKWebView()
        webView.scrollView.minimumZoomScale = 0.5
        webView.scrollView.maximumZoomScale = 5.0
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {
        let html = """
        <!DOCTYPE html>
        <html>
        <head>
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5">
        <style>
        body { margin: 0; display: flex; justify-content: center; background: #f5f5f7; }
        svg { max-width: 100%; height: auto; background: white; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }
        </style>
        </head>
        <body>\(svgContent)</body>
        </html>
        """
        webView.loadHTMLString(html, baseURL: nil)
    }
}
