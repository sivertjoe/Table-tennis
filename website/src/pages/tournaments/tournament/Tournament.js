import { React, Component } from 'react'
import '../../../index.css'
import '../Tournaments.css'
import './Tournament.css'
// import * as Api from '../../../api/TournamentApi'
// import Button from '../../../components/button/Button'

class Tournament extends Component {
  constructor(args) {
    super()
    this.state = {
      show: args.show,
      goBack: args.goBack,
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
          <h1>Testing</h1>
        </div>
      </div>
    )
  }
}
export default Tournament
