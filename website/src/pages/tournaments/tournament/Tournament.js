import { React } from 'react'
import '../../../index.css'
import '../Tournaments.css'
import './Tournament.css'
// import * as Api from '../../../api/TournamentApi'
// import Button from '../../../components/button/Button'

export default function Tournament(show, goBack) {
  return (
    <div className={'body' + (show ? ' visible' : ' hidden')}>
      <span className="arrow" onClick={goBack}>
        &#10229;
      </span>
      <div className="container ">
        <h1>Testing</h1>
      </div>
    </div>
  )
}
