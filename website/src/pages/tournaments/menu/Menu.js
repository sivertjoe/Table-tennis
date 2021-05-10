import { React, Component } from 'react'
import '../../../index.css'
import '../Tournaments.css'
import './Menu.css'
import * as Api from '../../../api/TournamentApi'
import Button from '../../../components/button/Button'
import { default as TabsComponent } from '../../../components/tabs/Tabs'

const Tabs = {
  Active: 0,
  Old: 1,
}

class Menu extends Component {
  constructor(args) {
    super()
    this.state = {
      show: args.show,
      selectedTab: Tabs.Active,
      selectedTournamentId: -1,
      onSelectTournament: args.onSelectTournament,
      active: [],
      old: [],
    }

    this.onSelectTab = this.onSelectTab.bind(this)
    this.onSelectTournament = this.onSelectTournament.bind(this)

    Api.getTournaments('active')
      .then((tournaments) => this.setState({ active: tournaments }))
      .catch((err) => console.warn(err.toString()))
  }

  static getDerivedStateFromProps(props, state) {
    return props
  }

  onSelectTab(tab) {
    if (tab === Tabs.Old && !this.state.old.length)
      Api.getTournaments('old')
        .then((tournaments) => this.setState({ old: tournaments }))
        .catch((err) => console.warn(err.toString()))

    this.setState({ selectedTab: tab })
  }

  onSelectTournament(info) {
    if (this.state.onSelectTournament) this.state.onSelectTournament(info)
    this.setState({ selectedTournamentId: info.tournament.id })
  }

  render() {
    return (
      <div
        className={'container side-menu' + (this.state.show ? '' : ' hidden')}
      >
        <TabsComponent
          tabs={['In progress', 'Old']}
          onSelectTab={this.onSelectTab}
        />
        <div className="table-container">
          <table>
            <tbody>
              {(this.state.selectedTab === Tabs.Active
                ? this.state.active
                : this.state.old
              ).map((info, i) => (
                <tr
                  key={i}
                  className={
                    this.state.selectedTournamentId === info.tournament.id
                      ? 'orange'
                      : ''
                  }
                  onClick={() => this.onSelectTournament(info)}
                >
                  <td>{info.tournament.name}</td>
                  <td>
                    {info.data.Players && info.data.Players.length + '/'}
                    {info.tournament.player_count}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <Button
          placeholder="Create"
          callback={() => (window.location.href = '/create-tournament')}
        />
      </div>
    )
  }
}
export default Menu
