import { React, Component } from 'react'
import './ImageUpload.css'

class ImageUpload extends Component {
  constructor(args) {
    super()
    this.onUpload = args.onUpload
    this._onUpload = this._onUpload.bind(this)
  }

  _onUpload(event) {
    if (this.onUpload && event.target.files[0])
      this.onUpload(event.target.files[0])
  }

  render() {
    return <input type="file" accept="image/*" onChange={this._onUpload} />
  }
}

export default ImageUpload
