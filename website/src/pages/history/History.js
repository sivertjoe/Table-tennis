import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './History.css'
import '../../index.css'

class History extends Component {
  constructor() {
    super()
    Api.getHistory().then((history) =>
      this.setState({
        history: history,
      }),
    )
  }
  render() {
    let ranking = 0
    return (
      <div className="container">
        <h1 className="center">Match History</h1>
        <div className="table-container">
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
                      {Math.trunc(match.winner_elo)} (
                      <div className="green">+{Math.trunc(match.elo_diff)}</div>
                      )
                    </td>
                    <td>
                      <a href={'/profiles/' + match.loser}>{match.loser}</a>
                    </td>
                    <td>
                      {Math.trunc(match.loser_elo)} (
                      <div className="red">-{Math.trunc(match.elo_diff)}</div>)
                    </td>
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

export default History
