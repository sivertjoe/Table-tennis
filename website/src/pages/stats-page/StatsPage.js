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
    this.onSelectSeason = this.onSelectSeason.bind(this)
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

  onSelectSeason(e) {
    this.selectedSeason = e
    this.current = { kd: [0, 0], netEloDiff: 0 }
    this.stats.forEach((stat) => {
      if (stat.season === this.selectedSeason.value)
        this.addStat(stat, 'current')
    })
    this.setState({})
  }

  initStats() {
    this.current = { kd: [0, 0], netEloDiff: 0 }
    this.rest = { kd: [0, 0], netEloDiff: 0 }
    this.stats.forEach((stat) => {
      this.addStat(stat, 'rest')
      if (stat.season === this.selectedSeason.value)
        this.addStat(stat, 'current')
    })
  }

  addStat(stat, prop) {
    if (stat.winner === this.user1) {
      this[prop].kd[0] += 1
      this[prop].netEloDiff += stat.elo_diff
    } else {
      this[prop].kd[1] += 1
      this[prop].netEloDiff -= stat.elo_diff
    }
  }

  initSeasons(stats) {
    this.seasons = stats.rest.reduce((tot, cur) => {
      if (!tot.some((x) => x.value === cur.season))
        tot.push({ label: `Season ${cur.season}`, value: cur.season })
      return tot
    }, [])
    this.seasons.unshift({
      label: 'Current season',
      value: stats.current[0]?.season,
    })
    this.selectedSeason = this.seasons[0]
  }

  getStats() {
    if (this.user1 && this.user2)
      MatchApi.getStats(this.user1, this.user2)
        .then((stats) => {
          this.stats = [...stats.current, ...stats.rest]
          this.initSeasons(stats)
          this.initStats()
        })
        .catch((error) => (this.error = error.message))
        .finally(() => this.mounted && this.setState({}))
  }

  renderStats(stats) {
    return (
      <div>
        <h3>
          K/D: <span style={{ color: 'var(--green)' }}>{stats.kd[0]}</span>/
          <span style={{ color: 'var(--red)' }}>{stats.kd[1]}</span>
        </h3>
        <h3>
          Net elo diff:{' '}
          <span
            style={{
              color: stats.netEloDiff < 0 ? 'var(--red)' : 'var(--green)',
            }}
          >
            {Math.trunc(stats.netEloDiff)}
          </span>
        </h3>
      </div>
    )
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
            <div>
              <Select
                className="season-select"
                onChange={this.onSelectSeason}
                options={this.seasons}
                value={this.selectedSeason}
              />
              {this.renderStats(this.current)}
            </div>
            <div>
              <h1 style={{ margin: '0 auto' }}>All time</h1>
              {this.renderStats(this.rest)}
            </div>
          </div>
        )}
      </div>
    )
  }
}

export default StatsPage
