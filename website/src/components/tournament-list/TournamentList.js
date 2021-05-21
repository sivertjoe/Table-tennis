import { React, Component } from 'react'
import '../../index.css'
import * as Api from '../../api/TournamentApi'
import './TournamentList.css'
import Button from '../../components/button/Button'

class TournamentList extends Component {
  constructor(args) {
    super()
    this.state = {
      info: '',
      color: 'green',
      tournament: args.tournament,
      users: args.tournament.data.Players,
      username: localStorage.getItem('username'),
    }

    this.join = this.join.bind(this)
    this.leave = this.leave.bind(this)
    this.removeUser = this.removeUser.bind(this)
  }

  static getDerivedStateFromProps(props, state) {
    return state.tournament.tournament.id === props.tournament.tournament.id
      ? {
          ...props,
          users: state.users,
        }
      : {
          ...props,
          info: '',
          users: props.tournament.data.Players,
        }
  }

  removeUser() {
    const index = this.state.users.indexOf(this.state.username)
    if (index < 0) {
      this.setState({ color: 'red', info: 'User not in tournament' })
    } else {
      this.state.users.splice(index, 1)
      this.setState({ color: 'green', info: 'Left tournament!' })
    }
  }

  leave(id) {
    Api.leaveTournament(id)
      .then(() => this.removeUser())
      .catch((e) => this.setState({ color: 'red', info: e.toString() }))
  }

  join(id) {
    Api.joinTournament(id)
      .then(() =>
        this.setState({
          users: [...this.state.users, this.state.username],
          color: 'green',
          info: 'Joined tournament!',
        }),
      )
      .catch((e) => this.setState({ color: 'red', info: e.toString() }))
  }

  render() {
    const id = this.state.tournament.tournament.id
    const tournamentName = this.state.tournament.tournament.name
    const numPlayers = this.state.tournament.tournament.player_count
    const joined = this.state.users.some((user) => user === this.state.username)

    const list = this.state.users.map((name, index) => (
      <tr key={name}>
        <td>{name}</td>
        <td>
          {index + 1}/{numPlayers}
        </td>
      </tr>
    ))

    return (
      <>
        <h1>'{tournamentName}' Participants:</h1>
        <div className="table-container">
          <table>
            <tbody>
              <tr key={0}>
                <th>User</th>
                <th>Player Count</th>
              </tr>
              {list}
            </tbody>
          </table>
        </div>
        <br />

        <div
          className={'center' + (joined ? '' : ' hide')}
          onClick={() => this.leave(id)}
        >
          <Button placeholder="Leave" />
        </div>
        <div
          className={'center' + (joined ? ' hide' : '')}
          onClick={() => this.join(id)}
        >
          <Button placeholder="Join" />
        </div>
        <div className="center">
          {this.state.info && (
            <h2 style={{ color: this.state.color }}>{this.state.info}</h2>
          )}
        </div>
      </>
    )
  }
}
export default TournamentList
