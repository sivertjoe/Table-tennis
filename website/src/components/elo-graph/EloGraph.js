import { React, Component } from 'react'
import { ResponsiveLine } from '@nivo/line'
import Select from 'react-select'
import './EloGraph.css'
import { getShortDate, sameDay, getPreviousDate } from '../../utils/Date'

function matchElo(match, name) {
  return match.winner === name ? match.winner_elo : match.loser_elo
}

class EloGraph extends Component {
  periods = [
    {
      value: 'day',
      label: '1 Day',
      ticks: 'every 1 day',
      date: getPreviousDate(1),
    },
    {
      value: 'week',
      label: '1 Week',
      ticks: 'every 1 day',
      date: getPreviousDate(7),
    },
    {
      value: 'month',
      label: '1 Month',
      ticks: 'every 2 day',
      date: getPreviousDate(30),
    },
    {
      value: '3month',
      label: '3 Month',
      ticks: 'every 4 day',
      date: getPreviousDate(91),
    },
    {
      value: '6month',
      label: '6 Month',
      ticks: 'every 1 week',
      date: getPreviousDate(182),
    },
    {
      value: 'year',
      label: '1 Year',
      ticks: 'every 1 month',
      date: getPreviousDate(365),
    },
    {
      value: 'alltime',
      label: 'All Time',
      ticks: 'every 1 year',
      date: new Date(0),
    },
  ]
  selectedPeriod = this.periods[2]

  details = [
    {
      value: 'average',
      label: 'Daily Average',
    },
    {
      value: 'detailed',
      label: 'Every Match',
    },
  ]
  selectedDetail = this.details[0]

  constructor(args) {
    super()
    this.user = args.user
    this.changePeriod = this.changePeriod.bind(this)
    this.changeDetail = this.changeDetail.bind(this)
    this.getEveryMatch = this.getEveryMatch.bind(this)
    this.getDailyAverage = this.getDailyAverage.bind(this)
  }

  changePeriod(e) {
    this.selectedPeriod = e
    this.setState({})
  }

  changeDetail(e) {
    this.selectedDetail = e
    this.setState({})
  }

  getEveryMatch() {
    const name = this.user.name
    return this.user.match_history?.reduceRight((res, match, i) => {
      if (match.epoch > this.selectedPeriod.date.getTime())
        res.push({
          x: new Date(match.epoch),
          y: Math.round(matchElo(match, name)),
        })
      return res
    }, [])
  }

  getDailyAverage() {
    const name = this.user.name
    const history = this.user.match_history

    // Find the last match within the selected range
    const end = this.selectedPeriod.date.getTime()
    const startIdx = history.findIndex((match) => match.epoch < end)
    let idx = (startIdx < 0 ? history.length : startIdx) - 1

    // If there are no matches within the selected range
    if (idx <= 0) return []

    const last = history[idx]
    let currentDate = new Date(last.epoch)
    let sum = matchElo(last, name)
    let count = 1

    let matchDate = new Date(last.epoch)
    let elo = sum

    let res = []
    const tomorrow = getPreviousDate(-1).getTime()
    while (currentDate.getTime() < tomorrow) {
      // If there were no matches on currentDate
      if (!sameDay(matchDate, currentDate) || idx <= 0) {
        // Add the average elo of that day, and go to next day
        res.push({
          x: new Date(currentDate),
          y: Math.round(sum / count),
        })
        currentDate.setDate(currentDate.getDate() + 1)
        sum = elo
        count = 1
      }
      // If the match was on currentDate
      else {
        // Add the match, and go to next match
        sum += elo
        count++
        idx--
        matchDate = new Date(history[idx].epoch)
        elo = matchElo(history[idx], name)
      }
    }

    return res
  }

  render() {
    const items = [
      {
        id: 0,
        data:
          this.selectedDetail.value === 'average'
            ? this.getDailyAverage()
            : this.getEveryMatch(),
      },
    ]

    // If there is no history to show, just show current elo
    if (!items[0].data.length)
      items[0].data = [{ x: new Date(), y: Math.round(this.user.elo) }]

    return (
      <>
        <h2>Elo history</h2>
        <div className="inputs">
          <Select
            className="selector"
            onChange={this.changeDetail}
            options={this.details}
            value={this.selectedDetail}
          />
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
            margin={{ top: 50, right: 50, bottom: 100, left: 50 }}
            xScale={{
              type: 'time',
              format: 'native',
            }}
            yScale={{
              type: 'linear',
              min: 'auto',
              max: 'auto',
            }}
            axisBottom={{
              tickValues: this.selectedPeriod.ticks,
              orient: 'bottom',
              tickSize: 5,
              tickPadding: 5,
              tickRotation: 45,
              format: '%d %b %Y',
            }}
            axisLeft={{
              orient: 'left',
              tickSize: 5,
              tickPadding: 5,
            }}
            colors={{ scheme: 'nivo' }}
            pointSize={4}
            pointColor={{ theme: 'background' }}
            pointBorderWidth={2}
            pointBorderColor={{ from: 'serieColor' }}
            useMesh={true}
            enableGridX={false}
            enableGridY={false}
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
                <div>{input.point.data.y}</div>
                <div>{getShortDate(input.point.data.x)}</div>
              </div>
            )}
          ></ResponsiveLine>
        </div>
      </>
    )
  }
}

export default EloGraph
