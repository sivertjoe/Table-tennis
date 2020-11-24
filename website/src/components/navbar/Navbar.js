import { React, Component } from 'react'
import './Navbar.css'
import Logo from '../../assets/rankzter_big.png'

class Navbar extends Component {
  menuOpen = false
  items = [
    { name: 'Match', path: '/match' },
    { name: 'History', path: '/history' },
    { name: 'Profiles', path: '/profiles' },
    { name: 'Register', path: '/register' },
  ]

  constructor() {
    super()
    this.toggleMenu = this.toggleMenu.bind(this)

    if (localStorage.getItem('token'))
      this.items.push({
        name: 'Profile',
        path: '/profiles/' + localStorage.getItem('username'),
      })
    else
      this.items.push({
        name: 'Login',
        path: '/login',
      })
  }

  toggleMenu() {
    this.menuOpen = !this.menuOpen
    this.setState({})
  }

  render() {
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
            {this.items.map((item, i) => (
              <h2 key={i}>
                <a href={item.path}>{item.name}</a>
              </h2>
            ))}
          </div>
        </div>
      </div>
    )
  }
}

export default Navbar
