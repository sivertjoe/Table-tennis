import { React, Component } from 'react'
import '../../../index.css'
import '../Tournaments.css'
import './Menu.css'
// import * as Api from '../../../api/TournamentApi'
import Button from '../../../components/button/Button'
import Tabs from '../../../components/tabs/Tabs'

class Menu extends Component {
  selectedTab = 0

  constructor(args) {
    super()
    this.state = {
      show: args.show,
      selectTournament: args.selectTournament,
    }
  }

  static getDerivedStateFromProps(props, state) {
    return props
  }

  onSelectTab(i) {}

  render() {
    return (
      <div
        className={
          'container side-menu' +
          (this.state.show ? ' menu-visible' : ' hidden')
        }
      >
        <Tabs tabs={['In progress', 'Old']} onSelectTab={this.onSelectTab} />
        <Button
          placeholder="Create"
          callback={() => (window.location.href = '/create-tournament')}
        />
      </div>
    )
  }
}
export default Menu
