const path = require('path')
const CopyWebpackPlugin = require('copy-webpack-plugin');

const paths = {
  src: path.join(__dirname, 'src'),
  dist: path.join(__dirname, 'dist'),
  public: path.join(__dirname, 'public')
}

module.exports = {
  context: paths.src,
  entry: ['./app.ts'],
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/
      }
    ]
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        {from: paths.public, to: paths.dist}
      ]
    }),
  ],
  resolve: {
    extensions: ['.tsx', '.ts', '.js', '.html']
  },
  output: {
    filename: 'app.bundle.js',
    path: paths.dist,
    publicPath: 'dist',
  },
  devtool: 'inline-source-map',
  devServer: {
    contentBase: paths.dist,
  },
}
