import React from 'react'
import { DoubleElimination } from './DoubleElimination'
import { SingleElimination } from './SingleElimination'

function selectTournament(props) {
  switch (props.info.ttype) {
    case 0:
      return <SingleElimination info={props.info} matches={props.matches} />
    case 1:
      return (
        <DoubleElimination
          info={props.info}
          matches={props.matches}
          table={props.table}
        />
      )

    default:
      return <p>error</p>
  }
}

export const Tournament = (props) => {
  const tournament = selectTournament(props)

  return tournament
}

export default Tournament
