import { React, Component } from 'react'
import './ImageUpload.css'

class ImageUpload extends Component {
  constructor(args) {
    super()
    this.onUpload = args.onUpload
    this.maxSize = args.maxSize ?? 1000
    this._onUpload = this._onUpload.bind(this)
  }

  _onUpload(event) {
    if (this.onUpload && event.target.files[0]) {
      if (event.target.files[0].size > this.maxSize) {
        this.error = 'Image is too large: max size is ' + this.maxSize
      } else {
        const file = event.target.files[0]

        // Need to capture 'this' because 'this' inside onload is not the
        // correct 'this' :^)
        const setError = (text) => (this.error = text)
        const upload = () => this.onUpload(event.target.files[0])
        const setState = () => this.setState({})

        // Do this scheme to just get the width and height of the file (image)
        var reader = new FileReader()
        reader.readAsDataURL(file)
        reader.onload = function (e) {
          var image = new Image()
          image.src = e.target.result
          image.onload = function () {
            const diff = Math.abs((this.width - this.height) / this.height)
            if (this.width > 512 || this.height > 512) {
              setError(
                'Image dimensions are too big, yours: ' +
                  this.width +
                  'x' +
                  this.height +
                  ',  max is 512x512',
              )
            } else if (diff > 0.15) {
              setError(
                'Image dimension difference too big, yours: ' +
                  diff +
                  ', max: 0.15',
              )
            } else {
              upload()
              setError('')
            }
            setState()
          }
        }
      }
      this.setState({})
    }
  }

  render() {
    return (
      <>
        <input
          type="file"
          accept="image/png"
          onChange={this._onUpload}
          maxLength={this.maxSize}
        />
        {this.error && <p style={{ color: 'red' }}>{this.error}</p>}
      </>
    )
  }
}

export default ImageUpload
