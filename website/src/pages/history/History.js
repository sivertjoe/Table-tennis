import { React, Component } from 'react'
import * as MatchApi from '../../api/MatchApi'
import './History.css'
import '../../index.css'
import { getDateTime } from '../../utils/Date'

class History extends Component {
  history = []

  constructor() {
    super()
    MatchApi.getHistory()
      .then((history) => (this.history = history))
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
  }

  render() {
    let ranking = 0
    const name = localStorage.getItem('username')
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
                <th>Date</th>
              </tr>
              {this.history.map((match) => {
                ranking += 1
                return (
                  <tr key={ranking}>
                    <td
                      style={
                        match.winner === name ? { color: 'var(--orange)' } : {}
                      }
                    >
                      <a href={'/profiles/' + match.winner}>{match.winner}</a>
                    </td>
                    <td>
                      {Math.trunc(match.winner_elo)} (
                      <div className="green">+{Math.trunc(match.elo_diff)}</div>
                      )
                    </td>
                    <td
                      style={
                        match.loser === name ? { color: 'var(--orange)' } : {}
                      }
                    >
                      <a href={'/profiles/' + match.loser}>{match.loser}</a>
                    </td>
                    <td>
                      {Math.trunc(match.loser_elo)} (
                      <div className="red">-{Math.trunc(match.elo_diff)}</div>)
                    </td>
                    <td>{getDateTime(match.epoch)}</td>
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
