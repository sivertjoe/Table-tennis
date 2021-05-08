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
      selectedTournament: null,
    }
  }

  render() {
    return (
      <div className="page">
        <Menu
          show={this.state.page === Pages.Menu}
          onSelectTournament={(tournament) =>
            this.setState({
              page: Pages.Tournament,
              selectedTournament: tournament,
            })
          }
        />
        <Tournament
          show={this.state.page === Pages.Tournament}
          tournament={this.state.selectedTournament}
          goBack={() => this.setState({ page: Pages.Menu })}
        />
      </div>
    )
  }
}

export default Tournaments
