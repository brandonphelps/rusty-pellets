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
		test: /\.(css)$/,
		use: [
		    {
			loader: 'style-loader'
		    },
		    {
			loader: 'css-loader'
		    },
		    {
			loader: 'postcss-loader',
			options: {
			    postcssOptions: {
				plugins: () => [
				    require('autoprefixer')
				]
			    }
			}
		    },
		    {
			loader: 'sass-loader'
		    }
		]
	    }
	]
    }
};
