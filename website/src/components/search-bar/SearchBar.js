import { React, Component } from 'react'
import './SearchBar.css'

class SearchBar extends Component {
  constructor(args) {
    super()
    this.callback = args.callback
    this.search = this.search.bind(this)
    this.placeholder = args.placeholder ?? 'Search'
  }

  search(event) {
    if (this.callback) this.callback(event.target.value)
  }

  render() {
    return (
      <input
        type="text"
        placeholder={this.placeholder}
        onChange={this.search}
      />
    )
  }
}

export default SearchBar
