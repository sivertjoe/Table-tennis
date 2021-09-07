import React from 'react'

import '../../index.css'
import './Tournament.css'
import * as Api from '../../api/TournamentApi'

import Button from '../button/Button'
import Modal from 'react-modal'
import Select from 'react-select'
import DeleteTournament from '../../components/delete-tournament/DeleteTournament'
// import { defaultProps } from 'react-select/src/Select'

// fn loser_bracket_parent(&self, bucket: i64, player_count: i64) -> i64
// {
//     let bucket = bucket.abs();
//     let biggest_power_of_two = (((bucket + 2) as f64).ln() / 2.0_f64.ln()).ceil() as u32;
//     let power = 2_i64.pow(biggest_power_of_two);

//     let bracket_size = power / 4;
//     let x = bracket_size - 1;

//     let parent = |n: i64| (((n - 1) as f64) / 2.0).ceil() as i64;

//     if power - 2 - bracket_size > bucket
//     {
//         -(parent(bucket - x) + x)
//     }
//     else
//     {
//         -(bucket - bracket_size)
//     }
// }
//stolen from server/server/src/tournament.rs
// function loser_bracket_parent(bucket) {
//   bucket = Math.abs(bucket)
//   let numBrackets = Math.ceil(Math.log2(bucket + 2))
//   let n_matches = Math.pow(2, numBrackets)

//   const bracket_size = n_matches / 4
//   const x = bracket_size - 1
//   const parent = (n) => Math.ceil((n - 1) / 2)
//   if (n_matches - 2 - bracket_size > bucket) {
//     return -(parent(bucket - x) + x)
//   } else {
//     return -(bucket - bracket_size)
//   }
// }

function loser_bracket_parent(bucket) {
  bucket = Math.abs(bucket)
  let numBrackets = Math.ceil(Math.log2(bucket + 2))
  let n_matches = Math.pow(2, numBrackets)

  const bracket_size = n_matches / 4
  const x = bracket_size - 1
  const parent = (n) => Math.ceil((n - 1) / 2)
  if (n_matches - 2 - bracket_size > bucket) {
    return -(parent(bucket - x) + x)
  } else {
    return -(bucket - bracket_size)
  }
}

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
    if (match.bucket === -1) {
      let parent =props.upper[0]
      return ![parent.player1, parent.player2].filter((m) =>
      [match.player1, match.player2].includes(m),
    ).length
    }
    
    
   

    let parentIndex = loser_bracket_parent(match.bucket)
    let parent = matches.find((m) => {
      return parentIndex === m.bucket
    })

    return ![parent.player1, parent.player2].filter((m) =>
      [match.player1, match.player2].includes(m),
    ).length
  }
  function openModal() {
    if (
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
    // Api.registerTournamentMatch(winner, loser, props.match.id)
    //   .then(() => closeModal())
    //   .catch((e) => console.warn('Jaha' + e))
    props.callback(props.match, winner)
    closeModal()
  }

  function _handleChange(event) {
    setSelectedClient(event.value)
  }

  let winner = props.winner
  if (props.match.bucket < -1) {
    let parentIndex = loser_bracket_parent(props.match.bucket)
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
      <div className={'match test-' + matches[i].bucket} key={'match-div-' + i}>
        <TournamentMatch
          match={matches[i]}
          matches={matches}
          key={'match' + { i }}
          callback={props.callback}
          organizer={props.organizer}
          winner={props.winner}
          upper = {props.upper}
          setUpper = {props.setUpper}
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


function biggest_power_of_two(bucket) {
  let numBrackets = Math.ceil(Math.log2(bucket))
  return Math.pow(2, numBrackets)
}
function parent_is_empty(bucket) {
  bucket = Math.abs(bucket)
  const biggest_power_of_two = Math.ceil((Math.log(bucket + 2) / Math.log(2)))
  const power = Math.pow(2, biggest_power_of_two)

  const bracket_size = power / 4
  
  return !(power - 2 - bracket_size >= bucket)
}
export const LowerBracket = (props) => {
  const [matches, setMatches] = React.useState([...props.matches])
  const [tournament, setInfo] = React.useState(props.info)
  console.log('aaaaaaaaaaaaaa', matches)
  //matches would not rerender when parent rerender (:
  if (tournament.id !== props.info.id) {
    setInfo(props.info)
    setMatches([...props.matches])
  }
  if (!matches) {
    return <div>Loading..</div>
  }

  function handleInputChange(match, winner) {
    if (match.bucket === -1) {
      //tournament.winner = winner
      props.upper[0].player2 = winner
      props.setUpper([...props.upper])
    } else {
      let parent = loser_bracket_parent(match.bucket)
      const index = matches.findIndex((m) => m.bucket === parent)
      //identifisere "empty"
      //odd/even

      //else putte den i player 1

      //edge første gang

      // Empty bucket
      // TODO:
      if (parent_is_empty(parent, biggest_power_of_two(tournament.player_count))) {
        if ((match.bucket & 1) === 1) {
          matches[index].player1 = winner
        } else {
          matches[index].player2 = winner
        }
      } else {
        matches[index].player1 = winner
      }
      // if ((match.bucket & 1) === 1) {
      
      //TODO: we have 3 cases of forward: u-u, u-l, l-l, but we also have non trivial l-l(empty)
    }

    setMatches([...matches]) // Ouch.
  }
  let numBrackets = Math.ceil(Math.log2(tournament.player_count))
  let n_matches = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  let start_match = 0
  let competitors = n_matches
  const id = tournament.id
  const organizerName = tournament.organizer_name
  const name = localStorage.getItem('username')
  //TODO: fix the lower buckets
  for (let i = n_matches / 4; i > 0; i >>= 1) {
    for (let j = 0; j < 2; j++) {
      start_match += i * j
      console.log('start_match ', start_match)
      console.log('start_match+i ', start_match + i)
      tournamentBrackets.push(
        <TournamentBracket
          start={start_match}
          stop={start_match + i}
          matches={matches}
          callback={handleInputChange}
          organizer={tournament.organizer_name}
          title={finals[competitors]}
          key={' bracket' + i}
          winner={tournament.winner}
          upper = {props.upper}
          setUpper = {props.setUpper}
        />,
      )
    }
    start_match += i
  }
  // function setwinner(=(winner))=>setupper([...upper,upper[winner]])
  // for (let i = 0; i < numBrackets; i++) {
  //   n_matches /= 2
  //   tournamentBrackets.push(
  //     <TournamentBracket
  //       start={start_match}
  //       stop={start_match + n_matches}
  //       matches={matches}
  //       callback={handleInputChange}
  //       organizer={tournament.organizer_name}
  //       title={finals[competitors]}
  //       key={' bracket' + i}
  //       winner={tournament.winner}
  //     />,
  //   )
  //   competitors /= 2
  //   start_match += n_matches
  // }
  //   if (tournament.winner) {
  // if (tournament.winner !== '') {
  //   tournamentBrackets.push(
  //     <div className="bracket-container" key="winner-bracket">
  //       <h2>Winner</h2>
  //       <div className="bracket">
  //         <div className="match">
  //           <div className="winner">{tournament.winner}</div>
  //         </div>
  //       </div>
  //     </div>,
  //   )
  // }

  return (
    <>
      <div key="tournament" className="tournament">
        {tournamentBrackets}
      </div>
    </>
  )
}
