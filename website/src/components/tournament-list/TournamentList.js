import React, { useState } from 'react'
import '../../index.css'
import * as Api from '../../api/TournamentApi'
import './TournamentList.css'
import Button from '../../components/button/Button'

function removeUser(users, setUsers, setInfo, setColor) {
  const name = localStorage.getItem('username')
  const index = users.indexOf(name)
  if (index < 0) {
    setColor('red')
    setInfo('User not in tournament')
  } else {
    users.splice(index, 1)
    setUsers(users)
    setColor('green')
    setInfo('Left tournament!')
  }
}

function leave(id, users, setUsers, setInfo, setColor) {
  Api.leaveTournament(id)
    .then(() => {
      removeUser(users, setUsers, setInfo, setColor)
    })
    .catch((e) => {
      setColor('red')
      setInfo(e.toString())
    })
}

function join(id, users, setUsers, count, numPlayers, setInfo, setColor) {
  Api.joinTournament(id)
    .then(() => {
      const name = localStorage.getItem('username')
      setUsers([...users, name])
      setColor('green')
      setInfo('Success!')
    })
    .catch((e) => {
      setColor('red')
      setInfo(e.toString())
    })
}

export default function TournamentList(args) {
  const tournament = {
    data: { Players: ['Markus', 'Bernt', 'Ella'] },
    tournament: {
      id: 1,
      name: 'Epic',
      player_count: 8,
      prize: 'assets/tournament_badges/1.png',
      state: 0,
    },
  }
  const count = tournament.data.Players.length
  const id = tournament.tournament.id
  const tournamentName = tournament.tournament.name
  const numPlayers = tournament.tournament.player_count
  const [info, setInfo] = useState('')
  const [color, setColor] = useState('red')
  const [users, setUsers] = useState(tournament.data.Players)

  const list = users.map((name, index) => (
    <tr key={name}>
      <td>{name}</td>
      <td>
        {index + 1}/{numPlayers}
      </td>
    </tr>
  ))
  return (
    <>
      <h1>'{tournamentName}' Participants:</h1>
      <div className="table-container">
        <table>
          <tbody>
            <tr key={0}>
              <th>User</th>
              <th>Player Count</th>
            </tr>
            {list}
          </tbody>
        </table>
      </div>
      <br />

      <div
        className="center"
        onClick={(e) =>
          join(id, users, setUsers, count, numPlayers, setInfo, setColor)
        }
      >
        <Button placeholder="Join" />
      </div>
      <div
        className="center"
        onClick={(e) => leave(id, users, setUsers, setInfo, setColor)}
      >
        <Button placeholder="Leave" />
      </div>
      <div className="center">
        {info && <h2 style={{ color: color }}>{info}</h2>}
      </div>
    </>
  )
}
