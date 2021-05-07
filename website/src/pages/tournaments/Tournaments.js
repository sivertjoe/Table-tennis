import { React, Component } from 'react'
import '../../index.css'
import './Tournaments.css'
import Menu from './menu/Menu'
import Tournament from './tournament/Tournament'

const Pages = {
  Menu: 0,
  Tournament: 1,
}

class Tournaments extends Component {
  page = Pages.Menu

  constructor() {
    super()
    this.goBack = this.goBack.bind(this)
    this.selectTournament = this.selectTournament.bind(this)
  }

  goBack() {
    this.page = Pages.Menu
    this.setState({})
  }

  selectTournament() {
    this.page = Pages.Tournament
    this.setState({})
  }

  render() {
    return (
      <div className="page">
        <Menu
          show={this.page === Pages.Menu}
          selectTournament={this.selectTournament}
        />
        <Tournament
          show={this.page === Pages.Tournament}
          goBack={this.goBack}
        />
      </div>
    )
  }
}

export default Tournaments
