import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './Leaderboard.css'

class Leaderboard extends Component {
  constructor() {
    super()
    Api.getUsers().then((users) => this.setState({ users: users }))
  }
  render() {
    let ranking = 0
    return (
      <div className="container">
        <h1 className="center">Leaderboard</h1>
        <table>
          <tbody>
            <tr key={ranking}>
              <th>Rank</th>
              <th>Name</th>
              <th>Elo</th>
            </tr>
            {this.state?.users.map((user) => {
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
    )
  }
}

export default Leaderboard
