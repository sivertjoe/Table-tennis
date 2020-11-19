import { React, Component } from 'react'
import * as Api from '../../api/Api'
import '../../index.css'
import './Admin.css'

class Admin extends Component {
  state = {
    isAdmin: null,
    notifications: null,
  }

  constructor() {
    super()
    const token = localStorage.getItem('token')
    if (token) {
      Api.isAdmin(token).then((isAdmin) => {
        Api.isAdmin &&
          Api.getNewUserNotification(token).then((notifications) => {
            this.setState({ isAdmin, notifications })
          })
      })
    }
  }
  render() {
    if (this.state.isAdmin) {
      const items = this.state.notifications.map((not) => (
        <tr key={not.id} id={not.id}>
          <th>{not.name}</th>
          <th>
            <button className="accept" onClick={() => click_button(not.id, 1)}>
              <span>&#10003;</span>
            </button>
          </th>
          <th>
            <button className="decline" onClick={() => click_button(not.id, 2)}>
              <span>&#10005;</span>
            </button>
          </th>
        </tr>
      ))
      return (
        <div>
          <h1 style={{ textAlign: 'center' }}>Hello Admin &#128526;</h1>
          <div className="container">
            <h2 style={{ textAlign: 'center' }}>New users:</h2>
            <div className="table-container">
              <table id="table">
                <tbody>
                  <tr>
                    <th>Name</th>
                    <th></th>
                    <th></th>
                  </tr>
                  {items}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )
    }
    return (
      <div>
        <h1 style={{ textAlign: 'center' }}>
          Shoo Shoo, Only admins can view this page &#129300;
        </h1>
      </div>
    )
  }
}

const click_button = (id, ans) => {
  const token = localStorage.getItem('token')
  Api.replyToNewUser(id, token, ans).then(() => {
    document.getElementById(id).remove()
  })
}

export default Admin
