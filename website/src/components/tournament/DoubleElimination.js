import React from 'react'

import '../../index.css'
import './Tournament.css'
import * as Api from '../../api/TournamentApi'

import Button from '../button/Button'
import Modal from 'react-modal'
import Select from 'react-select'
const SectionContext = React.createContext({
  primary: [],
  setPrimary: () => {},

  secondary: [],
  setSecondary: () => {},

  forward: () => {},
  transelate: () => {},

  finals: [],
  setFinals: () => {},

  info: {},
  setInfo: () => {},

  titles: {},

  tournamentTable: [[]],
})

// const TournamentContext = React.createContext({
//   lower: [],
//   setLower: () => {},

//   upper: [],
//   setUpper: () => {},

//   finals: [],
//   setFinals: () => {},

//   info: {},
// })
function parent_is_empty(bucket) {
  bucket = Math.abs(bucket)
  const biggest_power_of_two = Math.ceil(Math.log(bucket + 2) / Math.log(2))
  const power = Math.pow(2, biggest_power_of_two)

  const bracket_size = power / 4

  return !(power - 2 - bracket_size >= bucket)
}
// function map_from_upper_to_lower(bucket, player_count) {
//   let numBrackets = Math.ceil(Math.log2(player_count))
//   let power = Math.pow(2, numBrackets)
//   const npower = -1 * power
//   const x = power / 4
//   //TODO: fix the commented out if, outer: hotfix
//   // if((power -2 - (power/2)) > -(bucket)){
//   if (bucket < power && bucket >= power / 2 - 1) {
//     let reduced = bucket - power / 2 + 1
//     reduced = reduced / 2
//     const y = Math.trunc(reduced)
//     const p = Number((bucket & 1) === 0)
//     return -(bucket + (x - y - p))
//   } else {
//     let highest = Math.ceil(Math.log2(bucket + 2))
//     let actual_x = Math.pow(2, highest) / 2
//     return -(bucket + actual_x)
//   }
// }

function map_from_upper_to_lower(arr, bucket) {
  console.log('=============', arr)
  let a = arr.find((a) => a[0] === bucket)
  return a
}

function biggest_power_of_two(bucket) {
  let numBrackets = Math.ceil(Math.log2(bucket))
  return Math.pow(2, numBrackets)
}
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

function ForwardUpper(
  primary,
  setPrimary,
  finals,
  setFinals,
  match,
  winner,
  loser,
  info,
  setInfo,
) {
  const matches = primary
  if (match.bucket === biggest_power_of_two(info.player_count)) {
    //need one more match
    if (winner === match.player2) {
      //   const secondFinal = Object.assign({}, finals[0])
      //   secondFinal.bucket++
      console.log(finals)
      finals[1].player1 = finals[0].player1
      finals[1].player2 = finals[0].player2
      //   finals.push(secondFinal)
      console.log(finals)
      setFinals([...finals])
    } else {
      //winner of finale. is done
      setInfo({ ...info, winner: winner })
    }
  } else if (match.bucket === biggest_power_of_two(info.player_count) + 1) {
    setInfo({ ...info, winner: winner })
  } else if (match.bucket !== 0) {
    let parent = Math.trunc((match.bucket - 1) / 2)
    const index = matches.findIndex((m) => m.bucket === parent)
    if ((match.bucket & 1) === 1) {
      matches[index].player1 = winner
      // props.lower[-(idx)].player2 = loser
    } else {
      matches[index].player2 = winner
    }
    setPrimary([...primary]) //Ouch.
  }
  if (match.bucket === 0) {
    finals[0].player1 = winner
    setFinals([...finals])
  }
}
function ForwardToLower(
  secondary,
  setSecondary,
  finals,
  setFinals,
  match,
  winner,
  loser,
  info,
  setInfo,
  tournamentTable,
) {
  const power = biggest_power_of_two(info.player_count)

  if (match.bucket >= power) return

  console.log(tournamentTable[0])
  let lower_index = tournamentTable.find((a) => a[0] === match.bucket)

  const idx = secondary.findIndex((m) => m.bucket == lower_index[1])
  if (lower_index[2] === 1) secondary[idx].player1 = loser
  else secondary[idx].player2 = loser

  //player count must be power, will not work with odd trournamentt
  //   if (match.bucket < power && match.bucket >= power / 2 - 1) {
  //     if ((match.bucket & 1) === 1) {
  //       secondary[idx].player2 = loser
  //       // secondary[-(idx)].player2 = loser
  //     } else {
  //       secondary[idx].player1 = loser
  //     }
  //   } else {
  //     secondary[idx].player2 = loser
  //   }
  setSecondary([...secondary])
}

