import { React, Component } from 'react'

class ProfilePage extends Component {
  render() {
    return (
      <div>
        <p>Profile</p>
        <p>{this.props.match.params.id}</p>
      </div>
    )
  }
}

export default ProfilePage
