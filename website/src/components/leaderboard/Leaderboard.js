import { React, Component } from 'react'
import * as Api from '../../api/Api'

class Leaderboard extends Component {
  render() {
        Api.getUsers().then((users) => (this.users = users));
      return
      <h1>Leaderboard</h1>
        {this.users.map(user => {
            <p>user.name</p>
        })}
    }
}

export default Leaderboard

