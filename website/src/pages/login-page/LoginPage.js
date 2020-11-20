import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './LoginPage.css'
import '../../index.css'
import Button from '../../components/button/Button'

class LoginPage extends Component {
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

    UserApi.login(this.username, this.password)
      .then((token) => {
        localStorage.setItem('token', token)
        localStorage.setItem('username', this.username)
        window.location.href = '/profiles/' + this.username
      })
      .catch((error) => (this.error = error.message))
      .finally(() => this.setState({}))
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
        <h1>Login</h1>
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
          <Button placeholder="Login" callback={this.onClick} />
        </div>
        <p style={{ textAlign: 'center' }}>
          (For already registered users, default password is '@uit', change it
          asap Xdd, see profile page)
        </p>
      </div>
    )
  }
}

export default LoginPage
