import { React, Component } from 'react'
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer } from 'recharts'
import './EloGraph.css'

function yearDate(date) {
  var start = new Date(date.getFullYear(), 0, 0)
  var diff =
    date -
    start +
    (start.getTimezoneOffset() - date.getTimezoneOffset()) * 60 * 1000
  var oneDay = 1000 * 60 * 60 * 24
  var day = Math.floor(diff / oneDay)
  return day
}

function formatDate(ms) {
  const d = new Date(ms)
  return `${d.getDate()}`
}
function dateContained(array, date) {
  if (array.length === 0) return false
  const oneDay = 86400000
  let d = array[array.length - 1]
  let res = d - date
  return res >= 0 && res <= oneDay * 2
}

function setYticks(array) {
  let temp = array.map((obj) => obj.elo)
  let min = Math.round(Math.min(...temp) / 100) * 100
  let max = Math.round(Math.max(...temp) / 100) * 100

  let new_array = []
  new_array.push(min - 100)
  for (let i = min; i <= max; i += 100) new_array.push(i)
  new_array.push(max + 100)
  return new_array
}

function setXticks(array) {
  const oneDay = 86400000
  let min = new Date(array[0].date).getDate()
  let max = new Date(array[array.length - 1].date).getDate()
  let minDay = array[0].date
  let newArray = []
  for (let i = min; i + 1 <= max; i += 2) {
    newArray.push({ date: minDay + (i - min) * oneDay, elo: 1500 })
  }
  return newArray
}

function reduceArray(array, nMonths) {
  const new_array = []

  var temp = array.slice().reverse()
  const oldestDay = yearDate(new Date(temp[0].date))

  // Filter such that you only get the game in that month interval, e.g 1 month, 3 months
  for (let i = temp.length - 2; i >= 0; i--) {
    const date = new Date(temp[i].date)
    if (oldestDay - yearDate(date) <= 30) {
      new_array.push(temp[i])
    }
  }

  // Go from the start to the end of the month interval, then go back to the start of
  // the month
  let dayScore = undefined
  let day = undefined
  let final_array = []
  for (let i = 0; i < new_array.length; i++) {
    if (dayScore === undefined) {
      dayScore = new_array[i].elo
      day = new_array[i].date
      continue
    }

    let d1 = new Date(new_array[i].date)
    let d2 = new Date(day)
    let dayDiff = d1.getDate() - d2.getDate()

    if (dayDiff !== 0) {
      // Okay, _at_ _least_ one day of no play have happend, figure out
      // how much and padd it
      padDays(final_array, dayScore, dayDiff, day)

      dayScore = new_array[i].elo
      day = new_array[i].date
    }
    dayScore = new_array[i].elo
    final_array.push({ date: new_array[i].date, elo: new_array[i].elo })
  }
  return final_array
}

function padDays(array, elo, nDays, day) {
  const oneDay = 86400000
  for (let i = 1; i < nDays; i++) {
    array.push({ date: day + oneDay * i, elo: elo })
  }
}

function smoothGraph(array) {
    console.log(array)
    if(array.length < 5) return;
    for(let i = 1; i < array.length - 1; i++) { 
        const left = array[i - 1].elo
        const right = array[i + 1].elo
        let center = array[i].elo

        if((left < center && right < center) || (left > center && right > center)) {
            array[i].elo  = (left + right) / 2
        }
    }
}

class EloGraph extends Component {
  items = []
  yTicks = []
  xTicks = []
  data = []
  constructor(user) {
    super()
    let name = user.user['name']
    this.data = user.user.match_history
      ?.map((user) => {
        let elo = user.winner === name ? user.winner_elo : user.loser_elo
        return { date: user.epoch, elo: Math.trunc(elo) }
      })
      .reverse()
    this.items = this.data
    if (this.items) {
      this.items = reduceArray(this.items, 1)
      smoothGraph(this.items)
      this.yTicks = setYticks(this.items)
      this.xTicks = setXticks(this.items)
    }
  }

  render() {
    return (
      <ResponsiveContainer>
        <LineChart data={this.items}>
          <Line dot={false} dataKey="elo" stroke="gold" />
          <XAxis
            dataKey={(v) => v.date}
            tickFormatter={formatDate}
            stroke="#ebf0f2"
            type="number"
            domain={['dataMin', 'dataMax']}
            scale={'time'}
          />
          <YAxis
            type="number"
            dataKey={(v) => v.elo}
            domain={['auto', 'auto']}
            stroke="#ebf0f2"
            ticks={this.yTicks}
            tickCount={this.yTicks.length}
          />
        </LineChart>
      </ResponsiveContainer>
    )
  }
}

export default EloGraph
