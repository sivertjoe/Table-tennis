import { React, Component } from 'react'
import './Input.css'

class Input extends Component {
  constructor(args) {
    super()
    this.args = args
    this.onChange = this.onChange.bind(this)
  }

  onChange(event) {
    if (this.args.onChange) this.args.onChange(event.target.value)
  }

  render() {
    return (
      <input
        className="input"
        style={this.args.style}
        type={this.args.type ?? 'text'}
        placeholder={this.args.placeholder ?? 'Input'}
        onChange={this.onChange}
      />
    )
  }
}

export default Input
