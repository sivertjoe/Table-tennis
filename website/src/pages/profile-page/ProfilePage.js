import { React, Component } from 'react'
import { Redirect } from 'react-router-dom'
import * as Api from '../../api/Api'
import './ProfilePage.css'
import '../../index.css'
import { MatchHistory } from '../../components/match-history/MatchHistory'
import { Notifications } from '../../components/notifications/Notifications'
import SearchBar from '../../components/search-bar/SearchBar'

class Profile extends Component {
  user = {}
  error = false

  constructor(args) {
    super()
    this.log_in_flag = true
    if (this.log_in_flag) {
      Promise.all([Api.getUser(args.user), Api.getNotifications()]).then(
        (data) => {
          this.user = data[0]
          this.notifications = data[1]
          this.setState({})
        },
      )
    } else {
      Api.getUser(args.user).then((user) => {
        this.user = user
        this.setState({})
      })
    }
  }

  render() {
    const numberOfMatches = this.user.match_history?.length
    const numberOfNotifications = this.notifications?.length

    return (
      <div className="container">
        <h1 className="name">{this.user.name}</h1>
        <h2 className="elo">{Math.trunc(this.user.elo ?? 0)}</h2>
        <h2 className="history">Match history {numberOfMatches}</h2>
        <MatchHistory user={this.user} />
        {this.log_in_flag && (
          <div>
            <h2 className="history">
              Notifications (
              <div className="divWrapper" id="notificationCounter">
                {numberOfNotifications}
              </div>
              )
            </h2>
            <Notifications
              values={this.notifications}
              token={'2501b80e-45c2-4de8-894a-ca950b7ba638'}
            />
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
    Api.getUsers().then((users) => {
      this.users = users
      this.filtered = users
      this.setState({})
    })

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
