import { React, Component } from 'react'
import './Navbar.css'
import Logo from '../../assets/rankzter_big.png'

class Navbar extends Component {
  menuOpen = false

  constructor() {
    super()
    this.toggleMenu = this.toggleMenu.bind(this)
  }

  toggleMenu() {
    this.menuOpen = !this.menuOpen
    this.setState({})
  }

  render() {
    const username = localStorage.getItem('username')
    return (
      <div className="navbar">
        <button className="hamburger" onClick={this.toggleMenu}>
          <div className="slice"></div>
          <div className="slice"></div>
          <div className="slice"></div>
        </button>
        <a className="logo-box" href="/">
          <img className="logo" alt="Logo" src={Logo} />
        </a>
        <div
          className={'overlay ' + (this.menuOpen ? 'overlay-open' : '')}
          onClick={this.toggleMenu}
        ></div>
        <div className={'menu ' + (this.menuOpen ? 'menu-open' : '')}>
          <div className="items">
            <h2>
              <a href="/match">Match</a>
            </h2>
            <h2>
              <a href="/history">History</a>
            </h2>
            <h2>
              <a href="/stats">Stats</a>
            </h2>
            <h2>
              <a href="/profiles">Profiles</a>
            </h2>
            {username ? (
              <h2>
                <a style={{ color: '#F8A532' }} href={'/profiles/' + username}>
                  {username}
                </a>
              </h2>
            ) : (
              <>
                <h2>
                  <a href="/register">Register</a>
                </h2>
                <h2>
                  <a href="/login">Login</a>
                </h2>
              </>
            )}
          </div>
        </div>
      </div>
    )
  }
}

export default Navbar
