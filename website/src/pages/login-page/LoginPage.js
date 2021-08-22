import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './LoginPage.css'
import '../../index.css'
import Button from '../../components/button/Button'

class LoginPage extends Component {
  error = ''
  resetPasswordMessage = false

  constructor() {
    super()
    this.onLogin = this.onLogin.bind(this)
    this.saveUsername = this.saveUsername.bind(this)
    this.savePassword = this.savePassword.bind(this)
    this.onResetPassword = this.onResetPassword.bind(this)
  }

  onLogin(e) {
    e.preventDefault() // Prevents the page from refreshing
    if (!this.username) return this.setError('Username cannot be empty')
    if (!this.password) return this.setError('Password cannot be empty')

    UserApi.login(this.username, this.password)
      .then((tokens) => {
        localStorage.setItem('token', tokens[0])
        localStorage.setItem('refreshToken', tokens[1])
        localStorage.setItem('username', this.username)
        window.location.href = '/profiles/' + this.username
      })
      .catch((error) => (this.error = error.message))
      .finally(() => {
        this.resetPasswordMessage = false
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

  onResetPassword() {
    if (!this.username)
      return this.setError('Fill in your username to reset password')

    UserApi.requestResetPassword(this.username)
      .then(() => {
        this.resetPasswordMessage = true
        this.error = ''
      })
      .catch((error) => {
        this.resetPasswordMessage = false
        this.error = error.message
      })
      .finally(() => this.setState({}))
  }

  render() {
    return (
      <form onSubmit={this.onLogin} className="container">
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
        {this.resetPasswordMessage && (
          <>
            <h2 className="message">Your request has been submitted</h2>
            <h2 className="message">
              If an admin approves, your password will be reset to '@uit'
            </h2>
          </>
        )}
        <div className="button">
          <Button placeholder="Login" />
          <p className="reset-password" onClick={this.onResetPassword}>
            Reset Password
          </p>
        </div>
      </form>
    )
  }
}

export default LoginPage
