import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './RegisterMatch.css'
import Select from 'react-select'
import PropTypes from 'prop-types'
import Button from '../../components/button/Button'

class RegisterMatch extends Component {
  users = []

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
    let label = document.getElementById('infoLabel')
    label.innerHTML = ''

    const time = document.getElementById('time')
    const epoch = new Date(time.value).getTime()

    if (isNaN(epoch)) {
      label.style = 'color: rgb(255, 0, 0);'
      label.innerHTML = 'Must select a time'
      return
    }

    if (this.winner === this.loser) {
      this.setErrorLabel(label, "Can't be the same person")
      return
    }
    if (this.winner === undefined || this.loser === undefined) {
      this.setErrorLabel(label, 'Please select two people')
      return
    }

    Api.registerMatch(this.winner, this.loser, epoch).then(() => {
      this.setSuccessLabel(label, 'Added match')
    })
  }

  setErrorLabel(label, text) {
    label.style = 'color: rgb(255, 0, 0);'
    label.innerHTML = text
  }
  setSuccessLabel(label, text) {
    label.style = 'color: rgb(0, 255, 0);'
    label.innerHTML = text
  }

  setWinner(e) {
    this.winner = e.value
  }
  setLoser(e) {
    this.loser = e.value
  }

  render() {
    return (
      <div className="container">
        <h1 className="center">Register Match</h1>
        <table>
          <tbody>
            <tr>
              <th>Winner</th>
              <th>Loser</th>
              <th>Date</th>
            </tr>
            <tr>
              <th>
                <Select
                  onChange={this.setWinner}
                  className="selector"
                  options={this.users}
                />
              </th>
              <th>
                <Select
                  onChange={this.setLoser}
                  className="selector"
                  options={this.users}
                />
              </th>
              <th>
                <input className="date" type="datetime-local" id="time"></input>
              </th>
            </tr>
          </tbody>
        </table>
        <Button placeholder="Register match" callback={this.pressButton} />
        <br />
        <label id="infoLabel"></label>
      </div>
    )
  }
}

export default RegisterMatch
