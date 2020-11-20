import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import * as NotificationApi from '../../api/NotificationApi'
import Select from 'react-select'
import '../../index.css'
import './Admin.css'
import Button from '../../components/button/Button'

class Admin extends Component {
  isAdmin = 0
  notifications = []
  users = []
  selectedUsers = []
  editOptions = [
    { value: 'MAKE_USER_ACTIVE', label: 'Make active' },
    { value: 'MAKE_USER_INACTIVE', label: 'Make inactive' },
    { value: 'MAKE_USER_REGULAR', label: 'Make regular' },
    { value: 'MAKE_USER_SUPERUSER', label: 'Make superuser' },
  ]
  selectedOption = null
  success = ''

  constructor() {
    super()
    const token = localStorage.getItem('token')
    if (token) {
      UserApi.isAdmin(token).then((isAdmin) => {
        if (isAdmin) {
          this.isAdmin = 1
          NotificationApi.getNewUserNotification(token)
            .then((notifications) => (this.notifications = notifications))
            .catch((error) => console.warn(error.message))
            .finally(() => this.setState({}))
        } else {
          this.isAdmin = -1
          this.setState({})
        }
      })

      UserApi.getAllUsers()
        .then((users) => {
          this.users = users.map((u) => ({
            value: u.name,
            label: u.name,
          }))
        })
        .catch((error) => console.warn(error.message))
        .finally(() => this.setState({}))
    } else this.isAdmin = -1

    this.selectUser = this.selectUser.bind(this)
    this.selectOption = this.selectOption.bind(this)
    this.editUsersButton = this.editUsersButton.bind(this)
  }

  newUserButton(id, ans) {
    const token = localStorage.getItem('token')
    NotificationApi.replyToNewUser(id, token, ans)
      .then(() => document.getElementById(id).remove())
      .catch((error) => console.warn(error.message))
  }

  selectUser(event) {
    this.selectedUsers = event
  }

  selectOption(event) {
    this.selectedOption = event
  }

  editUsersButton() {
    const users = this.selectedUsers.map((u) => u.value)
    UserApi.editUsers(users, this.selectedOption.value)
      .then(() => (this.success = 'Users successfully updated'))
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
  }

  render() {
    if (this.isAdmin === 1) {
      const items = this.notifications.map((not) => (
        <tr key={not.id} id={not.id}>
          <th>{not.name}</th>
          <th>
            <button
              className="new-user-button accept"
              onClick={() => this.newUserButton(not.id, 1)}
            >
              <span>&#10003;</span>
            </button>
          </th>
          <th>
            <button
              className="new-user-button decline"
              onClick={() => this.newUserButton(not.id, 2)}
            >
              <span>&#10005;</span>
            </button>
          </th>
        </tr>
      ))
      return (
        <div>
          <h1>Hello Admin &#128526;</h1>
          <div className="container">
            {items.length > 0 && (
              <div style={{ marginBottom: '2rem' }}>
                <h2>New users</h2>
                <div className="table-container">
                  <table id="new-users-table">
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
            )}
            <div>
              <h2>Edit users</h2>
              <h3>Select users</h3>
              <Select
                isMulti
                className="selector"
                options={this.users}
                closeMenuOnSelect={false}
                onChange={this.selectUser}
                placeholder="Select the users to edit..."
              />
              <h3>Select action</h3>
              <Select
                className="selector"
                options={this.editOptions}
                onChange={this.selectOption}
                placeholder="Select the action to perform..."
              />
              {this.success && <h2 className="success">{this.success}</h2>}
              <div className="button">
                <Button placeholder="Submit" callback={this.editUsersButton} />
              </div>
            </div>
          </div>
        </div>
      )
    } else if (this.isAdmin === -1)
      return (
        <div>
          <img alt="STOP!!!" src={'unauth.png'} />
        </div>
      )
    else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
  }
}

export default Admin
