import SwiftUI

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
            } else if viewModel.pageTree != nil {
                ScrollView {
                    let size = viewModel.currentPageSize
                    PageCanvasView(
                        renderTree: viewModel.pageTree,
                        pageHeight: size.height,
                        document: viewModel.document
                    )
                    .frame(width: CGFloat(size.width), height: CGFloat(size.height))
                    .background(Color.white)
                    .shadow(color: .black.opacity(0.1), radius: 4, x: 0, y: 2)
                    .padding(.vertical, 8)
                }
                .background(Color(UIColor.systemGroupedBackground))
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
