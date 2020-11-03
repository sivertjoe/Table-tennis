import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './History.css'

function arrayToObject(array) {
  return array.reduce((res, cur) => {
    res[cur['name']] = Math.trunc(cur['elo'])
    return res
  }, {})
}

class History extends Component {
  constructor() {
    super()
    Promise.all([Api.getHistory(), Api.getUsers()]).then(([history, users]) =>
      this.setState({
        history: history,
        users: arrayToObject(users),
      }),
    )
  }
  render() {
    let ranking = 0
    return (
      <div className="container">
        <h1 className="center">Match History</h1>
        <table>
          <tbody>
            <tr key={ranking}>
              <th>Winner</th>
              <th>New Elo</th>
              <th>Loser</th>
              <th>New Elo</th>
            </tr>
            {this.state?.history.map((match) => {
              ranking += 1
              return (
                <tr key={ranking}>
                  <td>
                    <a href={'/profiles/' + match.winner}>{match.winner}</a>
                  </td>
                  <td>
                    {this.state?.users[match.winner]} (
                    <div className="green">
                      +{Math.trunc(match.winner_elo_diff)}
                    </div>
                    )
                  </td>
                  <td>
                    <a href={'/profiles/' + match.loser}>{match.loser}</a>
                  </td>
                  <td>
                    {this.state?.users[match.winner]} (
                    <div className="red">
                      -{Math.trunc(match.winner_elo_diff)}
                    </div>
                    )
                  </td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    )
  }
}

export default History
