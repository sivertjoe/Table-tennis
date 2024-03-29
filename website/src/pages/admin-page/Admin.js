import { React, Component } from 'react'
import * as AdminApi from '../../api/AdminApi'
import * as NotificationApi from '../../api/NotificationApi'
import Select from 'react-select'
import '../../index.css'
import './Admin.css'
import Button from '../../components/button/Button'

class Admin extends Component {
  newUserNotifications = []
  resetPasswordNotifications = []
  users = []
  selectedUsers = []
  editOptions = [
    { value: 'MAKE_USER_ACTIVE', label: 'Make active' },
    { value: 'MAKE_USER_INACTIVE', label: 'Make inactive' },
    { value: 'MAKE_USER_SOFT_INACTIVE', label: 'Make soft inactive' },
    { value: 'MAKE_USER_REGULAR', label: 'Make regular' },
    { value: 'MAKE_USER_SUPERUSER', label: 'Make superuser' },
  ]
  selectedOption = null
  success = ''

  constructor() {
    super()
    NotificationApi.getAdminNotifications()
      .then((res) => {
        this.newUserNotifications = res.new_users
        this.resetPasswordNotifications = res.reset_password
      })
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))

    AdminApi.getAllUsers()
      .then((users) => {
        this.users = users.map((u) => ({
          value: u.name,
          label: u.name,
        }))
      })
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))

    this.selectUser = this.selectUser.bind(this)
    this.selectOption = this.selectOption.bind(this)
    this.editUsersButton = this.editUsersButton.bind(this)
    this.rollBack = this.rollBack.bind(this)
  }

  newUserButton(id, ans) {
    const token = localStorage.getItem('token')
    NotificationApi.replyToNewUser(id, token, ans)
      .then(() => document.getElementById('New users' + id).remove())
      .catch((error) => console.warn(error.message))
  }

  resetPasswordButton(id, ans) {
    const token = localStorage.getItem('token')
    NotificationApi.replyToResetPassword(id, token, ans)
      .then(() => document.getElementById('Password resets' + id).remove())
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
    AdminApi.editUsers(users, this.selectedOption.value)
      .then(() => (this.success = 'Users successfully updated'))
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
  }

  rollBack() {
    AdminApi.rollBack()
      .then(() => (this.success = 'Rolled back successfully'))
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
  }

  notification_table(notifications, title, button) {
    return (
      notifications.length > 0 && (
        <div style={{ marginBottom: '2rem' }}>
          <h2>{title}</h2>
          <div className="table-container">
            <table id="new-users-table">
              <tbody>
                <tr>
                  <th>Name</th>
                  <th></th>
                  <th></th>
                </tr>
                {notifications.map((not) => (
                  <tr key={not.id} id={title + not.id}>
                    <th>{not.name}</th>
                    <th>
                      <button
                        className="notification-button accept"
                        onClick={() => button(not.id, 1)}
                      >
                        <span>&#10003;</span>
                      </button>
                    </th>
                    <th>
                      <button
                        className="notification-button decline"
                        onClick={() => button(not.id, 2)}
                      >
                        <span>&#10005;</span>
                      </button>
                    </th>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )
    )
  }

  render() {
    return (
      <div>
        <h1>Hello Admin &#128526;</h1>
        <div className="container">
          {this.notification_table(
            this.newUserNotifications,
            'New users',
            this.newUserButton,
          )}
          {this.notification_table(
            this.resetPasswordNotifications,
            'Password resets',
            this.resetPasswordButton,
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
          <Button placeholder="Rollback" callback={this.rollBack} />
        </div>
        <div className="container">
          <h2>Other Admin pages</h2>
          <a href="/admin/edit-match">
            <Button placeholder="Edit Match" />
          </a>
          <a className="adminButton" href="/admin/edit-season">
            <Button placeholder="Edit Season" />
          </a>
          <a className="adminButton" href="/admin/terminal">
            <Button placeholder="Terminal" />
          </a>
        </div>
      </div>
    )
  }
}

export default Admin
