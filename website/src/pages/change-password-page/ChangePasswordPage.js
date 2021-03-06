import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './ChangePasswordPage.css'
import '../../index.css'
import Button from '../../components/button/Button'

class ChangePasswordPage extends Component {
  error = ''

  constructor() {
    super()
    if (!localStorage.getItem('token')) window.location.href = '/'

    this.onResetPassword = this.onResetPassword.bind(this)
    this.saveOldPassword = this.saveOldPassword.bind(this)
    this.savePassword = this.savePassword.bind(this)
    this.saveConfirmPassword = this.saveConfirmPassword.bind(this)
  }

  onResetPassword(e) {
    e.preventDefault() // Prevents the page from refreshing
    if (!this.password) return this.setError('Old password cannot be empty')
    if (!this.newPassword) return this.setError('New password cannot be empty')
    if (!this.confirmPassword)
      return this.setError('Confirm password cannot be empty')
    if (this.newPassword !== this.confirmPassword)
      return this.setError('Passwords do not match')

    const username = localStorage.getItem('username')
    UserApi.changePassword(username, this.password, this.newPassword)
      .then((res) => (window.location.href = '/profiles/' + username))
      .catch((error) => (this.error = error.message))
      .finally(() => this.setState({}))
  }

  setError(val) {
    this.error = val
    this.setState({})
  }

  saveOldPassword(event) {
    this.password = event.target.value
  }

  savePassword(event) {
    this.newPassword = event.target.value
  }

  saveConfirmPassword(event) {
    this.confirmPassword = event.target.value
  }

  render() {
    return (
      <form onSubmit={this.onResetPassword} className="container">
        <h1>Change Password</h1>
        <div className="inputs">
          <input
            type="password"
            placeholder="Old Password"
            onChange={this.saveOldPassword}
          />
          <br />
          <br />
          <input
            type="password"
            placeholder="New Password"
            onChange={this.savePassword}
          />
          <br />
          <br />
          <input
            type="password"
            placeholder="Confirm Password"
            onChange={this.saveConfirmPassword}
          />
        </div>
        {this.error && <h2 className="error"> {this.error} </h2>}
        <div className="button">
          <Button placeholder="ChangePassword" />
        </div>
      </form>
    )
  }
}

export default ChangePasswordPage
