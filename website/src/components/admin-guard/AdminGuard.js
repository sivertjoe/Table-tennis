import { React, Component } from 'react'
import * as AdminApi from '../../api/AdminApi'
import '../../index.css'

class AdminGuard extends Component {
  isAdmin = 0
  constructor() {
    super()
    const token = localStorage.getItem('token')
    if (token) {
      AdminApi.isAdmin(token)
        .then((isAdmin) => {
          if (isAdmin) {
            this.isAdmin = 1
          } else {
            this.isAdmin = -1
          }
        })
        .catch((error) => console.warn(error.message))
        .finally(() => this.setState({}))
    } else this.isAdmin = -1
  }

  render() {
    if (this.isAdmin === 1) {
      return this.props.children
    } else if (this.isAdmin === -1)
      return (
        <div>
          <img className="arnold" alt="STOP!!!" src={'../unauth.png'} />
        </div>
      )
    else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
  }
}

export default AdminGuard
