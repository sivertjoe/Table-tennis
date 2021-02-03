import React, { Component } from 'react'
import * as AdminApi from '../../../api/AdminApi'
import * as MatchApi from '../../../api/MatchApi.js'
import Button from '../../../components/button/Button'
import './EditSeason.css'
import '../../../index.css'

class EditSeason extends Component {
  seasonLength = -1
  successLabel = ''
  constructor() {
    super()
    const token = localStorage.getItem('token')
    if (token) {
      AdminApi.isAdmin(token)
        .then((isAdmin) => {
          if (isAdmin) {
            this.isAdmin = 1
            MatchApi.getSeasonLength()
              .then((len) => (this.seasonLength = len))
              .catch((error) => console.warn(error.message))
              .finally(() => this.setState({}))
          } else {
            this.isAdmin = -1
          }
        })
        .catch((error) => console.warn(error.message))
        .finally(() => this.setState({}))
    } else this.isAdmin = -1
    this.isAdmin = 1

    this.incNumber = this.incNumber.bind(this)
    this.decNumber = this.decNumber.bind(this)
    this.submit = this.submit.bind(this)
    this.stop = this.stop.bind(this)
    this.start = this.start.bind(this)
  }

  incNumber() {
    this.seasonLength += 1
    this.setState({})
  }

  decNumber() {
    if (this.seasonLength > 1) {
      this.seasonLength -= 1
      this.setState({})
    }
  }

  stop() {
    AdminApi.stopSeason()
      .then(() => {
        this.successLabel = 'Stopped the season'
        this.setState({})
      })
      .catch((error) => console.warn(error))
    this.setState({})
  }

  start() {
    AdminApi.startSeason()
      .then(() => {
        this.successLabel = 'Started the season'
        this.setState({})
      })
      .catch((error) => console.warn(error))
    this.setState({})
  }

  submit() {
    AdminApi.setSeasonLength(this.seasonLength)
      .then(() => {
        this.successLabel = 'Succesfully changed season length'
        this.setState({})
      })
      .catch((error) => console.warn(error))
    this.setState({})
  }

  render() {
    if (this.isAdmin === 1) {
      return (
        <div className="container">
          <div className="center">
            <h1>Edit Season</h1>
            <label className="leftSpace">
              Current season length: {this.seasonLength}
            </label>
            <br />
            <button onClick={() => this.decNumber()}>-</button>
            <button onClick={() => this.incNumber()}>+</button>
            <br />
            <br />
            <div className="button">
              <Button placeholder="Submit" callback={() => this.submit()} />
            </div>
            <br />
            <Button placeholder="Stop Season" callback={() => this.stop()} />
            <Button placeholder="Start Season" callback={() => this.start()} />
            <br />
            <br />
            <label className="success">{this.successLabel}</label>
          </div>
        </div>
      )
    } else if (this.isAdmin === -1)
      return (
        <div>
          <img className="arnold" alt="STOP!!!" src={'../unauth.png'} />
        </div>
      )
    else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
  }
}

export default EditSeason
