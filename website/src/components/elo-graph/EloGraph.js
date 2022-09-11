import { React, Component } from 'react'
import { ResponsiveLine } from '@nivo/line'
import Select from 'react-select'
import './EloGraph.css'
import { Defs } from '@nivo/core'
import { area, curveMonotoneX } from 'd3-shape'
import { getShortDate, getPreviousDate } from '../../utils/Date'
import * as UserApi from '../../api/UserApi'
import * as MatchApi from '../../api/MatchApi'

function matchElo(match, name) {
  return match.winner === name ? match.winner_elo : match.loser_elo
}
class GraphToolTip extends Component {
  render() {
    let player = this.props.props
    return (
      <div key={player.label} className="match">
        <div
          style={{
            height: '15px',
            width: '15px',
            backgroundColor: player.serieColor,
            display: 'inline-block',
            borderRadius: '50%',
          }}
        ></div>
        <div className="justify-center">{' ' + player.serieId}</div>
        <div className="justify-center">{' ' + player.data.y}</div>

        <div className="justify-center">
          {player.data.elo_diff === undefined ? (
            ''
          ) : (
            <span style={{ fontSize: 'inherit' }}>
              {' ('}
              {player.data.winner === player.serieId ? (
                <span className="winner">{'+' + player.data.elo_diff}</span>
              ) : (
                <span className="loser">{'−' + player.data.elo_diff}</span>
              )}
              {')'}
            </span>
          )}
        </div>
      </div>
    )
  }
}

class EloGraph extends Component {
  x = 0
  userList = []
  seasons = []
  selectedSeason = { value: '', label: 'latest' }

  periods = [
    {
      value: 'today',
      label: 'Today',
      ticks: 'every 1 day',
      date: getPreviousDate(0),
    },
    {
      value: 'week',
      label: 'Last Week',
      ticks: 'every 1 day',
      date: getPreviousDate(7),
    },
    {
      value: 'month',
      label: 'This Month',
      ticks: 'every 2 day',
      date: getPreviousDate(30),
    },
    {
      value: 'season',
      label: 'This Season',
      ticks: 'every 2 day',
      date: getPreviousDate(30),
    },
  ]
  selectedPeriod = this.periods[1]

  constructor(args) {
    super()

    this.changePeriod = this.changePeriod.bind(this)
    this.genLayers = this.genLayers.bind(this)
    this.addUser = this.addUser.bind(this)
    this.changeSeason = this.changeSeason.bind(this)
    UserApi.getMultipleUsers(args.users)
      .then((users) => (this.users = users))
      .catch((error) => (this.error = error.message))
      .finally(() => this.setState({}))

    UserApi.getActiveUsers()
      .then((users) => (this.userList = users))
      .catch((err) => console.warn(err))
      .finally(() => {
        this.setState({})
      })

    MatchApi.getSeasonStart()
      .then((date) => {
        const start = new Date(date)
        this.periods[2].date = start
      })
      .catch((_e) => {
        const start = new Date(0)
        this.periods[2].date = start
      })

    MatchApi.getSeasons()
      .then((seasons) => (this.seasons = seasons))
      .catch((_e) => console.error('Could not fetch seasonss'))
      .finally(() => {
        this.setState({})
      })
  }
  changeSeason(e) {
    this.selectedSeason = e
    UserApi.getMultipleUsers(
      this.users.map((user) => user.name),
      e.value,
    )
      .then((users) => {
        this.users = users
      })
      .catch((err) => console.warn(err))
      .finally(() => this.setState({}))
  }

  changePeriod(e) {
    this.selectedPeriod = e
    this.setState({})
  }

  addUser(e) {
    if (e == null) {
      this.users = []
      this.setState({})
    } else {
      UserApi.getMultipleUsers(
        e.map((user) => user.value),
        this.selectedSeason.value,
      )
        .then((users) => {
          this.users = users
        })
        .catch((err) => console.warn(err))
        .finally(() => this.setState({}))
    }
  }

