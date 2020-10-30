import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './HomePage.css'
import Leaderboard from '../../components/leaderboard/Leaderboard'

class HomePage extends Component {
  render() {
    return (
        <Leaderboard />
    )
  }
}

export default HomePage
