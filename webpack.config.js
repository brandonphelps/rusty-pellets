const path = require('path');

module.exports = {
    entry: {
	app: './static/main.js'
    },
    resolve: {
	extensions: ['.ts', '.tsx', '.js']
    },
    output: {
	path: path.resolve(__dirname, 'static_gen'),
	filename: 'main_gen.js'
    },
    mode: 'development',
    module: {
	rules: [
            {
		test: /\.m?js$/,
		exclude: /(node_modules|bower_components)/,
		include: path.resolve(__dirname, 'static'),
		loader: 'babel-loader',
		options: {
		    presets: ['@babel/preset-react']
		}
            },
	    {
		test: /\.css$/i,
		use: ['style-loader', 'css-loader'],
	    }
	]
    }
};
