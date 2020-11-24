import { React, Component } from 'react'
import * as Api from '../../api/Api'
import { LineChart, Line, CartesianGrid, XAxis, YAxis } from 'recharts'

class EloGraph extends Component {
  items = []
  ticks = []
  constructor(user) {
    super()
    let name = user.user['name']
    this.items = user.user.match_history?.map((user) => {
      let elo = user.winner === name ? user.winner_elo : user.loser_elo
      return { date: new Date(user.epoch), elo: Math.trunc(elo) }
    })
    if (this.items) {
      let temp = this.items.map((obj) => obj.elo)
      let min = Math.round(Math.min(...temp) / 100) * 100
      let max = Math.round(Math.max(...temp) / 100) * 100

      this.ticks.push(min - 100)
      for (let i = min; i <= max; i += 100) this.ticks.push(i)
      this.ticks.push(max + 100)
    }
    this.setState({})
  }

  render() {
    return (
      <LineChart width={950} height={500} data={this.items}>
        <Line type="monotone" dataKey="elo" stroke="gold" />
        <CartesianGrid stroke="#ebf0f2" />
        <XAxis dataKey="date" stroke="#ebf0f2" />
        <YAxis
          type="number"
          dataKey={(v) => v.elo}
          domain={['auto', 'auto']}
          stroke="#ebf0f2"
          ticks={this.ticks}
          tickCount={this.ticks.length}
        />
      </LineChart>
    )
  }
}

export default EloGraph
