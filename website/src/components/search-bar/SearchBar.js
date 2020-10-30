import { React, Component } from 'react'
import './SearchBar.css'

class SearchBar extends Component {
  callback = null

  constructor(args) {
    super()
    this.callback = args.callback
    this.search = this.search.bind(this)
  }

  search(event) {
    this.callback(event.target.value)
  }

  render() {
    return (
      <input type="text" placeholder="Search" onChange={this.search} />
    )
  }
}

export default SearchBar
