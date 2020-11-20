import React from 'react'
import * as MatchApi from '../../api/MatchApi'
import '../../index.css'
import './Notifications.css'

function formatDate(ms) {
  const d = new Date(ms)
  return (
    `${d.getFullYear()}/${d.getMonth()}/${d.getDate()} ` +
    `${d.getHours()}:${d.getMinutes()}`
  )
}

export const Notifications = (notifications, token) => {
  const items = notifications.values?.map((not) => {
    return <NotificationItem key={not.id} values={not} />
  })
  return (
    <div className="table-container">
      <table id="table">
        <tbody className="tbody">
          <tr>
            <th>Winner</th>
            <th>Loser</th>
            <th>Date</th>
            <th></th>
            <th></th>
          </tr>
          {items}
        </tbody>
      </table>
    </div>
  )
}

const NotificationItem = (values) => {
  const vals = values.values
  return (
    <tr id={vals.id} className="tr">
      <th>{vals.winner}</th>
      <th>{vals.loser}</th>
      <th>{formatDate(vals.epoch)}</th>
      <th>
        <button onClick={() => clickButton(vals.id, 1)}>
          <span>&#10003;</span>
        </button>
      </th>
      <th>
        <button onClick={() => clickButton(vals.id, 2)}>
          <span>&#10005;</span>
        </button>
      </th>
    </tr>
  )
}

const clickButton = (id, ans) => {
  const token = localStorage.getItem('token')
  MatchApi.replyToMatch(id, token, ans)
    .then(() => {
      document.getElementById(id).remove()
      document.getElementById('notificationCounter').innerHTML -= 1
    })
    .catch((err) => console.warn(err.message))
}
