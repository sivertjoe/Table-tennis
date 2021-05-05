import React, { useState } from 'react'
import '../../index.css'
import './CreateTournament.css'
import * as TournamentApi from '../../api/TournamentApi'
import Button from '../../components/button/Button'

const maxSize = 256 * 256

function UploadImage(event, setImage, setError) {
  if (event.target.files[0]) {
    if (event.target.files[0].size > maxSize) {
      setError('Image is too large: max size is ' + maxSize)
    } else {
      setImage(event.target.files[0])
      setError('')
    }
  }
}

export default function CreateTournament() {
  const [name, setName] = useState('')
  const [numPlayers, setNumPlayers] = useState(0)
  const [image, setImage] = useState('')
  const [error, setError] = useState('')
    console.log(image)

  return (
    <div className="container">
      <h1>Create a tournament</h1>
      <div className="inputs">
        <input
          className="input"
          type="text"
          placeholder="Tournament name"
          onChange={(e) => setName(e.target.value)}
        />
        <br />
        <br />
        <input
          className="input"
          type="number"
          placeholder="Number of players"
          onChange={(e) => setNumPlayers(e.target.value)}
        />
          <br />
          <br />
        <input
          type="file"
          accept="image/png"
          onChange={(e) => UploadImage(e, setImage, setError)}
          maxLength={maxSize}
        />
        {error && <p style={{ color: 'red' }}>{error}</p>}
        <p>
          (Note: by creating a tournament <i>only you</i> can register games.
        </p>
        <p>To do so, press and hold on the winner in each bracket)</p>
        <div className="button">
            <Button placeholder="Create Tournament" callback={() => TournamentApi.createTournament(name, numPlayers, image)}/>
        </div>
      </div>
    </div>
  )
}
