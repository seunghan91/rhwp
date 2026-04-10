import SwiftUI

/// 앱 전체 구조를 담당하는 최상위 뷰.
/// 향후 탭 확장 시 TabView + 여러 DocumentView를 배치.
struct ContentView: View {
    @StateObject private var viewModel = DocumentViewModel()
    @State private var showFilePicker = false

    var body: some View {
        DocumentView(viewModel: viewModel)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        showFilePicker = true
                    } label: {
                        Image(systemName: "folder.badge.plus")
                    }
                }
            }
            .sheet(isPresented: $showFilePicker) {
                DocumentPickerView { data, filename in
                    viewModel.loadDocument(data: data, filename: filename)
                }
            }
            .onAppear {
                // 번들 샘플이 있으면 자동 로드
                if viewModel.document == nil {
                    viewModel.loadSampleFromBundle()
                }
            }
    }
}