function ForwardLower(
  primary,
  setPrimary,
  finals,
  setFinals,
  match,
  winner,
  loser,
  info,
  setInfo,
) {
  if (match.bucket === -1) return

  let parent = loser_bracket_parent(match.bucket)
  const index = primary.findIndex((m) => m.bucket === parent)
  if (parent_is_empty(parent, biggest_power_of_two(info.player_count))) {
    if ((match.bucket & 1) === 1) {
      primary[index].player1 = winner
    } else {
      primary[index].player2 = winner
    }
  } else {
    primary[index].player1 = winner
  }
  setPrimary([...primary])
}

function ForwardToUpper(
  secondary,
  setSecondary,
  finals,
  setFinals,
  match,
  winner,
  loser,
  info,
  setInfo,
  a,
) {
  if (match.bucket === -1) {
    //TODO...
    finals[0].player2 = winner
    setFinals([...finals])
  }
}

function TournamentMatch(props) {
  const {
    primary,
    setPrimary,

    secondary,
    setSecondary,

    forward,
    transelate,

    finals,
    setFinals,

    info,
    setInfo,
    titles,
    tournamentTable,
  } = React.useContext(SectionContext)
  const match = props.match
  const [selectedClient, setSelectedClient] = React.useState(undefined)
  const [winner, setWinner] = React.useState('')
  const [loser, setLoser] = React.useState('')
  const [modalIsOpen, setIsOpen] = React.useState(false)
  const options = [
    { value: match.player1, label: match.player1 },
    { value: match.player2, label: match.player2 },
  ]

  function openModal() {
    if (
      // props.winner === '' &&
      match.player1 !== '' &&
      match.player2 !== '' &&
      info.organizer_name === localStorage.getItem('username')
    )
      setIsOpen(true)
  }
  function closeModal() {
    setIsOpen(false)
  }
  function commitMatch() {
    if (!selectedClient) return

    // let winner = selectedClient
    // let loser =
    //   match.player1 === winner ? match.player2 : match.player1
    let l = options.find((l) => l.value !== selectedClient).value
    let w = selectedClient
    // Api.registerTournamentMatch(w, l, props.match.id)
    //   .then(() => closeModal())
    //   .catch((e) => console.warn('Jaha' + e))
    forward(primary, setPrimary, finals, setFinals, match, w, l, info, setInfo)

    transelate(
      secondary,
      setSecondary,
      finals,
      setFinals,
      match,
      w,
      l,
      info,
      setInfo,
      tournamentTable,
    )
    closeModal()
    setWinner(w)
    setLoser(l)
  }
  function _handleChange(event) {
    setSelectedClient(event.value)
  }

  if (winner === '') {
    let numBrackets = Math.ceil(Math.log2(info.player_count))
    let power = Math.pow(2, numBrackets)
    let parent = primary
    let w = ''

    //this is the match that has the parent in the other bracket
    if (match.bucket === -1) {
      parent = secondary
    }

    //this is the match that has the parent in the finals
    if (match.bucket === 0) {
      parent = finals
    }

    //potentially the last match
    if (match.bucket === power) {
      // game is played
      if (finals[1].player2) {
        w = finals[0].player2
      }
      // tournament is over
      else if (info.winner !== '') {
        w = info.winner
      }
      //game is not yet played
      else {
        w = ''
      }
    }
    //definitive last match outcome is defined by the winner
    else if (match.bucket === power + 1) {
      w = info.winner
    } else {
      // winner must be determinated if the player have been forwarded to the parent list
      const parentMatch = parent.find((m) => {
        return m.bucket === match.parent_bucket
      })
      w = [parentMatch?.player1, parentMatch?.player2].filter((m) =>
        [match.player1, match.player2].includes(m),
      )[0]
    }
    if (w !== '') setWinner(w)
  }

  return (
    <>
      <div className="match-info" onClick={openModal}>
        <span
          style={{
            gridColumn: 1,
            color: winner === match.player1 ? 'var(--orange)' : '',
          }}
        >
          {match.player1}
        </span>
        <span style={{ gridColumn: 2 }}>|</span>
        <span
          style={{
            gridColumn: 3,
            color: winner === match.player2 ? 'var(--orange)' : '',
          }}
        >
          {match.player2}
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
            Editing match. Who won between {match.player1} & {match.player2}
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
  const {
    primary,
    // setPrimary,

    secondary,
    // setSecondary,

    // forward,
    // transelate,

    finals,
    // setFinals,

    // info,
    // setInfo
  } = React.useContext(SectionContext)

  let ret = []
  props.slice.forEach((match) => {
    ret.push(
      <div
        className={'match test' + match.bucket}
        key={'match-div-' + match.bucket}
      >
        <TournamentMatch match={match} key={'match-' + match.bucket} />
      </div>,
    )
  })
  return (
    <div className="bracket-container">
      <h2>{props.title}</h2>
      <div
        // className={
        //   'bracket' + ((props.slice[0].bucket === finals[0].bucket ||
        //                 props.slice[props.slice.length-1].bucket === primary[primary.length-1].bucket) ? '' : ' border-right')
        // } //ouch todo sivert gi meg finalen til slutt/ start
        className={'bracket' + (props.last ? '' : ' border-right')} //ouch todo sivert gi meg finalen til slutt/ start
        // key={'bracket-' + props.key}
      >
        {ret}
      </div>
    </div>
  )
}

function UpperBracket(props) {
  const {
    primary,
    // setPrimary,

    secondary,
    // setSecondary,

    // forward,
    // transelate,

    finals,
    // setFinals,

    info,
    titles,
    // setInfo
    tournamentTable,
  } = React.useContext(SectionContext)
  if (!primary || !secondary || !tournamentTable) {
    return <div>Loading..</div>
  }
  let numBrackets = Math.ceil(Math.log2(info.player_count))
  let n_matches = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  let competitors = n_matches
  let start_match = 0

  for (let i = 0; i < numBrackets; i++) {
    n_matches /= 2
    let slice = primary.slice(start_match, start_match + n_matches)
    tournamentBrackets.push(
      <TournamentBracket
        slice={slice}
        // callback={handleInput(tournament.player_count)}
        title={titles[competitors]}
        key={'bracket-upper' + i}
      />,
    )

    competitors /= 2
    start_match += n_matches
  }
  let finalss = finals
  console.log(finalss)
  if (finals[1].player1 === '') {
    finalss = [finals[0]]
  }
  console.log(finalss)
  tournamentBrackets.push(
    <TournamentBracket
      last={true}
      slice={finalss}
      title={titles[0]}
      key={'bracket-upper-f-' + 1}
    />,
  )
  if (info.winner !== '') {
    tournamentBrackets.push(
      <div className="bracket-container" key="winner-bracket">
        <h2>Winner</h2>
        <div className="bracket">
          <div className="match">
            <div className="winner">{info.winner}</div>
          </div>
        </div>
      </div>,
    )
  }
  return (
    <>
      <div
        key="tournament-upper"
        className={props.selected ? 'tournament' : 'inactive-bracket'}
      >
        {tournamentBrackets}
      </div>
    </>
  )
}
function LowerBracket(props) {
  const {
    primary,
    // setPrimary,

    // secondary,
    // setSecondary,

    // forward,
    // transelate,

    // finals,
    // setFinals,

    info,
    // setInfo,
  } = React.useContext(SectionContext)
  if (!primary || !info.player_count) {
    return <div>Loading..</div>
  }
  let numBrackets = Math.ceil(Math.log2(info.player_count))
  let n_matches = Math.pow(2, numBrackets)
  let tournamentBrackets = []
  let competitors = n_matches
  let start_match = 0
  let round = 1
  for (let i = n_matches / 4; i > 0; i >>= 1) {
    for (let j = 0; j < 2; j++) {
      start_match += i * j
      let slice = primary.slice(start_match, start_match + i)

      tournamentBrackets.push(
        <TournamentBracket
          slice={slice}
          title={'Loosers Round ' + round++}
          key={' bracket-lower-' + i + j}
          last={j === 1 && i === 1}
        />,
      )
    }
    start_match += i
  }
  return (
    <>
      <div
        key="tournament"
        className={props.selected ? 'tournament' : 'inactive-bracket'}
      >
        {tournamentBrackets}
      </div>
    </>
  )
}

export const DoubleElimination = (props) => {
  const [isDesktop, setDesktop] = React.useState(window.innerWidth > 1450)

  let numBrackets = Math.ceil(Math.log2(props.info.player_count))
  let power = Math.pow(2, numBrackets)

  const [lower, setLower] = React.useState(
    props.matches.filter((m) => m.bucket < 0).reverse(),
  )

  const [upper, setUpper] = React.useState(
    props.matches.filter((m) => m.bucket >= 0 && m.bucket < power),
  )

  const [finals, setFinals] = React.useState(
    props.matches.filter((m) => m.bucket >= power).reverse(),
  )
  const [info, setInfo] = React.useState(props.info)

  const tabs = ['Upper Bracket', 'Lower Bracket']
  const [activeTab, setActiveTab] = React.useState(tabs[0])

  const [tournamentTable, setTournamentTable] = React.useState(undefined)
  React.useEffect(() => {
    Api.getTournamentTable(info.id)
      .then((t) => setTournamentTable(json(t)))
      .catch((c) => console.warn('au'))
  }, [])
  console.log(typeof tournamentTable)
  const titlesUpper = {
    0: 'Final',
    2: 'Upperfinal',
    4: 'Semifinals',
    8: 'Quarterfinals',
    16: 'Eighth-finals',
    32: '16th-finals',
    64: '32nd-finals',
  }
  let a = [
    [1, 2, 3],
    [1, 2, 3],
    [1, 2, 3],
    [1, 2, 3],
  ]
  const upperSection = {
    primary: upper,
    setPrimary: setUpper,

    secondary: lower,
    setSecondary: setLower,

    forward: ForwardUpper,
    transelate: ForwardToLower,

    finals,
    setFinals,

    info,
    setInfo,
    titles: titlesUpper,

    tournamentTable,
  }

  const lowerSection = {
    primary: lower,
    setPrimary: setLower,

    secondary: upper,
    setSecondary: setUpper,

    forward: ForwardLower,
    transelate: ForwardToUpper,

    finals,
    setFinals,

    info,
    setInfo,
    tournamentTable,
  }
  const Tabs = () => (
    <div className="tabs">
      {tabs.map((tab, i) => (
        <button
          key={tab}
          className={
            'tab' +
            (activeTab === tab ? ' selected' : '') +
            (i === 0 ? ' left-round' : '') +
            (i === tabs.length - 1 ? ' right-round' : '')
          }
          onClick={() => setActiveTab(tab)}
        >
          {tab}
        </button>
      ))}
    </div>
  )

  const selectedTab = (tab) => {
    if (isDesktop) {
      return true
    } else {
      return activeTab === tab
    }
  }

  return (
    <>
      {!isDesktop && <Tabs />}
      <SectionContext.Provider value={upperSection}>
        <UpperBracket selected={selectedTab(tabs[0])} />
      </SectionContext.Provider>

      <SectionContext.Provider value={lowerSection}>
        <LowerBracket selected={selectedTab(tabs[1])} />
      </SectionContext.Provider>
    </>
  )
}
