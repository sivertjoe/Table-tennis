import React, { useState } from 'react'
import '../../index.css'
import './CreateTournament.css'
import * as Api from '../../api/TournamentApi'
import ImageUpload from '../../components/image-upload/ImageUpload'
import images from '../../assets/images'

const maxSize = 256 * 256

function srcToFile(src, fileName, mimeType) {
  return fetch(src)
    .then(function (res) {
      return res.arrayBuffer()
    })
    .then(function (buf) {
      return new File([buf], fileName, { type: mimeType })
    })
}

function getDefaultTouramentPrize() {
  return srcToFile(images['tournament.png'], 'image.png', 'image/png')
}

function createTournament(image, name, numPlayers, setInfo, setColor) {
  setColor('red')
  if (!name) {
    setInfo('Tournament need a name')
    return
  } else if (numPlayers === undefined || numPlayers < 3) {
    setInfo('Tournament player count needs to be at least 4')
    return
  }

  if (typeof image.name !== 'string') {
    getDefaultTouramentPrize().then((newImage) => {
      Api.createTournament(name, numPlayers, newImage)
        .then(() => {
          setColor('green')
          setInfo('Success!')
        })
        .catch((e) => console.warn('Jaha' + e))
    })

    return
  }

  Api.createTournament(name, numPlayers, image)
    .then(() => {
      setColor('green')
      setInfo('Success!')
    })
    .catch((e) => console.warn('Jaha' + e))
}

export default function CreateTournament() {
  const [name, setName] = useState('')
  const [numPlayers, setNumPlayers] = useState(0)
  const [image, setImage] = useState([])
  const [info, setInfo] = useState('')
  const [color, setColor] = useState('red')

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
        <p>Choose prize, or don't upload to use the default</p>
        <ImageUpload onUpload={(i) => setImage(i)} maxSize={maxSize} />
        <p>
          (Note: by creating a tournament <i>only you</i> can register games.
          <br />
          To do so, press and hold on the winner in each bracket)
        </p>
        <div className="button">
          <input
            value="Create Tournament"
            className="big-button"
            type="button"
            onClick={() =>
              createTournament(image, name, numPlayers, setInfo, setColor)
            }
          ></input>
        </div>
        {info && <p style={{ color: color }}>{info}</p>}
      </div>
    </div>
  )
}
