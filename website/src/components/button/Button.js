import { React, Component } from 'react'
import './Button.css'

class Button extends Component {
  constructor(args) {
    super()
    this.args = args
    this.onClick = this.onClick.bind(this)
  }

  onClick() {
    if (this.args.callback) this.args.callback()
  }

  render() {
    return (
      <button className="big-button" onClick={this.onClick}>
        {this.args.placeholder ?? 'Click'}
      </button>
    )
  }
}

export default Button
