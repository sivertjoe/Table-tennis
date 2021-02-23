import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './RegisterPage.css'
import '../../index.css'
import Button from '../../components/button/Button'

class RegisterPage extends Component {
  error = ''
  success = ''

  constructor() {
    super()
    this.onRegister = this.onRegister.bind(this)
    this.saveUsername = this.saveUsername.bind(this)
    this.savePassword = this.savePassword.bind(this)
  }

  onRegister(e) {
    e.preventDefault() // Prevents the page from refreshing
    if (!this.username) return this.setError('Username cannot be empty')
    if (!this.password) return this.setError('Password cannot be empty')

    UserApi.register(this.username, this.password)
      .then((res) => {
        this.success = 'Success!' + res
      })
      .catch((error) => (this.error = error.message))
      .then(() => this.setState({}))
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
      <form onSubmit={this.onRegister} className="container">
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
        {this.success && <h2 className="success"> {this.success} </h2>}
        <div className="button">
          <Button placeholder="Register" />
        </div>
      </form>
    )
  }
}

export default RegisterPage
