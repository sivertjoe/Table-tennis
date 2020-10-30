import { React, Component } from 'react'
import './Navbar.css'

class Navbar extends Component {
  render() {
    return (
      <div class="navbar">
        <div class="items">
          <h2 class="item">
            <a href="/profiles">Profiles</a>
          </h2>
          <h2 class="item">
            <a href="/">Home</a>
          </h2>
        </div>
      </div>
    )
  }
}

export default Navbar
