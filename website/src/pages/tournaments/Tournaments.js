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
  constructor() {
    super()
    this.state = {
      page: Pages.Menu,
    }
  }

  render() {
    return (
      <div className="page">
        <Menu
          show={this.state.page === Pages.Menu}
          onSelectTournament={() => this.setState({ page: Pages.Tournament })}
        />
        <Tournament
          show={this.state.page === Pages.Tournament}
          goBack={() => this.setState({ page: Pages.Menu })}
        />
      </div>
    )
  }
}

export default Tournaments
