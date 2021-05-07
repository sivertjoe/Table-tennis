import { React } from 'react'
import '../../../index.css'
import '../Tournaments.css'
import './Menu.css'
// import * as Api from '../../../api/TournamentApi'
import Button from '../../../components/button/Button'

export default function Menu(show, selectTournament) {
  return (
    <div
      className={'container side-menu' + (show ? ' menu-visible' : ' hidden')}
    >
      <h1>Test</h1>
      <Button callback={selectTournament} />
    </div>
  )
}
