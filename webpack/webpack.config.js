'use strict';

var webpack = require('webpack')
var path = require('path');
var srcPath = __dirname + '/src';
var HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
    entry: {
        app: path.join(srcPath, 'index.js'),
    },
    output: {
        path: path.join(__dirname, '../public'),
        filename: 'app.js',
    },
    module: {
        loaders: [
            {
                test: /\.jsx?$/,
                exclude: /node_modules/,
                loader: 'babel-loader',
                query: {
                    presets: [ 'react', 'es2015']
                }
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
        contentBase: '../public',
    }
};
