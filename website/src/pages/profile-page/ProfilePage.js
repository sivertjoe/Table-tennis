import { React, Component } from 'react'
import * as Api from '../../api/Api'
import './ProfilePage.css'

class Profile extends Component {
  constructor(user) {
    super()
    this.username = user.user
  }

  render() {
    Api.getUser(this.username).then((user) => (this.user = user))
      return <div>{this.user}</div>
  }
}

class Profiles extends Component {
  render() {
    Api.getUsers().then((x) => console.log(x))
    return (
      <div>
        <p>All profiles</p>
      </div>
    )
  }
}

class ProfilePage extends Component {
  render() {
    return this.props.match.params.user ? (
      <Profile user={this.props.match.params.user} />
    ) : (
      <Profiles />
    )
  }
}

export default ProfilePage
