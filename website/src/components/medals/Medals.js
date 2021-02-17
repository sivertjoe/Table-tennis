import { React, Component } from 'react'
import '../../index.css'
import { Badge } from '../badge/Badge'
import images from '../../assets/images'

class Medals extends Component {
  constructor(args) {
    super()
    this.user = args.user
    this.size = args.size ?? '16px'
  }

  render() {
    // TODO: Stack badges when too wide
    return this.user.badges.map((badge, i) => (
      <Badge
        key={i}
        i={i}
        src={images[badge.name]}
        size={this.size}
        placeholder="placeholder"
      />
    ))
  }
}

export default Medals
