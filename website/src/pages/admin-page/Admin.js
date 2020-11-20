import { React, Component } from 'react'
import * as Api from '../../api/Api'
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
      Api.isAdmin(token).then((isAdmin) => {
        if (isAdmin)
          Api.getNewUserNotification(token).then((notifications) => {
            this.isAdmin = 1
            this.notifications = notifications
            this.setState({})
          })
        else {
          this.isAdmin = -1
          this.setState({})
        }
      })

      Api.getAllUsers().then((users) => {
        this.users = users.map((u) => ({
          value: u.name,
          label: u.name,
        }))
        this.setState({})
      })
    } else this.isAdmin = -1

    this.selectUser = this.selectUser.bind(this)
    this.selectOption = this.selectOption.bind(this)
    this.editUsersButton = this.editUsersButton.bind(this)
  }

  newUserButton(id, ans) {
    const token = localStorage.getItem('token')
    Api.replyToNewUser(id, token, ans).then(() => {
      document.getElementById(id).remove()
    })
  }

  selectUser(event) {
    this.selectedUsers = event
  }

  selectOption(event) {
    this.selectedOption = event
  }

  editUsersButton() {
    console.log(this.selectedUsers)
    console.log(this.selectedOption)
    const users = this.selectedUsers.map((u) => u.value)
    Api.editUsers(users, this.selectedOption.value).then((res) => {
      if (res.status === 200) {
        this.success = 'Users successfully updated'
        this.setState({})
      }
    })
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
              {this.success && <h2 class="success">{this.success}</h2>}
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
          <img alt='STOP!!!' src={'unauth.png'} />
        </div>
      )
    else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
  }
}

export default Admin
