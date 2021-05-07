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
    console.log(winner, loser)
    Api.registerTournamentMatch(winner, loser, props.match.id)
      .then(() => {
        closeModal()
      })
      .catch((e) => console.warn('Jaha' + e))
  }
  function _handleChange(event) {
    console.log(event)
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
    { value: props.match.player1, label: props.match.player1 },
    { value: props.match.player2, label: props.match.player2 },
  ]
  let winner,
    loser = undefined

  return (
    <div>
      {/* <button onClick={openModal}>Open Modal</button> */}
      <p onClick={openModal}>
        {props.match.player1}, {props.match.player2}
      </p>
      <Modal
        isOpen={modalIsOpen}
        onAfterOpen={afterOpenModal}
        onRequestClose={closeModal}
        style={customStyles}
        ariaHideApp={false}
      >
        {/* <h2 ref={(_subtitle) => (subtitle = _subtitle)}>Hello</h2> */}
        <div>
          Editing match. Who won between {props.match.player1},{' '}
          {props.match.player2}
          <Select
            // value={selectedClient}
            options={options}
            placeholder="Select a person"
            onChange={_handleChange}
          />
        </div>
        <button onClick={commitMatch}>Commit</button>
      </Modal>
    </div>
  )
}

function TournamentBracket(props) {
  console.log(props.b)
  let ret = []
  props.b.forEach((match) => {
    ret.push(
      <div className="flex-item">
        <TournamentMatch match={match} />
      </div>,
    )
  })

  return <div className="flex-container">{ret}</div>
}

export const Tournament = (props) => {
  if (props.info === undefined) {
    return <div>Loading..</div>
  }
  console.log(props.info)

  let tournament = props.info
  let games = tournament.data.Games.reverse()

  let numBrackets = Math.ceil(Math.log2(tournament.tournament.player_count))

  let power = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  for (let i = 0; i < numBrackets; i++) {
    power /= 2
    tournamentBrackets.push(<TournamentBracket b={games.splice(0, power)} />)
  }

  return <div className="tournament">{tournamentBrackets}</div>
}

export default Tournament
