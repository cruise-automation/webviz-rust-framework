/* eslint-env node */
/* eslint-disable @typescript-eslint/no-var-requires */

const path = require("path");

// TODO(Paras): Export type definitions for our library builds, both for TypeScript
// and potentially Flow, using something like https://github.com/joarwilk/flowgen.

module.exports = {
  entry: {
    /* eslint-disable camelcase */
    wrf_wasm_runtime: "./wrf_wasm_runtime.ts",
    wrf_cef_runtime: "./wrf_cef_runtime.ts",
    wrf_runtime: "./wrf_runtime.ts",
    wrf_user_worker_runtime: "./wrf_user_worker_runtime.ts",
    test_suite: "./test_suite.ts",
    /* eslint-enable camelcase */
  },
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
    library: {
      name: "wrf",
      type: "umd",
    },
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: "ts-loader",
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: [".tsx", ".ts", ".js"],
  },
  devtool: "eval-cheap-module-source-map",
};
