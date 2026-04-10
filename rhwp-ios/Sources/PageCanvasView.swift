// PageCanvasView — Core Graphics로 렌더 트리를 그리는 UIView
// UIViewRepresentable로 SwiftUI에서 사용

import SwiftUI
import UIKit

/// Core Graphics 기반 페이지 렌더링 뷰
class PageCanvasUIView: UIView {
    var renderTree: RenderNode?
    var document: RhwpDocument?
    let renderer = CGTreeRenderer()
    private var pageHeight: Double = 0

    func configure(tree: RenderNode?, pageHeight: Double, document: RhwpDocument?) {
        self.renderTree = tree
        self.pageHeight = pageHeight
        self.document = document
        self.layer.contentsScale = UIScreen.main.scale
        self.backgroundColor = .white
        setNeedsDisplay()
    }

    override func draw(_ rect: CGRect) {
        guard let ctx = UIGraphicsGetCurrentContext(), let tree = renderTree else { return }
        renderer.render(tree: tree, in: ctx, pageHeight: pageHeight, document: document)
    }
}

/// SwiftUI 래퍼
struct PageCanvasView: UIViewRepresentable {
    let renderTree: RenderNode?
    let pageHeight: Double
    let document: RhwpDocument?

    func makeUIView(context: Context) -> PageCanvasUIView {
        let view = PageCanvasUIView()
        view.configure(tree: renderTree, pageHeight: pageHeight, document: document)
        return view
    }

    func updateUIView(_ uiView: PageCanvasUIView, context: Context) {
        uiView.configure(tree: renderTree, pageHeight: pageHeight, document: document)
    }
}
