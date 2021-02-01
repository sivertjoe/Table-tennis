import React, { Component } from 'react'
import * as AdminApi from '../../../api/AdminApi'
import '../../../index.css'

class EditSeason extends Component {
  seasonLength = -1 //
  constructor() {
    super()
    const token = localStorage.getItem('token')
    if (token) {
      AdminApi.isAdmin(token).then((isAdmin) => {
        if (isAdmin) {
          this.isAdmin = 1
          this.setState({})
        } else {
          this.isAdmin = -1
          this.setState({})
        }
      })
    } else this.isAdmin = -1
      this.isAdmin = 1
  }

  render() {
    if (this.isAdmin === 1) {
      return <h1>Edit Season</h1>
    } else if (this.isAdmin === -1)
      return (
        <div>
          <img className="arnold" alt="STOP!!!" src={'../unauth.png'} />
        </div>
      )
    else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
  }
}

export default EditSeason
