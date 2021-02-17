import { React, Component } from 'react'
import { ResponsiveLine } from '@nivo/line'
import Select from 'react-select'
import './EloGraph.css'
import { Defs } from '@nivo/core'
import { area, curveMonotoneX } from 'd3-shape'
import { getShortDate, getPreviousDate } from '../../utils/Date'
import * as UserApi from '../../api/UserApi'

function matchElo(match, name) {
  return match.winner === name ? match.winner_elo : match.loser_elo
}

class EloGraph extends Component {
  x = 0

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
      label: 'This Season',
      ticks: 'every 2 day',
      date: getPreviousDate(30),
    },
    // {
    //   value: 'alltime',
    //   label: 'All Time',
    //   ticks: 'every 1 year',
    //   date: new Date(0),
    // },
    /* Qurey db for all time */
  ]
  selectedPeriod = this.periods[1]

  constructor(args) {
    super()
    this.users = args.users.filter((user) => user.match_history.length > 0)
    this.changePeriod = this.changePeriod.bind(this)
    this.genLayers = this.genLayers.bind(this)
    // UserApi.getUser('Sander')
    //   .then((user) => {
    //     // user.match_history = user.match_history.reverse()
    //     this.users.push(user)
    //   })
    //   .finally(() => this.setState({}))
    // UserApi.getUser('Sivert')
    //   .then((user) => this.users.push(user))
    //   .finally(() => this.setState({}))
    // UserApi.getUser('Ella')
    //   .then((user) => this.users.push(user))
    //   .finally(() => this.setState({}))
  }

  changePeriod(e) {
    this.selectedPeriod = e
    this.setState({})
  }

  orderMatches(users) {
    let items = {}
    this.union = []
    users.forEach((user) => {
      items[user.name] = []
      user._match_history = user.match_history.filter(
        (match) => match.epoch >= this.selectedPeriod.date.getTime(),
      )
      //   user.match_history.forEach((match) => {
      //     // if (match.epoch > this.selectedPeriod.date.getTime()) {
      //     if (!this.union.includes(match.epoch)) {
      //       this.union.push(match.epoch)
      //     }
      //     // }
      //   })
    })
    this.union.sort()
    // console.log(items)

    let indexes = users.map((user) => user._match_history.length - 1)
    //loop over each user
    // users.forEach((user) => (user._match_history = user._match_history.reverse()))
    let x = -1
    let prev = -1
    while (users.some((_user, _i) => indexes[_i] >= 0)) {
      let small = Infinity
      let index = -1

      users.forEach((user, i) => {
        const _match_history = user._match_history
        const len = _match_history.length
        const userIndex = indexes[i]
        if (userIndex < 0) {
          return
        }

        if (_match_history[userIndex].epoch < small) {
          small = _match_history[userIndex].epoch
          index = i
        }
      })
      if (prev !== small) x++

      const user = users[index]
      const match = user._match_history[indexes[index]]
      indexes[index] -= 1
      const y = Math.round(matchElo(match, user.name))
      items[user.name].push({
        x: x,
        y: y,
        time: new Date(match.epoch),
        info: [match.winner, match.loser],
        name: user.name,
        elo_diff: Math.round(match.elo_diff),
      })
      prev = small

      this.minElo = y < this.minElo ? y : this.minElo
      this.maxElo = y > this.maxElo ? y : this.maxElo
    }
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
    let items = []
    this.minElo = this.users[0].elo
    this.maxElo = this.users[0].elo
    this.users.forEach((user, i) => {
      items.push({
        id: user.name,
        data: [],
      })
    })
    // console.log(this.users)
    let matches = this.orderMatches(this.users)
    // console.log(matches)
    items.forEach((item) => {
      if (matches[item.id].length === 0 && this.users.length === 1) {
        console.log('panic')
        item.data = [
          {
            time: new Date(),
            x: 0,
            y: Math.round(this.users.find((user) => user.name === item.id).elo),
          },
        ]
      } else {
        item.data = matches[item.id]
      }
    })
    // this.union = this.union.map((d) => {
    //   d = new Date(new Date(d).toDateString())
    //   return d.getTime()
    // })
    // this.union = [...new Set(this.union)]

    // let col = true
    let layers = []

    // let min = { x: 0 },
    //   max = { x: 0 }

    // for (let i = 0; i < this.union.length; i++) {
    //   items.forEach((user) => {
    //     const a = user.data.filter((match) => {
    //       const t = new Date(match.time.toDateString())
    //       return t.getTime() === this.union[i]
    //     })
    //     if (a.length !== 0) {
    //       min.x = Math.min(min.x, a[0].x)
    //       max.x = Math.max(max.x, a[a.length - 1].x)
    //     } else return
    //   })
    //   col = !col
    //   const layer = [{ x: min.x }, { x: max.x }]
    //   layers.push(this.genLayers(col, layer))
    //   min.x = max.x
    // }
    return (
      <>
        <h2>Elo history</h2>
        <div className="inputs">
          <Select
            className="selector"
            onChange={this.changePeriod}
            options={this.periods}
            value={this.selectedPeriod}
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
              type: items[0].data.length === 1 ? 'point' : 'linear',
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
              return (
                <div className="tooltip">
                  {slice.points.map((player, i) => {
                    return (
                      <div key={i}>
                        <span
                          style={{
                            height: '15px',
                            width: '15px',
                            backgroundColor: player.serieColor,
                            display: 'inline-block',
                            borderRadius: '50%',
                          }}
                        ></span>
                        {' ' + player.serieId}
                        {' ' + player.data.y}
                        {player.data.elo_diff === undefined
                          ? ''
                          : player.data.info[0] === player.serieId
                          ? '(+' + player.data.elo_diff + ')'
                          : '(-' + player.data.elo_diff + ')'}
                        {player.data.info
                          ? 'W: ' +
                            player.data.info[0] +
                            ', L: ' +
                            player.data.info[1]
                          : ' '}
                        {player.data.x}
                      </div>
                    )
                  })}
                  {getShortDate(slice.points[0].data.time)}
                </div>
              )
            }}
            layers={[
              ...layers,
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
            // animate={false} //#TODO: find out if only some parts can animate
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
