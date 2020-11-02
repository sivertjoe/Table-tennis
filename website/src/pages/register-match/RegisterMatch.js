import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './RegisterMatch.css'
import Select from 'react-select'
import Button from '../../components/button/Button'

class RegisterMatch extends Component {
  users = []
  label = ''
  error = true

  constructor() {
    super()
    Api.getUsers().then((users) => {
      this.users = users.map((u) => ({
        value: u.name,
        label: u.name,
      }))

      this.setState({})
    })

    this.setWinner = this.setWinner.bind(this)
    this.setLoser = this.setLoser.bind(this)
    this.pressButton = this.pressButton.bind(this)
  }

  pressButton(e) {
    if (this.winner === undefined || this.loser === undefined)
      return this.setErrorLabel('Please select two players')

    if (this.winner === this.loser)
      return this.setErrorLabel('Players cannot be the same')

    const time = document.getElementById('time')
    const epoch = new Date(time.value).getTime()
    if (isNaN(epoch)) return this.setErrorLabel('Must select a time')

    Api.registerMatch(this.winner, this.loser, epoch).then(() => {
      this.setSuccessLabel('Match registered successfully')
    })
  }

  setErrorLabel(text) {
    this.label = text
    this.error = true
    return this.setState({})
  }

  setSuccessLabel(text) {
    this.label = text
    this.error = false
    return this.setState({})
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
              <th className={large ? 'large' : 'small'}>Date</th>
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
                  options={this.users}
                />
              </th>
              <th className={large ? 'large' : 'small'}>
                <input className="date" type="datetime-local" id="time"></input>
              </th>
            </tr>
          </tbody>
        </table>
        {this.label && (
          <h2 className={this.error ? 'error' : 'success'}> {this.label} </h2>
        )}
        <Button placeholder="Submit" callback={this.pressButton} />
      </div>
    )
  }
}

export default RegisterMatch
