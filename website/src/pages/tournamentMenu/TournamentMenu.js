import React, { useState, useEffect } from 'react'
import Button from '../../components/button/Button'
import '../../index.css'
import './TournamentMenu.css'

import * as Api from '../../api/TournamentApi'

import { default as TournamentComponenet } from '../../components/tournament/Tournament.js'
import TournamentList from '../../components/tournament-list/TournamentList'
import { useLocation } from 'react-router'

function TournamentMenu(props) {
  const tabs = ['In progress', 'Old']
  const [activeTab, setActiveTab] = useState(tabs[0])
  const [tournamentList, setTournamentList] = useState(undefined)
  const [selectedTournament, setSelectedTournament] = useState(undefined)
  const location = useLocation()
  const [isDesktop, setDesktop] = useState(window.innerWidth > 1450)
  useEffect(() => {
    const updateTInfo = (state) =>
      Api.getTournamentInfosToggle(state)
        .then((tournaments) => {
          setTournamentList(tournaments)
        })
        .catch('could not get tournamentlist')

    const id = new URLSearchParams(props.location.search).get('match')
    if (id === selectedTournament?.tournament.id) return
    if (id) {
      Api.getTournament(id)
        .then((tournament) => {
          let state = tournament.tournament.state === 2 ? 'old' : 'active'
          let tab = tabs[tournament.tournament.state === 2 ? 1 : 0]
          setSelectedTournament(tournament)
          if (tab !== activeTab || !tournamentList) updateTInfo(state)
          if (tab !== activeTab) setActiveTab(tab)
        })
        .catch((err) => console.warn('jaha' + err))
    } else {
      setSelectedTournament(undefined)
      let state = activeTab === tabs[1] ? 'old' : 'active'
      updateTInfo(state)
    }
  }, [location.search])

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

    setSelectedTournament(undefined)
    props.history.push('/tournaments')
    Api.getTournamentInfosToggle(arg)
      .then((tournaments) => setTournamentList(tournaments))
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

  const selectTournament = (tournamentListItem) => {
    const id = new URLSearchParams(props.location.search).get('match')

    if (id === tournamentListItem?.id) return
    props.history.push({
      search: '?match=' + tournamentListItem?.id,
    })
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
                  selectedTournament?.tournament.id === info.id ? 'orange' : ''
                }
                onClick={() => selectTournament(info)}
              >
                {/* // setSelectedTournament(info)} */}
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
      {selectedTournament?.tournament.name ? (
        selectedTournament.tournament.state > 0 ? (
          <TournamentComponenet
            matches={selectedTournament.data.Games}
            info={selectedTournament.tournament}
            table={selectedTournament.table}
          />
        ) : (
          <TournamentList
            players={selectedTournament.data.Players}
            tournament={selectedTournament.tournament}
          />
        )
      ) : (
        isDesktop && (
          <>
            <h1>No tournament selected...</h1>
          </>
        )
      )}
    </div>
    //</div>
  )

  if (isDesktop) {
  }

  return (
    <>
      {!tournamentList && <h1>Loading..</h1>}
      {tournamentList && (
        <div className="tournament-grid">
          {isDesktop && (
            <>
              <Menu info={tournamentList} history={props.history} />
              <TournamentContainer info={tournamentList} />
            </>
          )}

          {!isDesktop && (
            <>
              {!selectedTournament?.tournament.name ? (
                <Menu info={tournamentList} history={props.history} />
              ) : (
                <>
                  <Arrow />
                  <TournamentContainer info={tournamentList} />
                </>
              )}
            </>
          )}
        </div>
      )}
    </>
  )
}

export default TournamentMenu
