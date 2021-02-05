import { React, Component } from 'react'
import * as UserApi from '../../api/UserApi'
import * as MatchApi from '../../api/MatchApi'
import './StatsPage.css'
import '../../index.css'
import Select from 'react-select'

class StatsPage extends Component {
  constructor() {
    super()
    this.mounted = true

    const params = new URLSearchParams(window.location.search)
    this.user1 = params.get('user1')
    this.user2 = params.get('user2')

    UserApi.getActiveUsers()
      .then((users) => {
        this.users = users.map((u) => ({
          value: u.name,
          label: u.name,
        }))
        const name = localStorage.getItem('username')
        if (name) {
          const index = this.users.findIndex((user) => user.label === name)
          const user = this.users.splice(index, 1)[0]
          this.users.unshift(user)
        }
      })
      .catch((error) => (this.error = error.message))
      .finally(() => this.setState({}))

    this.setUser1 = this.setUser1.bind(this)
    this.setUser2 = this.setUser2.bind(this)
    this.updateUrl = this.updateUrl.bind(this)
    this.getStats = this.getStats.bind(this)
    this.getStats()
  }

  componentDidMount() {
    this.mounted = true
  }

  componentWillUnmount() {
    this.mounted = false
  }

  updateUrl() {
    if (this.user1 && this.user2)
      this.props.history.push({
        search: '?user1=' + this.user1 + '&user2=' + this.user2,
      })
    else if (this.user1)
      this.props.history.push({ search: '?user1=' + this.user1 })
    else if (this.user2)
      this.props.history.push({ search: '?user2=' + this.user2 })

    this.getStats()
  }

  setUser1(e) {
    this.user1 = e.value
    this.updateUrl()
  }

  setUser2(e) {
    this.user2 = e.value
    this.updateUrl()
  }

  getStats() {
    if (this.user1 && this.user2)
      MatchApi.getStats(this.user1, this.user2)
        .then((stats) => {
          this.stats = stats
          this.kd = [0, 0]
          this.netEloDiff = 0
          this.totalEloDiff = 0
          stats.current.forEach((stat) => {
            this.totalEloDiff += stat.elo_diff
            if (stat.winner === this.user1) {
              this.kd[0] += 1
              this.netEloDiff += stat.elo_diff
            } else {
              this.kd[1] += 1
              this.netEloDiff -= stat.elo_diff
            }
          })
        })
        .catch((error) => (this.error = error.message))
        .finally(() => this.mounted && this.setState({}))
  }

  render() {
    return (
      <div className="container">
        <h1>Select users to compare</h1>
        <table>
          <tbody>
            <tr>
              <th>User 1</th>
              <th>User 2</th>
            </tr>
            <tr>
              <th>
                <Select
                  onChange={this.setUser1}
                  className="selector"
                  options={this.users}
                  value={{ label: this.user1, value: this.user1 }}
                />
              </th>
              <th>
                <Select
                  onChange={this.setUser2}
                  className="selector"
                  options={this.users}
                  value={{ label: this.user2, value: this.user2 }}
                />
              </th>
            </tr>
          </tbody>
        </table>
        {this.error && <h2 className="error"> {this.error} </h2>}
        {this.stats && (
          <div className="stats">
            <h1>
              K/D: {this.kd[0]}/{this.kd[1]}
            </h1>
            <h1>Total elo diff: {Math.trunc(this.totalEloDiff)}</h1>
            <h1>Net elo diff: {Math.trunc(this.netEloDiff)}</h1>
          </div>
        )}
      </div>
    )
  }
}

export default StatsPage
