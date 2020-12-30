import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import './Leaderboard.css'
import '../../index.css'
import Badges from '../badges/Badges'
import images from '../../assets/images'

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
    let badge
    if (rank === 1) badge = images['first_place.png']
    else if (rank === 2) badge = images['second_place.png']
    else if (rank === 3) badge = images['third_place.png']
    else return

    return <img alt="Badge" src={badge} className="rank-badge" />
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
                      <Badges user={user} />
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
