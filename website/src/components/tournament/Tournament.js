import React from 'react'
import '../../index.css'
import './Tournament.css'
import * as Api from '../../api/TournamentApi'

import Modal from 'react-modal'
import Select from 'react-select'
// Modal.setAppElement('.tournament')

function TournamentMatch(props) {
  //   var subtitle
  const [selectedClient, setSelectedClient] = React.useState(undefined)
  const [modalIsOpen, setIsOpen] = React.useState(false)
  function openModal() {
    if (props.match === undefined) return //ouch
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
    if (selectedClient === undefined) return

    winner = selectedClient
    loser =
      props.match.player1 === winner ? props.match.player2 : props.match.player1
    Api.registerTournamentMatch(winner, loser, props.match.id)
      .then(() => {
        closeModal()
      })
      .catch((e) => console.warn('Jaha' + e))
  }
  function _handleChange(event) {
    setSelectedClient(event.value)
  }

  const customStyles = {
    content: {
      top: '50%',
      left: '50%',
      right: 'auto',
      bottom: 'auto',
      marginRight: '-50%',
      height: '150px',
      transform: 'translate(-50%, -50%)',
      backgroundColor: 'var(--background-color)',
      opacity: '100',
    },
    overlay: {
      //   backgroundColor: 'gray',
      color: 'black',
      backgroundColor: 'rgba(128, 128, 128, 0.8)',
    },
  }
  const options = [
    { value: props.match?.player1, label: props.match?.player1 },
    { value: props.match?.player2, label: props.match?.player2 },
  ]
  let winner,
    loser = undefined
  return (
    <div className="match-info">
      {/* <button onClick={openModal}>Open Modal</button> */}
      <p onClick={openModal}>
        {props.match?.player1}, {props.match?.player2}
      </p>
      <Modal
        isOpen={modalIsOpen}
        onAfterOpen={afterOpenModal}
        onRequestClose={closeModal}
        style={customStyles}
        ariaHideApp={false}
      >
        <div>
          Editing match. Who won between {props.match?.player1},{' '}
          {props.match?.player2}
          <Select
            // value={selectedClient}
            options={options}
            placeholder="Select a person"
            onChange={_handleChange}
          />
        </div>
        <button onClick={commitMatch}>Commit</button>
      </Modal>
      {/* {(props.match.player1 = 'kåre')} */}
    </div>
  )
}

function TournamentBracket(props) {
  let ret = []
  const l = props.matches?.length

  //   for-loop, cause match can be undefined
  for (let i = 0; i < props.nMatches; i++) {
    let match = l > i ? props.matches[i] : undefined
    console.log(match)
    ret.push(
      <div className="match">
        <TournamentMatch match={match} />
      </div>,
    )
  }

  return <div className="bracket">{ret}</div>
}

export const Tournament = (props) => {
  console.log(props.tournament)
  if (props.tournament === undefined) {
    return <div>Loading..</div>
  }
  console.log(props.tournament)
  //   const [matches, setMatches] = React.useState(props.)

  let tournament = props.tournament
  let games = tournament.data.Games?.reverse()
  let numBrackets = Math.ceil(Math.log2(tournament.tournament.player_count))

  let power = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  for (let i = 0; i < numBrackets; i++) {
    power /= 2
    tournamentBrackets.push(
      <TournamentBracket matches={games?.splice(0, power)} nMatches={power} />,
    )
  }
  console.log(props.tournament)

  return <div className="tournament">{tournamentBrackets}</div>
}

export default Tournament
