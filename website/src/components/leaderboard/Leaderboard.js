import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './Leaderboard.css'
import '../../index.css'

class Leaderboard extends Component {
  users = []

  constructor() {
    super()

    UserApi.getUsers()
      .then((users) => (this.users = users))
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
  }

  render() {
    let ranking = 0
    return (
      <div className="container">
        <h1 className="center">Leaderboard</h1>
        <div className="table-container">
          <table>
            <tbody>
              <tr key={ranking}>
                <th>Rank</th>
                <th>Name</th>
                <th>Elo</th>
              </tr>
              {this.users.map((user) => {
                ranking += 1
                return (
                  <tr key={ranking}>
                    <td>{ranking}</td>
                    <td>
                      <a href={'/profiles/' + user.name}>{user.name}</a>
                    </td>
                    <td>{Math.trunc(user.elo)}</td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>
    )
  }
}

export default Leaderboard
