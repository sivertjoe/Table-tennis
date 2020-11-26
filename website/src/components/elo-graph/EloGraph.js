import { React, Component } from 'react'
import { LineChart, Line, XAxis, YAxis, ResponsiveContainer } from 'recharts'
import './EloGraph.css'

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

class EloGraph extends Component {
  items = []
  yTicks = []
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
      this.yTicks = setYticks(this.items)
    }
  }

  render() {
    return (
      <ResponsiveContainer>
        <LineChart data={this.items}>
          <Line dot={false} dataKey="elo" stroke="gold" />
          <XAxis stroke="#ebf0f2" />
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
