import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './RegisterMatch.css'
import Select from 'react-select'
import PropTypes from 'prop-types'

class RegisterMatch extends Component {
  constructor() {
    super()
    this.state = {
      users: [],
      winner: undefined,
      loser: undefined,
    }
    Api.getUsers().then((users) =>
      this.setState({
        users: users.map((u) => ({
          value: u.name,
          label: u.name,
        })),
      }),
    )

    this.handleChangeWinner = (selectedOption) => {
      this.setState({ winner: selectedOption.value })
    }
    this.handleChangeLoser = (selectedOption) => {
      this.setState({ loser: selectedOption.value })
    }

    this.pressButton = () => {
      let label = document.getElementById('infoLabel')
      label.innerHTML = ''
      console.log(this.state?.winner) //????
      const dummyvalue1 = 'Sivert'
      const dummyvalue2 = 'Sivert'
      const time = document.getElementById('time')
      const epoch = new Date(time.value).getTime()

      if (epoch === NaN) {
        label.style = 'color: rgb(255, 0, 0);'
        label.innerHTML = 'Must select a time'
        return
      }

      if (dummyvalue1 == dummyvalue2) {
        label.style = 'color: rgb(255, 0, 0);'
        label.innerHTML = "Can't be the same person"
        return
      }
      if (dummyvalue1 == undefined || dummyvalue2 == undefined) {
        label.style = 'color: rgb(255, 0, 0);'
        label.innerHTML = 'Select two people'
        return
      }

      Api.registerMatch(dummyvalue1, dummyvalue2, epoch).then(() => {
        label.style = 'color: rgb(0, 255, 0);'
        label.innerHTML = 'Added match'
      })
    }

    this.customStyles = {
      option: (provided) => ({
        ...provided,
        color: 'black',
      }),
      control: (provided) => ({
        ...provided,
        color: 'black',
      }),
      singleValue: (provided) => ({
        ...provided,
        color: 'black',
      }),
    }
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
                  onChange={this.handleChange}
                  className="react-select-container"
                  styles={this.customStyles}
                  options={this.state?.users}
                />
              </th>
              <th>
                <Select
                  onChange={this.onChangeLoser}
                  className="react-select-container"
                  styles={this.customStyles}
                  options={this.state?.users}
                />
              </th>
              <th>
                <input type="datetime-local" id="time"></input>
              </th>
            </tr>
          </tbody>
        </table>
        <div>
          <button onClick={this.pressButton}>Register Match</button>
          <label id="infoLabel"></label>
        </div>
      </div>
    )
  }
}

export default RegisterMatch
