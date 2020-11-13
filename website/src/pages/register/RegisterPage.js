import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './RegisterPage.css'
import '../../index.css'
import Button from '../../components/button/Button'

class RegisterPage extends Component {
  error = ''

  constructor() {
    super()
    this.onClick = this.onClick.bind(this)
    this.saveUsername = this.saveUsername.bind(this)
    this.savePassword = this.savePassword.bind(this)
  }

  onClick() {
    if (!this.username) return this.setError('Username cannot be empty')
    if (!this.password) return this.setError('Password cannot be empty')

    Api.register(this.username, this.password).then((res) => {
      if (res.status === 200)
        return res.json().then((token) => {
          localStorage.setItem('token', token)
          localStorage.setItem('username', this.username)
          window.location.href = '/profiles/' + this.username
        })

      if (res.status === 409) this.error = 'This username is unavailable'
      else this.error = 'Something went wrong'
      this.setState({})
    })
  }

  setError(val) {
    this.error = val
    this.setState({})
  }

  saveUsername(event) {
    this.username = event.target.value
  }

  savePassword(event) {
    this.password = event.target.value
  }

  render() {
    return (
      <div className="container">
        <h1>Create a new user</h1>
        <div className="inputs">
          <input
            type="text"
            placeholder="Username"
            onChange={this.saveUsername}
          />
          <br />
          <br />
          <input
            type="password"
            placeholder="Password"
            onChange={this.savePassword}
          />
        </div>
        {this.error && <h2 className="error"> {this.error} </h2>}
        <div className="button">
          <Button placeholder="Register" callback={this.onClick} />
        </div>
        <p>
          (Note: Website is currently NOT running on https, so maybe not use
          your normal password, just in case ¯\_(ツ)_/¯)
        </p>
        <p style={{ fontSize: 'small' }}>
          (Ofcourse, you should not use the same password twice anyways ',:^))
        </p>
      </div>
    )
  }
}

export default RegisterPage
