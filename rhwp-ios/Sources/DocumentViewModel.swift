import SwiftUI

/// 문서 뷰어의 뷰 모델. RhwpDocument 핸들의 수명을 뷰 생명주기와 동기화한다.
@MainActor
class DocumentViewModel: ObservableObject {
    @Published var document: RhwpDocument?
    @Published var currentPage: Int = 0
    @Published var pageTree: RenderNode?
    @Published var errorMessage: String?
    @Published var isLoading: Bool = false

    /// 문서의 총 페이지 수
    var pageCount: Int {
        document?.pageCount ?? 0
    }

    /// 현재 페이지의 크기 (포인트 단위)
    var currentPageSize: (width: Double, height: Double) {
        document?.pageSize(at: currentPage) ?? (0, 0)
    }

    /// HWP/HWPX 파일 데이터를 로드한다.
    func loadDocument(data: Data) {
        isLoading = true
        errorMessage = nil

        do {
            document = try RhwpDocument(data: data)
            currentPage = 0
            renderCurrentPage()
        } catch {
            errorMessage = error.localizedDescription
            document = nil
        }

        isLoading = false
    }

    /// 번들의 샘플 파일을 로드한다.
    func loadSampleFromBundle() {
        guard let url = Bundle.main.url(forResource: "sample", withExtension: "hwpx"),
              let data = try? Data(contentsOf: url) else {
            errorMessage = "샘플 파일을 찾을 수 없습니다."
            return
        }
        loadDocument(data: data)
    }

    /// 현재 페이지의 렌더 트리를 생성한다.
    func renderCurrentPage() {
        guard let doc = document else {
            pageTree = nil
            return
        }
        if let tree = doc.renderPageTree(at: currentPage) {
            pageTree = tree
        } else {
            errorMessage = "페이지 \(currentPage + 1) 렌더링 실패"
            pageTree = nil
        }
    }
}
