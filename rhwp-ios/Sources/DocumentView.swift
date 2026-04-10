import SwiftUI
import WebKit

/// 단일 HWP 문서를 렌더링하는 뷰.
/// 향후 탭 확장 시 탭 1개 = DocumentView 1개로 대응.
struct DocumentView: View {
    @ObservedObject var viewModel: DocumentViewModel

    var body: some View {
        VStack(spacing: 0) {
            // 상단 정보 바
            HStack {
                Text("알한글")
                    .font(.headline)
                    .fontWeight(.bold)
                Spacer()
                if viewModel.pageCount > 0 {
                    let size = viewModel.currentPageSize
                    Text("\(viewModel.currentPage + 1)/\(viewModel.pageCount)쪽")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    if size.width > 0 {
                        Text("(\(Int(size.width))×\(Int(size.height))pt)")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .background(Color(UIColor.systemBackground))

            // 문서 렌더링 영역
            if viewModel.isLoading {
                Spacer()
                ProgressView("로딩 중...")
                Spacer()
            } else if !viewModel.svgContent.isEmpty {
                SVGWebView(svgContent: viewModel.svgContent)
                    .edgesIgnoringSafeArea(.bottom)
            } else if let error = viewModel.errorMessage {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundColor(.orange)
                    Text(error)
                        .foregroundColor(.secondary)
                        .multilineTextAlignment(.center)
                }
                .padding()
                Spacer()
            } else {
                Spacer()
                Text("문서를 열어주세요.")
                    .foregroundColor(.secondary)
                Spacer()
            }
        }
    }
}

/// WKWebView로 SVG 렌더링
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
