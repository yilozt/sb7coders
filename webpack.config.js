const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WatchExternalFilesPlugin = require('webpack-watch-files-plugin');

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: 'index.html'
        }),
        new WatchExternalFilesPlugin.default({
            files: [
                path.resolve(__dirname, "./pkg"),
            ]
        }),
        // new WasmPackPlugin({
        //     crateDirectory: path.resolve(__dirname, "./pkg")
        // }),
        // Have this example work in Edge which doesn't ship `TextEncoder` or
        // `TextDecoder` at this time.
        new webpack.ProvidePlugin({
          TextDecoder: ['text-encoding', 'TextDecoder'],
          TextEncoder: ['text-encoding', 'TextEncoder']
        })
    ],
    mode: 'production',
    experiments: {
        asyncWebAssembly: true
   },
   watchOptions: {
       aggregateTimeout: 500
       // poll: 200, is not necessary as long as you remove pkg/* before building your wasm files
   }
};
