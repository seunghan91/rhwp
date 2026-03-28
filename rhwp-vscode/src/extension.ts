import * as vscode from "vscode";
import { HwpEditorProvider } from "./hwp-editor-provider";

export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(HwpEditorProvider.register(context));
}

export function deactivate() {}
