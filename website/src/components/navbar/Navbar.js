import { React, Component } from 'react'
import './Navbar.css'

class Navbar extends Component {
  render() {
    const large = window.matchMedia('(min-width: 600px)').matches
    return (
      <div className="navbar">
        <div className="items">
          <h2 className={`item ${large ? '' : 'small'}`}>
            <a href="/">Home</a>
          </h2>
          <h2 className={`item ${large ? '' : 'small'}`}>
            <a href="/match">Match</a>
          </h2>
          <h2 className={`item ${large ? '' : 'small'}`}>
            <a href="/history">History</a>
          </h2>
          <h2 className={`item ${large ? '' : 'small'}`}>
            <a href="/profiles">Profiles</a>
          </h2>
          <h2 className={`item ${large ? '' : 'small'}`}>
            <a href="/register">Register</a>
          </h2>
        </div>
      </div>
    )
  }
}

export default Navbar
