import { React, Component } from 'react'
import '../../index.css'
import { Badge } from '../badge/Badge'
import images from '../../assets/images'
import * as Api from '../../api/BaseApi.js'

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
        src={images[badge.name] ?? Api.getImageUrl(badge.name)}
        size={this.size}
        placeholder={badge.tooltip}
      />
    ))
  }
}

export default Medals
