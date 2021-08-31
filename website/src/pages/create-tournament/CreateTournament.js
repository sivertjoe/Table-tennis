import React, { useState } from 'react'
import '../../index.css'
import './CreateTournament.css'
import * as Api from '../../api/TournamentApi'
import ImageUpload from '../../components/image-upload/ImageUpload'
import Button from '../../components/button/Button'
import Input from '../../components/input/Input'
import Select from 'react-select'

const maxSize = 256 * 256

const createTournament = (image, name, numPlayers, type, setInfo, setColor) => {
  setColor('red')
  if (!name) {
    return setInfo('Tournament need a name')
  } else if (numPlayers === undefined || numPlayers < 3) {
    return setInfo('Tournament player count needs to be at least 4')
  }

  Api.createTournament(name, numPlayers, image, type)
    .then(() => {
      setColor('green')
      setInfo('Success!')

      // redirect after success
      window.setTimeout(function () {
        window.location.href = 'tournaments'
      }, 2000)
    })
    .catch((e) => setInfo(e.toString()))
}

const options = [
  { label: 'Single Elimination', value: 'singleElimination' },
  { label: 'Double Elimination', value: 'doubleElimination' },
]

const customStyles = {
  control: (styles, state) => ({
    ...styles,
    width: '100%',
    height: '3rem',
    background: '#232730',
  }),
  menu: (styles, state) => ({
    ...styles,
    background: '#1c1b1b',
  }),
  option: (styles, { data, isDisabled, isFocused, isSelected }) => {
    return {
      ...styles,
      backgroundColor: isFocused ? '#232730;' : '#1c1b1b;',
      color: '#ebf0f2;',
    }
  },
  singleValue: (provided, state) => {
    return { ...provided, color: 'white' }
  },
}

function changeType(e, setType) {
  setType(e.value)
}

export default function CreateTournament() {
  const [name, setName] = useState('')
  const [numPlayers, setNumPlayers] = useState(0)
  const [image, setImage] = useState(false)
  const [info, setInfo] = useState('')
  const [color, setColor] = useState('red')
  const [type, setType] = useState('singleElimination')

  return (
    <div className="container center">
      <h1>Create a tournament</h1>
      <div className="inputs">
        <Input
          style={{ marginBottom: '1rem' }}
          type="text"
          placeholder="Tournament name"
          onChange={setName}
        />
        <Input
          type="number"
          style={{ marginBottom: '1rem' }}
          placeholder="Number of players"
          onChange={setNumPlayers}
        />
        <Select
          defaultValue={options[0]}
          styles={customStyles}
          style={{ height: '30rem' }}
          onChange={(e) => {
            changeType(e, setType)
          }}
          options={options}
        />

        <h2 style={{ margin: '2rem auto 0.5rem auto' }}>Choose prize</h2>
        <p>Leave empty to use default prize</p>
        <ImageUpload onUpload={setImage} maxSize={maxSize} />
        <div
          className="button"
          onClick={() =>
            createTournament(image, name, numPlayers, type, setInfo, setColor)
          }
        >
          <Button placeholder="Create Tournament" />
        </div>
        <p>
          Note: by creating a tournament <i>only you</i> can register games.
          <br />
          To do so, press and hold on the winner in each bracket
        </p>
        {info && <h2 style={{ color: color }}>{info}</h2>}
      </div>
    </div>
  )
}
