import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './Leaderboard.css'
import '../../index.css'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'

class Leaderboard extends Component {
  users = []

  constructor() {
    super()

    UserApi.getUsers()
      .then((users) => (this.users = users))
      .catch((error) => console.warn(error.message))
      .finally(() => this.setState({}))
  }

  getRankBadge(rank) {
    let icon, color
    if (rank === 1) [icon, color] = ['crown', 'yellow']
    else if (rank === 2) [icon, color] = ['medal', 'silver']
    else if (rank === 3) [icon, color] = ['medal', 'orange']
    else if (rank === 4) [icon, color] = ['award', 'turquoise']
    else return

    return (
      <FontAwesomeIcon
        fixedWidth
        icon={icon}
        color={color}
        style={{
          marginLeft: '-22px',
          marginRight: '2px',
          fontSize: '16px',
        }}
      />
    )
  }

  _userBadge(icon, color, i) {
    return (
      <FontAwesomeIcon
        key={i}
        fixedWidth
        icon={icon}
        color={color}
        style={{
          fontSize: '16px',
          position: 'absolute',
          left: `${i * 5}px`,
          stroke: 'black',
          strokeWidth: '24',
        }}
      />
    )
  }

  getUserBadges(user) {
    return (
      <div style={{ position: 'relative' }}>
        <div style={{ position: 'absolute' }}>
          {Array.from(Array(3).keys()).map((i) =>
            this._userBadge('trophy', 'yellow', i),
          )}
        </div>
        <div style={{ position: 'absolute', left: `${3 * 5 + 20}px` }}>
          {Array.from(Array(2).keys()).map((i) =>
            this._userBadge('medal', 'silver', i),
          )}
        </div>
        <div style={{ position: 'absolute', left: `${5 * 5 + 40}px` }}>
          {Array.from(Array(4).keys()).map((i) =>
            this._userBadge('award', 'turquoise', i),
          )}
        </div>
      </div>
    )
  }

  render() {
    let ranking = 0
    const name = localStorage.getItem('username')
    return (
      <div className="container">
        <h1 className="center">Leaderboard</h1>
        <div className="table-container">
          <table>
            <tbody>
              <tr key={ranking}>
                <th>Rank</th>
                <th>Name</th>
                <th style={{ textAlign: 'left' }}>Badges</th>
                <th>Elo</th>
              </tr>
              {this.users.map((user) => {
                ranking += 1
                return (
                  <tr
                    key={ranking}
                    style={user.name === name ? { color: '#F8A532' } : {}}
                  >
                    <td>
                      {this.getRankBadge(ranking)}
                      {ranking}
                    </td>
                    <td>
                      <a href={'/profiles/' + user.name}>{user.name}</a>
                    </td>
                    <td style={{ textAlign: 'left', whiteSpace: 'nowrap' }}>
                      {this.getUserBadges(user)}
                    </td>
                    <td>{Math.trunc(user.elo)}</td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>
    )
  }
}

export default Leaderboard
