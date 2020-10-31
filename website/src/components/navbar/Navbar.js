import { React, Component } from 'react'
import './Navbar.css'

class Navbar extends Component {
  render() {
    return (
      <div className="navbar">
        <div className="items">
          <h2 className="item">
            <a href="/">Home</a>
          </h2>
          <h2 className="item">
            <a href="/match">Match</a>
          </h2>
          <h2 className="item">
            <a href="/profiles">Profiles</a>
          </h2>
          <h2 className="item">
            <a href="/register">Register</a>
          </h2>
        </div>
      </div>
    )
  }
}

export default Navbar
