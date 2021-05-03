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
        this.onUpload(event.target.files[0])
        this.error = ''
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
