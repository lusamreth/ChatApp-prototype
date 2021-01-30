const path = require('path');

module.exports = {
  entry: {
    app: ['@babel/polyfill', './js/index.js']
  },
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'app.bundle.js'
  },
  module: {
    rules: [
      {
        test: /\.js?$/,
        exclude: /node_modules/,
        loader: 'babel-loader',
        options: {
          presets: ['@babel/preset-env'],
          plugins: ["@babel/plugin-transform-named-capturing-groups-regex","@babel/plugin-proposal-unicode-property-regex"],
          cacheDirectory:true
        }
      }
    ]
  },
  watch:true
};
