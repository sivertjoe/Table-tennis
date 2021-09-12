import React, { useState } from 'react'

import Button from '../../components/button/Button'
import '../../index.css'
import './TournamentMenu.css'

import * as Api from '../../api/TournamentApi'

import { default as TournamentComponenet } from '../../components/tournament/Tournament.js'
import TournamentList from '../../components/tournament-list/TournamentList'

function TournamentMenu() {
  const tabs = ['In progress', 'Old']
  const [activeTab, setActiveTab] = useState(tabs[0])
  const [selectedTournament, setSelectedTournament] = useState(null)
  const [loading, data] = Api.GetTournamentInfos()
  const [info, setInfo] = useState(null)

  // How to deal with this?
  const show = true

  const Tabs = () => (
    <div className="tabs">
      {tabs.map((tab, i) => (
        <button
          key={tab}
          className={
            'tab' +
            (activeTab === tab ? ' selected' : '') +
            (i === 0 ? ' left-round' : '') +
            (i === tabs.length -1 ? ' right-round' : '')
          }
          onClick={() => setActiveTab(tab)}
        >
          {tab}
        </button>
      ))}
    </div>
  )

  const selectNewTournament = (tinfo) => {
    setSelectedTournament(tinfo.name)
    Api.getTournament(tinfo.id)
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
                className={selectedTournament === info.name ? 'orange' : ''}
                onClick={() => selectNewTournament(info)}
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
        /*<span className="arrow" onClick={this.state.goBack}>&#10229;</span>*/
  const TournamentContainer = (data) => (
    //<div className={'body' + (show ? '' : ' hidden')}>
      <div className="tournament-container ">
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

  return (
    <>
      {loading && <h1>Loading..</h1>}
      {!loading && (
        <div className="tournament-grid">
          <Menu info={data} />
          <TournamentContainer info={data} />
        </div>
      )}
    </>
  )
}

export default TournamentMenu
