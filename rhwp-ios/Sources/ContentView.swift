import SwiftUI

/// 앱 전체 구조를 담당하는 최상위 뷰.
/// 향후 탭 확장 시 TabView + 여러 DocumentView를 배치.
struct ContentView: View {
    @StateObject private var viewModel = DocumentViewModel()

    var body: some View {
        DocumentView(viewModel: viewModel)
            .onAppear {
                viewModel.loadSampleFromBundle()
            }
    }
}