  orderMatches(users) {
    let items = {}
    users.forEach((user) => {
      if (this.selectedPeriod.label === 'This Season') {
        items[user.name] = []
        user._match_history = user.match_history
      } else {
        items[user.name] = []
        user._match_history = user.match_history.filter(
          (match) => match.epoch >= this.selectedPeriod.date.getTime(),
        )
      }
    })

    let min_x = 0
    let day = ''
    let x = -1
    let prev = -1
    let col = true
    let user = undefined

    while (users.some((__user, _i) => __user._match_history.length > 0)) {
      let current_first = Infinity

      for (let i = 0; i < users.length; i++) {
        const _user = users[i]
        if (_user._match_history.length === 0) continue
        const this_epoch =
          _user._match_history[_user._match_history.length - 1].epoch
        if (this_epoch < current_first) {
          current_first = this_epoch
          user = _user
        }
      }

      const match = user._match_history.pop()
      const time = new Date(match.epoch)
      if (day === '') day = time.toDateString() //first day
      if (Date.parse(time.toDateString()) > Date.parse(day)) {
        this.layers.push(this.genLayers((col = !col), [{ x: min_x }, { x: x }]))
        min_x = x
        day = time.toDateString()
      }

      const y = Math.round(matchElo(match, user.name))
      if (prev !== current_first) x++
      items[user.name].push({
        x: x,
        y: y,
        time: time,
        winner: match.winner,
        winner_elo: Math.round(match.winner_elo),
        loser: match.loser,
        loser_elo: Math.round(match.loser_elo),
        name: user.name,
        elo_diff: Math.round(match.elo_diff),
      })
      prev = current_first

      this.minElo = y < this.minElo ? y : this.minElo
      this.maxElo = y > this.maxElo ? y : this.maxElo
    }
    this.layers.push(this.genLayers((col = !col), [{ x: min_x }, { x: x }]))

    return items
  }

  genLayers(flag_change_col, day) {
    const Layer = ({ xScale, innerHeight }) => {
      const areaGenerator = area()
        .x((d) => xScale(d.x))
        .y0(0)
        .y1(innerHeight)
        .curve(curveMonotoneX)

      const [color, id, fill] = flag_change_col
        ? ['#ff8c00', '1', 'url(#1)']
        : ['orange', '2', 'url(#2)']

      return (
        <>
          <Defs
            defs={[
              {
                id: id,
                type: 'patternLines',
                background: 'transparent',
                color: color,
                lineWidth: 1,
                spacing: 3,
                rotation: -45,
              },
            ]}
          />
          <path
            d={areaGenerator(day)}
            fill={fill}
            fillOpacity={0.2}
            stroke="var(--primary-color)"
            strokeWidth={0.5}
            strokeOpacity={0.2}
          />
        </>
      )
    }
    return Layer
  }

