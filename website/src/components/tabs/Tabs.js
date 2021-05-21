import { React, Component } from 'react'
import './Tabs.css'

class Tabs extends Component {
  selectedTab = 0

  constructor(args) {
    super()
    this.args = args
    this.tabs = args.tabs ?? []
    this.onClick = this.onClick.bind(this)
  }

  onClick(i) {
    if (this.args.onSelectTab) this.args.onSelectTab(i)
    this.selectedTab = i
    this.setState({})
  }

  render() {
    return (
      <div className="tabs">
        {this.tabs.map((tab, i) => (
          <button
            key={i}
            className={
              'tab' +
              (this.selectedTab === i ? ' selected' : '') +
              (i === 0 ? ' left-round' : '') +
              (i === this.tabs.length ? ' right-round' : '')
            }
            onClick={() => this.onClick(i)}
          >
            {tab}
          </button>
        ))}
      </div>
    )
  }
}

export default Tabs
