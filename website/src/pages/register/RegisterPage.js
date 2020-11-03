import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './RegisterPage.css'
import '../../index.css'
import SearchBar from '../../components/search-bar/SearchBar'
import Button from '../../components/button/Button'

class RegisterPage extends Component {
  error = ''

  constructor() {
    super()
    this.onClick = this.onClick.bind(this)
    this.saveInput = this.saveInput.bind(this)
  }

  onClick() {
    if (this.input)
      Api.register(this.input).then((res) => {
        if (res) this.props.history.push('/profiles/' + this.input)
        this.error = 'This username is unavailable'
        this.setState({})
      })
    else {
      this.error = 'Username cannot be empty'
      this.setState({})
    }
  }

  saveInput(input) {
    this.input = input
  }

  render() {
    return (
      <div className="container">
        <h1>Create a new user</h1>
        <SearchBar placeholder="Username" callback={this.saveInput} />
        {this.error && <h2 className="error"> {this.error} </h2>}
        <Button placeholder="Register" callback={this.onClick} />
      </div>
    )
  }
}

export default RegisterPage
