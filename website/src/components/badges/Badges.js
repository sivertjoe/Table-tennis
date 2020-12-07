import { React, Component } from 'react'
import './Badges.css'
import '../../index.css'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'

class Leaderboard extends Component {
  constructor(args) {
    super()
    this.user = args.user
    this.size = args.size ?? '16px'
  }

  userBadge(badge, i) {
    return (
      <div key={i} className="box">
        <FontAwesomeIcon
          fixedWidth
          icon={badge.name}
          color={badge.color}
          style={{
            fontSize: this.size,
            stroke: 'black',
            strokeWidth: '24',
          }}
        />
        <div className="badge-info">Season: {badge.season}</div>
      </div>
    )
  }

  render() {
    // TODO: Stack badges when too wide
    return this.user.badges.map((badge, i) => this.userBadge(badge, i))
  }
}

export default Leaderboard
