import { React, Component } from 'react'
import './Navbar.css'

class Navbar extends Component {
  menuOpen = false
  items = [
    { name: 'Home', path: '/home' },
    { name: 'Match', path: '/match' },
    { name: 'History', path: '/history' },
    { name: 'Profiles', path: '/profiles' },
    { name: 'Register', path: '/register' },
  ]

  constructor() {
    super()
    this.renderDesktop = this.renderDesktop.bind(this)
    this.renderMobile = this.renderMobile.bind(this)
    this.toggleMenu = this.toggleMenu.bind(this)
  }

  renderDesktop() {
    return (
      <div className="navbar">
        <div className="items">
          {this.items.map((item, i) => (
            <h2 key={i} className="item">
              <a href={item.path}>{item.name}</a>
            </h2>
          ))}
        </div>
      </div>
    )
  }

  renderMobile() {
    return (
      <div>
        <button className="hamburger" onClick={this.toggleMenu}>
          <div className="slice"></div>
          <div className="slice"></div>
          <div className="slice"></div>
        </button>
          <div className={'overlay ' + (this.menuOpen ? 'overlay-open' : '')} onClick={this.toggleMenu}>
          <div className={'menu ' + (this.menuOpen ? 'menu-open' : '')}>
            <ul className="list">
              {this.items.map((item, i) => (
                <li key={i}>
                  <h2>
                    <a href={item.path}>{item.name}</a>
                  </h2>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    )
  }

  toggleMenu() {
    this.menuOpen = !this.menuOpen
    this.setState({})
  }

  render() {
    const large = window.matchMedia('(min-width: 600px)').matches
    return large ? this.renderDesktop() : this.renderMobile()
  }
}

export default Navbar
