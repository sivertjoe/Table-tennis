import React from 'react'
import '../../index.css'
import './Tournament.css'
import * as Api from '../../api/TournamentApi'

import Button from '../button/Button'
import Modal from 'react-modal'
import Select from 'react-select'

const finals = {
  2: 'Final',
  4: 'Semifinals',
  8: 'Quarterfinals',
  16: 'Eighth-finals',
  32: '16th-finals',
  64: '32nd-finals',
}

function TournamentMatch(props) {
  const [selectedClient, setSelectedClient] = React.useState(undefined)
  const [modalIsOpen, setIsOpen] = React.useState(false)
  const options = [
    { value: props.match.player1, label: props.match.player1 },
    { value: props.match.player2, label: props.match.player2 },
  ]

  function parentNotPlayed(match, matches) {
    if (match.bucket === 0) {
      return true
    }

    let parentIndex = Math.trunc((match.bucket - 1) / 2)
    let parent = matches.find((m) => {
      return parentIndex === m.bucket
    })

    return ![parent.player1, parent.player2].filter((m) =>
      [match.player1, match.player2].includes(m),
    ).length
  }
  function openModal() {
    if (
      props.winner === '' &&
      props.match.player1 !== '' &&
      props.match.player2 !== '' &&
      props.organizer === localStorage.getItem('username') &&
      parentNotPlayed(props.match, props.matches)
    )
      setIsOpen(true)
  }
  function closeModal() {
    setIsOpen(false)
  }
  function commitMatch() {
    if (!selectedClient) return

    let winner = selectedClient
    let loser =
      props.match.player1 === winner ? props.match.player2 : props.match.player1
    Api.registerTournamentMatch(winner, loser, props.match.id)
      .then(() => closeModal())
      .catch((e) => console.warn('Jaha' + e))
    props.callback(props.match, winner)
    closeModal()
  }

  function _handleChange(event) {
    setSelectedClient(event.value)
  }

  let winner = props.winner
  if (props.match.bucket !== 0) {
    let parentIndex = Math.trunc((props.match.bucket - 1) / 2)
    let parent = props.matches.find((m) => {
      return parentIndex === m.bucket
    })

    winner = [parent?.player1, parent?.player2].filter((m) =>
      [props.match.player1, props.match.player2].includes(m),
    )[0]
  }

  return (
    <>
      <div className="match-info" onClick={openModal}>
        <span
          style={{
            gridColumn: 1,
            color: winner === props.match.player1 ? 'var(--orange)' : '',
          }}
        >
          {props.match.player1}
        </span>
        <span style={{ gridColumn: 2 }}>|</span>
        <span
          style={{
            gridColumn: 3,
            color: winner === props.match.player2 ? 'var(--orange)' : '',
          }}
        >
          {props.match.player2}
        </span>
      </div>

      <Modal
        className="Modal"
        overlayClassName="Overlay"
        isOpen={modalIsOpen}
        onRequestClose={closeModal}
        ariaHideApp={false}
      >
        <div className="modal-body">
          <h3>
            Editing match. Who won between {props.match.player1} &{' '}
            {props.match.player2}
          </h3>
          <Select
            className="black"
            options={options}
            placeholder="Select a person"
            onChange={_handleChange}
          />
          <div onClick={() => commitMatch(selectedClient)}>
            <Button style={{ marginTop: '2rem' }} placeholder="Submit" />
          </div>
        </div>
      </Modal>
    </>
  )
}

function TournamentBracket(props) {
  let ret = []
  const start = props.start
  const stop = props.stop
  const matches = props.matches
  const l = matches.length
  //   for-loop, cause match can be undefined
  for (let i = start; i < stop; i++) {
    ret.push(
      <div className="match" key={'match-div-' + i}>
        <TournamentMatch
          match={matches[i]}
          matches={matches}
          key={'match' + { i }}
          callback={props.callback}
          organizer={props.organizer}
          winner={props.winner}
        />
      </div>,
    )
  }

  return (
    <div className="bracket-container">
      <h2>{props.title}</h2>
      <div
        className={'bracket' + (stop === l ? '' : ' border-right')}
        key={'bracket-' + props.start}
      >
        {ret}
      </div>
    </div>
  )
}

export const SingleElimination = (props) => {
  const [matches, setMatches] = React.useState([...props.matches])
  const [tournament, setInfo] = React.useState(props.info)

  //matches would not rerender when parent rerender (:
  if (tournament.id !== props.info.id) {
    setInfo(props.info)
    setMatches([...props.matches])
  }
  if (!matches) {
    return <div>Loading..</div>
  }

  function handleInputChange(match, winner) {
    if (match.bucket === 0) {
      tournament.winner = winner
    } else {
      let parent = Math.trunc((match.bucket - 1) / 2)
      const index = matches.findIndex((m) => m.bucket === parent)

      if ((match.bucket & 1) === 1) {
        matches[index].player1 = winner
      } else {
        matches[index].player2 = winner
      }
    }

    setMatches([...matches])
  }
  let numBrackets = Math.ceil(Math.log2(tournament.player_count))
  let n_matches = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  let start_match = 0
  let competitors = n_matches

  for (let i = 0; i < numBrackets; i++) {
    n_matches /= 2
    tournamentBrackets.push(
      <TournamentBracket
        start={start_match}
        stop={start_match + n_matches}
        matches={matches}
        callback={handleInputChange}
        organizer={tournament.organizer_name}
        title={finals[competitors]}
        key={' bracket' + i}
        winner={tournament.winner}
      />,
    )
    competitors /= 2
    start_match += n_matches
  }
  if (tournament.winner !== '') {
    tournamentBrackets.push(
      <div className="bracket-container" key="winner-bracket">
        <h2>Winner</h2>
        <div className="bracket">
          <div className="match">
            <div className="winner">{tournament.winner}</div>
          </div>
        </div>
      </div>,
    )
  }

  return (
    <>
      <div key="tournament" className="tournament">
        {tournamentBrackets}
      </div>
    </>
  )
}

export default SingleElimination
