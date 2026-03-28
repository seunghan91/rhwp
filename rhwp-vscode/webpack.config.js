// @ts-check
const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");

/** @type {import('webpack').Configuration} Extension Host 번들 */
const extensionConfig = {
  target: "node",
  mode: "none",
  entry: "./src/extension.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "extension.js",
    libraryTarget: "commonjs2",
  },
  externals: {
    vscode: "commonjs vscode",
  },
  resolve: {
    extensions: [".ts", ".js"],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: [/node_modules/, /src\/webview/],
        use: {
          loader: "ts-loader",
          options: { configFile: "tsconfig.json" },
        },
      },
    ],
  },
  devtool: "nosources-source-map",
};

/** @type {import('webpack').Configuration} Webview 번들 */
const webviewConfig = {
  target: "web",
  mode: "none",
  entry: "./src/webview/viewer.ts",
  output: {
    path: path.resolve(__dirname, "dist", "webview"),
    filename: "viewer.js",
  },
  resolve: {
    extensions: [".ts", ".js"],
    alias: {
      "@rhwp-wasm": path.resolve(__dirname, "..", "pkg"),
    },
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        use: {
          loader: "ts-loader",
          options: { configFile: "tsconfig.webview.json" },
        },
      },
      {
        test: /\.wasm$/,
        type: "javascript/auto",
        loader: "null-loader",
      },
    ],
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        {
          from: path.resolve(__dirname, "..", "pkg", "rhwp_bg.wasm"),
          to: path.resolve(__dirname, "dist", "media", "rhwp_bg.wasm"),
        },
      ],
    }),
  ],
  devtool: "nosources-source-map",
};

module.exports = [extensionConfig, webviewConfig];
