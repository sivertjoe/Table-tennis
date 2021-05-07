import React, { Component } from 'react'
import * as AdminApi from '../../../api/AdminApi'
import * as MatchApi from '../../../api/MatchApi'
import '../../../index.css'
import './EditMatches.css'
import Select from 'react-select'
import { getDateTime } from '../../../utils/Date'

class EditMatches extends Component {
  users = []
  history = []
  inFocus = -1
  success = ''
  constructor() {
    super()
    MatchApi.getEditHistory()
      .then(
        (history) =>
          (this.history = history.map((his) => {
            his.focus = false
            return his
          })),
      )
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
    AdminApi.getAllUsers()
      .then((users) => {
        this.users = users.map((u) => ({
          value: u.name,
          label: u.name,
        }))
      })
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
    this.setFocus = this.setFocus.bind(this)
    this.decline = this.decline.bind(this)
    this.submit = this.submit.bind(this)
    this.sanityCheck = this.sanityCheck.bind(this)
    this.toggleTrash = this.toggleTrash.bind(this)
  }

  setFocus(id) {
    const old = this.history.find((e) => e.id === this.inFocus)
    if (this.inFocus === id) {
      return
    }
    if (this.inFocus === -id) {
      this.inFocus = -1
      return
    }

    if (old) {
      old.focus = false
    }
    this.inFocus = id
    const elem = this.history.find((e) => e.id === id)
    if (elem) {
      elem.focus = true
    }
    this.setState({})
  }

  decline(id) {
    const old = this.history.find((e) => e.id === id)
    if (old) {
      old.focus = false
    }
    this.inFocus = -id
    this.setState({})
  }

  sanityCheck(winner, loser, epoch, id) {
    const his = this.history.find((his) => his.id === id)
    if (!his) return 'No match with that id'

    if (his.winner === winner && his.loser === loser && his.epoch === epoch)
      return 'Did not change any field'

    if (winner === loser) return 'Winner cannot be loser'

    return null
  }

  submit(id) {
    const winner = document.getElementById('winner').innerText
    const loser = document.getElementById('loser').innerText
    const epoch = new Date(document.getElementById('date').value).getTime()

    const del = document.getElementById('trash').className === 'decline button'
    if (del) {
      AdminApi.deleteMatch(id)
        .then(() => {
          const his = this.history.find((his) => his.id === id)
          let index = this.history.indexOf(his)
          if (index > -1) {
            this.success = 'Deleted match'
            this.history.splice(index, 1)
            this.setState({})
          }
        })
        .catch((error) => console.warn(error))
      return
    }

    const error = this.sanityCheck(winner, loser, epoch, id)
    if (error) {
      console.warn(error)
      return
    }

    AdminApi.editMatch(winner, loser, epoch, id)
      .then(() => {
        this.success = 'Succesfully changed match'
        const his = this.history.find((his) => his.id === id)
        his.focus = false
        this.inFocus = -id
        his.winner = winner
        his.loser = loser
        his.epoch = epoch
        this.setState({})
      })
      .catch((error) => console.warn(error))
  }

  toggleTrash(id) {
    const trash = document.getElementById('trash')
    if (trash.className === 'decline button') {
      trash.className = 'trash button'
    } else {
      trash.className = 'decline button'
    }
  }

  getDefaultDate(epoch) {
    const d = new Date(epoch)
    return `${d.getFullYear()}-${('0' + (d.getMonth() + 1)).slice(-2)}-${(
      '0' + d.getDate()
    ).slice(-2)}T${d.getHours()}:${('0' + d.getMinutes()).slice(-2)}`
  }

  render() {
    let items = this.history.map((his) => {
      if (his.focus) {
        return (
          <tr key={his.id} id={his.id} onClick={() => this.setFocus(his.id)}>
            <td>
              <Select
                id="winner"
                className="selector"
                defaultValue={{ value: his.winner, label: his.winner }}
                options={this.users}
              />
            </td>
            <td>
              <Select
                id="loser"
                className="selector"
                defaultValue={{ value: his.loser, label: his.loser }}
                options={this.users}
              />
            </td>
            <td>
              <input
                id="date"
                type="datetime-local"
                defaultValue={this.getDefaultDate(his.epoch)}
              />
            </td>
            <td>
              <button
                className="accept button"
                onClick={() => this.submit(his.id)}
              >
                <span>&#10003;</span>
              </button>
            </td>
            <td>
              <button
                className="decline button"
                onClick={() => this.decline(his.id)}
              >
                <span>&#10005;</span>
              </button>
            </td>
            <td>
              <button
                id="trash"
                className="trash button"
                onClick={() => this.toggleTrash(his.id)}
              >
                <span>&#128465;</span>
              </button>
            </td>
          </tr>
        )
      } else {
        return (
          <tr
            className="table-row"
            key={his.id}
            id={his.id}
            onClick={() => this.setFocus(his.id)}
          >
            <td>{his.winner}</td>
            <td>{his.loser}</td>
            <td>
              <p>{getDateTime(his.epoch)}</p>
            </td>
            <td></td>
            <td></td>
            <td></td>
          </tr>
        )
      }
    })
    return (
      <div className="container">
        <h2>Edit Match</h2>
        <div className="table-container">
          <table>
            <tbody>
              <tr key={0}>
                <th>Winner</th>
                <th>Loser</th>
                <th>Time</th>
                <th></th>
                <th></th>
                <th></th>
              </tr>
              {items}
            </tbody>
          </table>
        </div>
        {this.success && <h2 className="success">{this.success}</h2>}
      </div>
    )
  }
}

export default EditMatches
