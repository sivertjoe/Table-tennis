import { React, Component } from 'react'
import '../../../index.css'
import '../Tournaments.css'
import './Tournament.css'
import TournamentList from '../../../components/tournament-list/TournamentList'

class Tournament extends Component {
  constructor(args) {
    super()
    this.state = {
      show: args.show,
      goBack: args.goBack,
      tournament: args.tournament,
    }
  }

  static getDerivedStateFromProps(props, state) {
    return props
  }

  render() {
    return (
      <div className={'body' + (this.state.show ? ' visible' : ' hidden')}>
        <span className="arrow" onClick={this.state.goBack}>
          &#10229;
        </span>
        <div className="container ">
          {this.state.tournament ? (
            <TournamentList tournament={this.state.tournament} />
          ) : (
            <h1>No tournament selected...</h1>
          )}
        </div>
      </div>
    )
  }
}
export default Tournament
