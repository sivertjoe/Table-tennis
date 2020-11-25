import { React, Component } from 'react'
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer } from 'recharts'
import './EloGraph.css'

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
    newArray.push(minDay + (i - minDay) * oneDay)
  }
  return newArray
}

function reduceArray(array, nMonths) {
  const new_array = []

  var temp = array.slice().reverse()
  const oldestMonth = new Date(temp[temp.length - 1].date).getMonth()

  // Filter such that you only get the game in that month interval, e.g 1 month, 3 months
  for (let i = temp.length - 2; i >= 0; i--) {
    const date = new Date(temp[i].date).getMonth()
    if (date - oldestMonth <= nMonths) {
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

    final_array.push({ date: new_array[i].date, elo: new_array[i].elo })
    if (dayDiff !== 0) {
      // Okay, _at_ _least_ one day of no play have happend, figure out
      // how much and padd it
      padDays(final_array, dayScore, dayDiff, day)

      dayScore = new_array[i].elo
      day = new_array[i].date
    }
  }

  return final_array
}

function padDays(array, elo, nDays, day) {
  const oneDay = 86400000
  for (let i = 1; i < nDays; i++) {
    array.push({ date: day + oneDay * i, elo: elo })
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
      this.yTicks = setYticks(this.items)
      this.xTicks = setXticks(this.items)
    }
  }

  render() {
    return (
      <ResponsiveContainer>
        <LineChart data={this.items}>
          <Line dot={false} type="monotone" dataKey="elo" stroke="gold" />
          <XAxis
            dataKey={(v) => v.date}
            stroke="#ebf0f2"
            ticks={this.xTicks}
            tickFormatter={formatDate}
            interval={0}
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
