import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './ProfilePage.css'

function formatDate(ms) {
  const d = new Date(ms)
  return (
    `${d.getFullYear()}/${d.getMonth()}/${d.getDate()} ` +
    `${d.getHours()}:${d.getMinutes()}`
  )
}

class Profile extends Component {
  constructor(user) {
    super()
    Api.getUser(user.user).then((user) => {
      this.setState({ user: user })
    })
  }

  render() {
    console.log(this.state?.user)
    return (
      <div className="container">
        <h1 className="name">{this.state?.user.name}</h1>
        <h2 className="elo">{Math.trunc(this.state?.user.elo ?? 0)}</h2>
        <h2 className="history">Match history</h2>
        <table>
          <tbody>
            <tr>
              <th>Winner</th>
              <th>Loser</th>
              <th>Date</th>
            </tr>
            {this.state?.user.match_history.map((elem) => (
              <tr>
                <td>{elem.winner}</td>
                <td>{elem.loser}</td>
                <td>{formatDate(elem.epoch)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    )
  }
}

class Profiles extends Component {
  render() {
    Api.getUsers().then((x) => console.log(x))
    return (
      <div>
        <p>All profiles</p>
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
