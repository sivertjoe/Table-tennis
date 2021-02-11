import { React, Component } from 'react'
import { ResponsiveLine } from '@nivo/line'
import Select from 'react-select'
import './EloGraph.css'
import { Defs } from '@nivo/core'
import { area, curveMonotoneX } from 'd3-shape'
import { getShortDate, getPreviousDate } from '../../utils/Date'

function matchElo(match, name) {
  return match.winner === name ? match.winner_elo : match.loser_elo
}

class EloGraph extends Component {
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
    this.user = args.user
    this.changePeriod = this.changePeriod.bind(this)
    this.getEveryMatch = this.getEveryMatch.bind(this)
    this.genLayers = this.genLayers.bind(this)
  }

  changePeriod(e) {
    this.selectedPeriod = e
    this.setState({})
  }

  getEveryMatch() {
    const name = this.user.name
    let x = 0
    return this.user.match_history?.reduceRight((res, match, i) => {
      if (match.epoch > this.selectedPeriod.date.getTime()) {
        res.push({
          y: Math.round(matchElo(match, name)),
          x: x,
          time: new Date(match.epoch),
          info: [match.winner, match.loser],
          elo_diff: Math.round(match.elo_diff),
        })
        x++
      }
      return res
    }, [])
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
    const items = [
      {
        id: 0,
        data: this.getEveryMatch(),
      } /* TODO: Insert opponent's graph here */,
    ]

    let minElo = this.user.elo,
      maxElo = this.user.elo

    let layers = []
    // If there is no history to show, just show current elo
    if (!items[0].data.length) {
      items[0].data = [{ time: new Date(), x: 0, y: Math.round(this.user.elo) }]
    } else {
      //min/max elo for upper and lower y value
      for (let i = 1; i < items[0].data.length; i++) {
        let value = items[0].data[i]
        minElo = value.y < minElo ? value.y : minElo
        maxElo = value.y > maxElo ? value.y : maxElo
      }

      //norm the timestamps in order to put them in the same grid
      items[0].data.forEach((d) => {
        d.dateTime = new Date(d.time.toDateString())
      })

      let first_match = items[0].data[0]
      let last_match = items[0].data[0]
      let col = true

      //send first match and last match of the current day to in order for generating layer for that day
      for (let i = 0; i < items[0].data.length; i++) {
        if (first_match.dateTime.getTime() === last_match.dateTime.getTime()) {
          last_match = items[0].data[i + 1]
        } else {
          col = !col
          layers.push(this.genLayers(col, [first_match, last_match]))
          first_match = last_match
        }
      }
      col = !col
      layers.push(
        this.genLayers(col, [
          first_match,
          items[0].data[items[0].data.length - 1],
        ]),
      )
    }
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
              format: 'native',
              max: 'auto',
              min: '0',
            }}
            yScale={{
              type: items[0].data.length === 1 ? 'point' : 'linear',
              max: maxElo + 25,
              min: minElo - 25,
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
            tooltip={(input) => (
              <div className="tooltip">
                <div>
                  {input.point.data.info
                    ? 'W: ' +
                      input.point.data.info[0] +
                      ', L: ' +
                      input.point.data.info[1]
                    : ''}
                </div>
                <div>
                  {input.point.data.y}{' '}
                  {input.point.data.elo_diff === undefined
                    ? ''
                    : input.point.data.info[0] === this.user.name
                    ? '(+' + input.point.data.elo_diff + ')'
                    : '(-' + input.point.data.elo_diff + ')'}
                </div>
                <div>{getShortDate(input.point.data.time)}</div>
              </div>
            )}
          ></ResponsiveLine>
        </div>
      </>
    )
  }
}

export default EloGraph
