import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import * as MatchApi from '../../api/MatchApi'
import './RegisterMatch.css'
import '../../index.css'
import Select from 'react-select'
import Button from '../../components/button/Button'

class RegisterMatch extends Component {
  users = []
  error = ''

  constructor() {
    super()
    UserApi.getUsers()
      .then((users) => {
        this.users = users.map((u) => ({
          value: u.name,
          label: u.name,
        }))
        const name = localStorage.getItem('username')
        if (name) {
          const index = this.users.findIndex((user) => user.label === name)
          const user = this.users.splice(index, 1)[0]
          this.users.unshift(user)
        }
      })
      .catch((error) => (this.error = error.message))
      .finally(() => this.setState({}))

    this.setWinner = this.setWinner.bind(this)
    this.setLoser = this.setLoser.bind(this)
    this.pressButton = this.pressButton.bind(this)
  }

  pressButton(e) {
    const token = localStorage.getItem('token')
    if (!token) {
      this.setErrorLabel('Need to be logged in to register matches')
      return
    }
    if (this.winner === undefined || this.loser === undefined)
      return this.setErrorLabel('Please select two players')

    if (this.winner === this.loser)
      return this.setErrorLabel('Players cannot be the same')

    MatchApi.registerMatch(this.winner, this.loser, token)
      .then(() => this.props.history.push('/'))
      .catch((error) => this.setErrorLabel('Something went wrong'))
  }

  setErrorLabel(text) {
    this.error = text
    this.setState({})
  }

  setWinner(e) {
    this.winner = e.value
  }

  setLoser(e) {
    this.loser = e.value
  }

  render() {
    const large = window.matchMedia('(min-width: 900px)').matches
    return (
      <div className="container">
        <h1 className="center">Register Match</h1>
        <table>
          <tbody>
            <tr>
              <th className={large ? 'large' : 'small'}>Winner</th>
              <th className={large ? 'large' : 'small'}>Loser</th>
            </tr>
            <tr>
              <th className={large ? 'large' : 'small'}>
                <Select
                  onChange={this.setWinner}
                  className="selector"
                  options={this.users}
                />
              </th>
              <th className={large ? 'large' : 'small'}>
                <Select
                  onChange={this.setLoser}
                  className="selector"
                  options={this.users} /* log */
                />
              </th>
            </tr>
          </tbody>
        </table>
        {this.error && (
          <h2
            className={this.error ? 'error' : 'success'}
            style={{ textAlign: 'center' }}
          >
            {' '}
            {this.error}{' '}
          </h2>
        )}
        <div className="button">
          <Button placeholder="Submit" callback={this.pressButton} />
        </div>
      </div>
    )
  }
}

export default RegisterMatch
