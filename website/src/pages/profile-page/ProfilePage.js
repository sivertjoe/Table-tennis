import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import * as NotificationApi from '../../api/NotificationApi'
import './ProfilePage.css'
import '../../index.css'
import { MatchHistory } from '../../components/match-history/MatchHistory'
import { Notifications } from '../../components/notifications/Notifications'
import SearchBar from '../../components/search-bar/SearchBar'
import Button from '../../components/button/Button'
import EloGraph from '../../components/elo-graph/EloGraph'
import Badges from '../../components/badges/Badges'

class Profile extends Component {
  user = {}

  constructor(args) {
    super()
    this.changePassword = this.changePassword.bind(this)

    UserApi.getUser(args.user)
      .then((user) => (this.user = user))
      .catch((error) => (window.location.href = '/profiles'))
      .finally(() => this.setState({}))

    if (localStorage.getItem('username') === args.user)
      NotificationApi.getNotifications()
        .then((notifications) => (this.notifications = notifications))
        .catch((error) => console.warn(error.message))
        .finally(() => this.setState({}))
  }

  logout() {
    localStorage.removeItem('token')
    localStorage.removeItem('username')
    window.location.href = '/'
  }

  changePassword() {
    window.location.href = '/change-password'
  }

  render() {
    const numberOfMatches = this.user.match_history?.length
    const numberOfNotifications = this.notifications?.length
    const loggedIn = localStorage.getItem('username') === this.user.name

    return (
      <div className="container">
        <h1 className="name">{this.user.name}</h1>
        <h2 className="elo">{Math.trunc(this.user.elo ?? 0)}</h2>
        <div style={{ width: 'fit-content', margin: 'auto' }}>
          {Object.keys(this.user).length > 0 && (
            <Badges user={this.user} size="40px" />
          )}
        </div>
        {loggedIn && numberOfNotifications > 0 && (
          <div>
            <h2>
              Notifications (
              <div className="divWrapper" id="notificationCounter">
                {numberOfNotifications}
              </div>
              )
            </h2>
            <Notifications
              values={this.notifications}
              token={localStorage.getItem('token')}
            />
          </div>
        )}
        {Object.keys(this.user).length !== 0 && <EloGraph user={this.user} />}
        <h2 style={{ marginTop: '4rem' }}>Match history ({numberOfMatches})</h2>
        <MatchHistory user={this.user} />
        {loggedIn && (
          <div className="row">
            <Button
              placeholder="Change password"
              callback={this.changePassword}
            />
            <Button placeholder="Logout" callback={this.logout} />
          </div>
        )}
      </div>
    )
  }
}

class Profiles extends Component {
  users = []
  filtered = []

  constructor() {
    super()
    UserApi.getActiveUsers()
      .then((users) => {
        this.users = users
        this.filtered = users
      })
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))

    this.searchUsers = this.searchUsers.bind(this)
  }

  searchUsers = (search) => {
    this.filtered = this.users.filter((u) => u.name.includes(search))
    this.setState({})
  }

  render() {
    return (
      <div className="container">
        <h1>Profiles</h1>
        <SearchBar callback={this.searchUsers} />
        <ul className="table-container">
          {this.filtered.map((user, i) => (
            <li key={i}>
              <h2>
                <a href={'/profiles/' + user.name}>{user.name}</a>
              </h2>
            </li>
          ))}
        </ul>
      </div>
    )
  }
}

class ProfilePage extends Component {
  render() {
    return this.props.match.params.user ? (
      <Profile user={this.props.match.params.user} />
    ) : (
      <Profiles />
    )
  }
}

export default ProfilePage
