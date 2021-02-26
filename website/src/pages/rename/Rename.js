import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import '../login-page/LoginPage.css'
import '../../index.css'
import Button from '../../components/button/Button'

class Rename extends Component {
  error = ''
  newUsernameMessage = false

  constructor() {
    super()
    this.saveUsername = this.saveUsername.bind(this)
    this.onNewName = this.onNewName.bind(this)
  }

  setError(val) {
    this.error = val
    this.setState({})
  }

  saveUsername(event) {
    this.username = event.target.value
  }

  onNewName(e) {
    e.preventDefault() // Prevents the page from refreshing
    if (!this.username)
      return this.setError('Fill in your new username to reset password')

    UserApi.requestNewName(this.username)
      .then(() => {
        this.newUsernameMessage = true
        this.error = ''
      })
      .catch((error) => {
        this.newUsernameMessage = false
        this.error = error.message
      })
      .finally(() => this.setState({}))
  }

  render() {
    return (
      <form onSubmit={this.onNewName} className="container">
        <h1>Rename</h1>
        <div className="inputs">
          <input
            type="text"
            placeholder="New username"
            onChange={this.saveUsername}
          />
        </div>
        {this.error && <h2 className="error"> {this.error} </h2>}
        {this.newUsernameMessage && (
          <h2 className="message">
            Your request has been submitted to an admin
          </h2>
        )}
        <div className="button">
          <Button placeholder="Submit" />
        </div>
      </form>
    )
  }
}

export default Rename
