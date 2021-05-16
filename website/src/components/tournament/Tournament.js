import React from 'react'
import '../../index.css'
import './Tournament.css'
import * as Api from '../../api/TournamentApi'

import Button from '../button/Button'
import Modal from 'react-modal'
import Select from 'react-select'
// Modal.setAppElement('.tournament')

function TournamentMatch(props) {
  //   var subtitle
  const [selectedClient, setSelectedClient] = React.useState(undefined)
  const [modalIsOpen, setIsOpen] = React.useState(false)
  function openModal() {
    if (
      props.match.player1 !== '' &&
      props.match.player2 !== '' &&
      props.organizer === localStorage.getItem('username')
    )
      setIsOpen(true)
  }
  function afterOpenModal() {
    // references are now sync'd and can be accessed.
    // subtitle.style.color = '#f00'
  }
  function closeModal() {
    setIsOpen(false)
  }
  function commitMatch() {
    if (!selectedClient) return

    winner = selectedClient
    loser =
      props.match.player1 === winner ? props.match.player2 : props.match.player1
    Api.registerTournamentMatch(winner, loser, props.match.id)
      .then(() => closeModal())
      .catch((e) => console.warn('Jaha' + e))
    props.callback(props.match, winner)
  }
  function _handleChange(event) {
    setSelectedClient(event.value)
  }

  const options = [
    { value: props.match?.player1, label: props.match?.player1 },
    { value: props.match?.player2, label: props.match?.player2 },
  ]
  let winner,
    loser = undefined
  return (
    <div key={'match-info-' + props.match.id}>
      <div className="match-info" onClick={openModal}>
        <span style={{ gridColumn: 1 }}>{props.match?.player1}</span>
        <span style={{ gridColumn: 2 }}>|</span>
        <span style={{ gridColumn: 3 }}>{props.match?.player2}</span>
      </div>
      <Modal
        className="Modal"
        overlayClassName="Overlay"
        isOpen={modalIsOpen}
        onAfterOpen={afterOpenModal}
        onRequestClose={closeModal}
        ariaHideApp={false}
      >
        <div className="modal-body">
          <h3 style={{ marginBottom: '2rem' }}>
            Editing match. Who won between {props.match?.player1} &{' '}
            {props.match?.player2}
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
      {/* {(props.match.player1 = 'kåre')} */}
    </div>
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
    // let match = l > i ? props.matches[i] : undefined
    ret.push(
      <div className="match" key={'match-div-' + i}>
        <TournamentMatch
          match={matches[i]}
          key={'match' + { i }}
          callback={props.callback}
          organizer={props.organizer}
        />
      </div>,
    )
  }

  return (
    <div className="bracket-container">
      <h2>{props.title}</h2>
      <div
        className={'bracket' + (stop === l ? '' : ' border-right')}
        key={'btacket-' + props.start}
      >
        {ret}
      </div>
    </div>
  )
}

export const Tournament = (props) => {
  const [matches, setMatches] = React.useState(props.matches)

  //matches would not rerender when parent rerender (:
  if (matches.length !== props.matches.length) {
    setMatches([...props.matches])
  }
  if (!matches) {
    return <div>Loading..</div>
  }

  function handleInputChange(match, winner) {
    let parent = Math.trunc((match.bucket - 1) / 2)
    const index = matches.findIndex((m) => m.bucket === parent)

    if ((match.bucket & 1) === 1) {
      matches[index].player2 = winner
    } else {
      matches[index].player1 = winner
    }

    setMatches([...matches]) // Ouch.
  }

  let games = matches
  let numBrackets = Math.ceil(Math.log2(props.info.player_count))
  let power = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  let iter = 0
  for (let i = 0; i < numBrackets; i++) {
    power /= 2
    tournamentBrackets.push(
      <TournamentBracket
        start={iter}
        stop={iter + power}
        matches={games}
        callback={handleInputChange}
        organizer={props.info.organizer_name}
        title={'Bracket ' + (i + 1)}
        key={i + ' bracket'}
      />,
    )
    iter += power
  }

  return (
    <div key="tournament" className="tournament">
      {tournamentBrackets}
    </div>
  )
}

export default Tournament
