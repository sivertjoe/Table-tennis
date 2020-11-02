import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './ProfilePage.css'
import SearchBar from '../../components/search-bar/SearchBar'

function formatDate(ms) {
  const d = new Date(ms)
  return (
    `${d.getFullYear()}/${d.getMonth()}/${d.getDate()} ` +
    `${d.getHours()}:${d.getMinutes()}`
  )
}

class Profile extends Component {
  constructor(args) {
    super()
    Api.getUser(args.user).then((user) => {
      this.setState({ user: user })
    })
  }

  render() {
    let wins = 0
    let losses = 0
    const history = this.state?.user.match_history.map((elem, i) => {
      elem.winner === this.state?.user.name ? (wins += 1) : (losses += 1)
      return (
        <tr key={i}>
          <td>{elem.winner}</td>
          <td>{elem.loser}</td>
          <td>{formatDate(elem.epoch)}</td>
        </tr>
      )
    })

    return (
      <div className="container">
        <h1 className="name">{this.state?.user.name}</h1>
        <h2 className="elo">{Math.trunc(this.state?.user.elo ?? 0)}</h2>
        <h2 className="history">Match history {wins + losses}</h2>
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
        <ul>
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
