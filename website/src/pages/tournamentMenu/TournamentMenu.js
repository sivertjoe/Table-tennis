import React, { useState, useEffect } from 'react'
import Button from '../../components/button/Button'
import '../../index.css'
import './TournamentMenu.css'

import * as Api from '../../api/TournamentApi'

import { default as TournamentComponenet } from '../../components/tournament/Tournament.js'
import TournamentList from '../../components/tournament-list/TournamentList'
import { Route, Router } from 'react-router-dom'
import { useHistory } from 'react-router-dom'
import { useLocation } from 'react-router-dom'
import { useParams } from 'react-router-dom'
function TournamentMenu(props) {
  const tabs = ['In progress', 'Old']
  const [activeTab, setActiveTab] = useState(tabs[0])
  const [selectedTournament, setSelectedTournament] = useState(undefined)
  const [data, setData] = useState(undefined)
  //   const [loading, data] = Api.GetTournamentInfos()
  const [info, setInfo] = useState(undefined)
  const [isDesktop, setDesktop] = useState(window.innerWidth > 1450)
  //   let h = useHistory()
  //   let l = useLocation()
  //   let { match } = useParams()

  //   useEffect(() => {
  //     console.log(h)
  //   }, [h])
  //   useEffect(() => {
  //     console.log(l)
  //   }, [l])

  //   useEffect(() => {
  //     console.log(match)
  //   }, [match])
  useEffect(() => {
    Api.getTournamentInfosToggle('active')
      .then((data) => setData(data))
      .catch('ahaj')
  }, [])

  useEffect(() => {
    if (data) {
      const match = new URLSearchParams(props.location.search).get('match')
      setSelectedTournament(data.find((m) => m.id == match))
    }
  }, [data, props.location.search])

  useEffect(() => {
    if (selectedTournament) {
      props.history.push({ search: '?match=' + selectedTournament?.id })
      selectNewTournament(selectedTournament)
    }
  }, [selectedTournament, props.history])

  const updateMedia = () => {
    setDesktop(window.innerWidth > 1450)
  }
  useEffect(() => {
    window.addEventListener('resize', updateMedia)
    return () => window.removeEventListener('resize', updateMedia)
  })

  const ToggleTournamentStatus = (tab) => {
    let arg = ''
    if (tab === tabs[1]) arg = 'old'
    else arg = 'active'

    Api.getTournamentInfosToggle(arg)
      .then((data) => setData(data))
      .catch('ahaj')

    setActiveTab(tab)
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
          onClick={() => ToggleTournamentStatus(tab)}
        >
          {tab}
        </button>
      ))}
    </div>
  )

  const selectNewTournament = (tinfo) => {
    Api.getTournament(tinfo?.id)
      .then((info) => {
        setInfo(info)
      })
      .catch((err) => console.log('jaha' + err))
  }

  const Menu = (props) => (
    <div className={'tournament-menu'}>
      <Tabs />
      <div className="table-container">
        <table>
          <tbody>
            {props.info.map((info, i) => (
              <tr
                key={i}
                className={
                  selectedTournament?.name === info.name ? 'orange' : ''
                }
                onClick={() => setSelectedTournament(info)}
              >
                <td>{info.name}</td>
                <td>
                  {info.player_count === info.num_players
                    ? info.player_count
                    : info.num_players + '/' + info.player_count}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <Button
        placeholder="Create"
        callback={() => (window.location.href = '/create-tournament')}
      />
    </div>
  )

  const Arrow = () => {
    return (
      <span
        className="arrow"
        onClick={() => {
          if (props.history.length > 1) {
            setSelectedTournament(undefined)
            setInfo(undefined)
            props.history.push('/tournaments')
          } else {
            props.history.goBack()
          }
        }}
      >
        &#10229;
      </span>
    )
  }

  const TournamentContainer = (data) => (
    <div className={'tournament-container'}>
      {info ? (
        info.tournament.state > 0 ? (
          //   <div className="center">
          //   {name === organizerName && <DeleteTournament id={id} />}
          // </div>
          <TournamentComponenet
            matches={info.data.Games}
            info={info.tournament}
          />
        ) : (
          <TournamentList
            players={info.data.Players}
            tournament={info.tournament}
          />
        )
      ) : (
        <h1>No tournament selected...</h1>
      )}
    </div>
    //</div>
  )

  if (isDesktop) {
  }

  return (
    <>
      {!data && <h1>Loading..</h1>}
      {data && (
        <div className="tournament-grid">
          {isDesktop && <Menu info={data} history={props.history} />}

          {!isDesktop && (
            <>
              {!selectedTournament ? (
                <Menu info={data} history={props.history} />
              ) : (
                <Arrow />
              )}
            </>
          )}
          <TournamentContainer info={data} />
        </div>
      )}
    </>
  )
}

export default TournamentMenu
