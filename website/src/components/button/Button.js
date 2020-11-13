import { React, Component } from 'react'
import './Button.css'

class Button extends Component {
  constructor(args) {
    super()
    this.callback = args.callback
    this.onClick = this.onClick.bind(this)
    this.placeholder = args.placeholder ?? 'Click'
  }

  onClick() {
    if (this.callback) this.callback()
  }

  render() {
    return (
      <button className="big-button" onClick={this.onClick}>
        {this.placeholder}
      </button>
    )
  }
}

export default Button
