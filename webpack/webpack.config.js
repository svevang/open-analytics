'use strict';

var webpack = require('webpack')
var path = require('path');
var srcPath = __dirname + '/src';
var HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
    context: __dirname + "/app",
    entry: {
        app: path.join(srcPath, 'index.js'),
    },
    output: {
        path: path.join(__dirname, 'build'),
        filename: 'app.js',
    },
    module: {
        loaders: [
            {
                test: /\.js?$/,
                exclude: /(node_modules|bower_components)/,
                loader: 'babel-loader',
                presets: [ 'react']
            },
        ]
    },
    externals: {
        'react': 'React',
        'react-dom': 'ReactDom'
    },
    resolve: {
        root: srcPath,
        extensions: ['', '.js', '.jsx'],
        modulesDirectories: ['node_modules', '.']
    },
    devServer: {
        contentBase: './build',
    }
};
