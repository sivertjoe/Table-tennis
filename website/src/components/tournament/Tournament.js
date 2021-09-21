import React from 'react'
import { DoubleElimination } from './DoubleElimination'
import { SingleElimination } from './SingleElimination'

export const Tournament = (props) => {
  switch (props.info.ttype) {
    case 0:
      return <SingleElimination info={props.info} matches={props.matches} />
    case 1:
      return <DoubleElimination info={props.info} matches={props.matches} />

    default:
      return <p>error</p>
  }
}

export default Tournament
