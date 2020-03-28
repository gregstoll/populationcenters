const path = require('path')

const paths = {
  src: path.join(__dirname, 'src'),
  dist: path.join(__dirname, 'dist'),
  data: path.join(__dirname, 'data')
}

module.exports = {
  context: paths.src,
  entry: ['./app.js'],
  output: {
    filename: 'app.bundle.js',
    path: path.resolve(paths.dist,"dist"),
    publicPath: 'dist',
  },
  devtool: 'inline-source-map',
}