  render() {
    if (this.users === undefined) return <h1>Loading users</h1>
    if (this.users.length === 0)
      return (
        <>
          <Select
            className="selectorElo"
            onChange={this.addUser}
            options={this.userList
              .filter((user) => !this.users.includes(user.name))
              .map((user) => ({ value: user.name, label: user.name }))}
            value={this.users.map((user) => ({
              value: user.name,
              label: user.name,
            }))}
            isMulti={true}
          />
          <h1>No users selected</h1>
        </>
      )
    let items = []
    this.layers = []

    this.minElo = this.users[0].elo
    this.maxElo = this.users[0].elo
    this.users.forEach((user, i) => {
      items.push({
        id: user.name,
        data: [],
      })
    })
    let matches = this.orderMatches(this.users)
    items.forEach((item) => {
      if (matches[item.id].length === 0) {
        item.data = [
          {
            x: 0,
            y: Math.round(this.users.find((user) => user.name === item.id).elo),
          },
        ]
      } else {
        item.data = matches[item.id]
      }
    })
    return (
      <>
        <h2>Elo history</h2>
        <div className="inputs">
          <Select
            className="selectorElo first-selector"
            onChange={this.changePeriod}
            options={this.periods}
            value={this.selectedPeriod}
          />
          <br />
          <Select
            className="selectorElo"
            onChange={this.addUser}
            options={this.userList
              .filter((user) => !this.users.includes(user.name))
              .map((user) => ({ value: user.name, label: user.name }))}
            value={this.users.map((user) => ({
              value: user.name,
              label: user.name,
            }))}
            isMulti={true}
          />
        </div>
        <h2>Season</h2>
        <div className="inputs">
          <Select
            className="selectorElo first-selector"
            onChange={this.changeSeason}
            options={[
              { value: '', label: 'latest' },
              ...this.seasons.map((season) => ({
                value: season,
                label: season,
              })),
            ]}
            value={this.selectedSeason}
          />
        </div>
        <div style={{ width: '100%', height: '600px' }}>
          <ResponsiveLine
            data={items}
            margin={{ top: 50, right: 10, bottom: 100, left: 40 }}
            xScale={{
              type: 'linear',
              max: 'auto',
              min: '0',
            }}
            yScale={{
              type: 'linear',
              max: this.maxElo + 25,
              min: this.minElo - 25,
            }}
            axisBottom={{
              orient: 'bottom',
              tickSize: 0,
              format: () => null,
              tickPadding: 5,
              tickRotation: 45,
              legendPosition: 'middle',
              legendOffset: 40,
            }}
            axisLeft={{
              orient: 'left',
              tickSize: 5,
              tickPadding: 5,
            }}
            pointSize={4}
            pointColor={{ theme: 'background' }}
            pointBorderWidth={2}
            pointBorderColor={{ from: 'serieColor' }}
            useMesh={true}
            enableSlices={'x'}
            sliceTooltip={({ slice }) => {
              let random_player = {}
              if (slice.points.length === 1) {
                random_player['data'] =
                  slice.points[0].data.winner !== slice.points[0].serieId
                    ? {
                        winner: slice.points[0].data.winner,
                        y: slice.points[0].data.winner_elo,
                      }
                    : {
                        loser: slice.points[0].data.loser,
                        y: slice.points[0].data.loser_elo,
                      }
                random_player['data'].elo_diff = slice.points[0].data.elo_diff
                random_player['serieId'] =
                  slice.points[0].data.winner === slice.points[0].serieId
                    ? slice.points[0].data.loser
                    : slice.points[0].data.winner
              }
              return (
                <div className="tooltip">
                  <div className="justify-center">
                    {getShortDate(slice.points[0].data.time)}
                  </div>
                  {slice.points.map((player, i) => {
                    return <GraphToolTip props={player} key={i} />
                  })}
                  {slice.points.length === 1 &&
                    random_player.serieId !== undefined && (
                      <GraphToolTip props={random_player} key={-1} />
                    )}
                </div>
              )
            }}
            layers={[
              ...this.layers,
              'markers',
              'areas',
              'lines',
              'slices',
              'axes',
              'points',
              'legends',
              'crosshair',
              'mesh',
            ]}
            theme={{
              textColor: 'var(--primary-color)',
              background: 'var(--black)',
              fontSize: 14,
              axis: {
                domain: {
                  line: {
                    stroke: 'var(--primary-color)',
                    strokeWidth: 1,
                  },
                },
                ticks: {
                  line: {
                    stroke: 'var(--primary-color)',
                    strokeWidth: 1,
                  },
                },
              },
              crosshair: {
                line: {
                  stroke: 'var(--primary-color)',
                  strokeWidth: 1,
                  strokeOpacity: 0.5,
                },
              },
            }}
            legends={[
              {
                anchor: 'top-left',
                direction: 'row',
                symbolShape: 'circle',
                itemCount: this.users.length,
                itemWidth: 100,
                itemHeight: 25,
                itemsSpacing: 10,
                translateX: 10,
              },
            ]}
          ></ResponsiveLine>
        </div>
      </>
    )
  }
}

export default EloGraph
