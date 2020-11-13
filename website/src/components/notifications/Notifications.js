import React from 'react'
import * as Api from '../../api/Api'
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
        <button onClick={() => click_button(vals.id, 1)}>
          <span>&#10003;</span>
        </button>
      </th>
      <th>
        <button onClick={() => click_button(vals.id, 2)}>
          <span>&#10005;</span>
        </button>
      </th>
    </tr>
  )
}

const click_button = (id, ans) => {
  const token = localStorage.getItem('token')
  Api.replyToMatch(id, token, ans).then(() => {
    document.getElementById(id).remove()
    document.getElementById('notificationCounter').innerHTML -= 1
  })
}
