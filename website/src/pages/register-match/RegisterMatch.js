import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './RegisterMatch.css'
import '../../index.css'
import Select from 'react-select'
import Button from '../../components/button/Button'

class RegisterMatch extends Component {
  users = []
  error = ''

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
      this.props.history.push('/')
    })
  }

  setErrorLabel(text) {
    this.error = text
    return this.setState({})
  }

  setWinner(e) {
    this.winner = e.value
  }

  setLoser(e) {
    this.loser = e.value
  }

  render() {
    const d = new Date()
    const currentDate = `${d.getFullYear()}-${('0' + (d.getMonth() + 1)).slice(
      -2,
    )}-${('0' + d.getDate()).slice(-2)}T${('0' + d.getHours()).slice(-2)}:${(
      '0' + d.getMinutes()
    ).slice(-2)}`

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
                <input
                  id="time"
                  className="date"
                  type="datetime-local"
                  defaultValue={currentDate}
                ></input>
              </th>
            </tr>
          </tbody>
        </table>
        {this.error && (
          <h2 className={this.error ? 'error' : 'success'}> {this.error} </h2>
        )}
        <Button placeholder="Submit" callback={this.pressButton} />
      </div>
    )
  }
}

export default RegisterMatch
