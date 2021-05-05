import React, { useState } from 'react'
import '../../index.css'
import './CreateTournament.css'
import * as Api from '../../api/TournamentApi'
import Button from '../../components/button/Button'
import ImageUpload from '../../components/image-upload/ImageUpload'

const maxSize = 256 * 256

export default function CreateTournament() {
  const [name, setName] = useState('')
  const [numPlayers, setNumPlayers] = useState(0)
  const [image, setImage] = useState('')
  // console.log(image)

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
        <ImageUpload onUpload={setImage} maxSize={maxSize} />
        <p>
          (Note: by creating a tournament <i>only you</i> can register games.
          <br />
          To do so, press and hold on the winner in each bracket)
        </p>
        <div className="button">
          <Button
            placeholder="Create Tournament"
            callback={Api.createTournament(name, numPlayers, image)}
          />
        </div>
      </div>
    </div>
  )
}
