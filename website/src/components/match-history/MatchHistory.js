import React from 'react'
import '../../index.css'

function formatDate(ms) {
  const d = new Date(ms)
  return (
    `${d.getFullYear()}/${d.getMonth()}/${d.getDate()} ` +
    `${d.getHours()}:${d.getMinutes()}`
  )
}

export const MatchHistory = (user) => {
  let wins = 0
  let losses = 0
  const history = user.user.match_history?.map((elem, i) => {
    elem.winner === user.user.name ? (wins += 1) : (losses += 1)
    return (
      <tr key={i}>
        <td>{elem.winner}</td>
        <td>{elem.loser}</td>
        <td>{formatDate(elem.epoch)}</td>
      </tr>
    )
  })
  return (
    <div className="table-container">
      <table>
        <tbody>
          <tr>
            <th>Winner ({wins})</th>
            <th>Loser ({losses})</th>
            <th>Date</th>
          </tr>
          {history}
        </tbody>
      </table>
    </div>
  )
}
