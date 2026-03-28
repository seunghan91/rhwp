import * as vscode from "vscode";

export class HwpEditorProvider implements vscode.CustomReadonlyEditorProvider {
  private static readonly viewType = "rhwp.hwpViewer";

  static register(context: vscode.ExtensionContext): vscode.Disposable {
    return vscode.window.registerCustomEditorProvider(
      HwpEditorProvider.viewType,
      new HwpEditorProvider(context),
      {
        webviewOptions: { retainContextWhenHidden: true },
        supportsMultipleEditorsPerDocument: false,
      }
    );
  }

  constructor(private readonly context: vscode.ExtensionContext) {}

  async openCustomDocument(
    uri: vscode.Uri,
    _openContext: vscode.CustomDocumentOpenContext,
    _token: vscode.CancellationToken
  ): Promise<vscode.CustomDocument> {
    return { uri, dispose: () => {} };
  }

  async resolveCustomEditor(
    document: vscode.CustomDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): Promise<void> {
    const webview = webviewPanel.webview;

    webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.context.extensionUri, "dist"),
      ],
    };

    webview.html = this.getHtml(webview);

    // Webview ready 후 HWP 파일 데이터만 전송 (WASM은 Webview에서 fetch)
    webview.onDidReceiveMessage(async (msg) => {
      if (msg.type === "ready") {
        const fileData = await vscode.workspace.fs.readFile(document.uri);
        const fileName = document.uri.path.split("/").pop() ?? "";

        webview.postMessage({
          type: "load",
          fileName,
          fileData: new Uint8Array(fileData),
        });
      }
    });
  }

  private getHtml(webview: vscode.Webview): string {
    const viewerUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, "dist", "webview", "viewer.js")
    );
    const wasmUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, "dist", "media", "rhwp_bg.wasm")
    );

    const nonce = getNonce();
    const cspSource = webview.cspSource;

    return /* html */ `<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none';
             script-src 'nonce-${nonce}' ${cspSource} 'unsafe-eval' 'wasm-unsafe-eval';
             style-src 'nonce-${nonce}' ${cspSource};
             img-src ${cspSource} data:;
             font-src ${cspSource};
             connect-src ${cspSource}">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>HWP Viewer</title>
  <style nonce="${nonce}">
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      background: var(--vscode-editor-background);
      color: var(--vscode-editor-foreground);
      overflow: hidden;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }
    #scroll-container {
      position: relative;
      overflow-y: auto;
      flex: 1;
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 12px;
      padding: 12px 0;
    }
    .page-wrapper {
      flex-shrink: 0;
      box-shadow: 0 2px 8px rgba(0,0,0,0.3);
      background: white;
    }
    /* 상태 표시줄 */
    #status-bar {
      display: flex;
      align-items: center;
      height: 26px;
      padding: 0 10px;
      background: var(--vscode-statusBar-background, #007acc);
      border-top: 1px solid var(--vscode-statusBar-border, transparent);
      flex-shrink: 0;
      font-size: 12px;
      color: var(--vscode-statusBar-foreground, #fff);
      user-select: none;
    }
    .stb-item {
      line-height: 26px;
      white-space: nowrap;
      flex-shrink: 0;
      padding: 0 4px;
    }
    .stb-divider {
      width: 1px;
      height: 14px;
      background: var(--vscode-statusBar-foreground, #fff);
      opacity: 0.3;
      margin: 0 6px;
      flex-shrink: 0;
    }
    .stb-message {
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      padding: 0 8px;
      line-height: 26px;
      opacity: 0.8;
    }
    .stb-right {
      display: flex;
      align-items: center;
      gap: 2px;
      flex-shrink: 0;
      margin-left: auto;
    }
    .stb-btn {
      height: 22px;
      border: none;
      border-radius: 3px;
      background: transparent;
      color: inherit;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      padding: 0 6px;
      font-size: 14px;
      line-height: 1;
    }
    .stb-btn:hover {
      background: rgba(255,255,255,0.15);
    }
    .stb-zoom-val {
      font-size: 12px;
      min-width: 40px;
      text-align: center;
      line-height: 26px;
    }
  </style>
</head>
<body>
  <div id="scroll-container" data-wasm-uri="${wasmUri}"></div>
  <div id="status-bar">
    <span id="stb-page" class="stb-item">- / - \uca4d</span>
    <span class="stb-divider"></span>
    <span id="stb-message" class="stb-message">\ubb38\uc11c\ub97c \ubd88\ub7ec\uc624\ub294 \uc911...</span>
    <span class="stb-right">
      <button id="stb-zoom-out" class="stb-btn" title="\ucd95\uc18c">\u2212</button>
      <span id="stb-zoom-val" class="stb-zoom-val">100%</span>
      <button id="stb-zoom-in" class="stb-btn" title="\ud655\ub300">+</button>
    </span>
  </div>
  <script nonce="${nonce}" src="${viewerUri}"></script>
</body>
</html>`;
  }
}

function getNonce(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let nonce = "";
  for (let i = 0; i < 32; i++) {
    nonce += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return nonce;
}
