import { React, useState, useEffect } from 'react'
import * as AdminApi from '../../api/AdminApi'
import '../../index.css'

export default function AdminGuard(props) {
  const [isAdmin, setIsAdmin] = useState(0)

  useEffect(() => {
    const token = localStorage.getItem('token')
    if (token) {
      AdminApi.isAdmin(token)
        .then((isAdmin) => {
          if (isAdmin) {
            setIsAdmin(1)
          } else {
            setIsAdmin(-1)
          }
        })
        .catch((error) => console.warn(error.message))
    } else setIsAdmin(-1)
  }, [])

  if (isAdmin === 1) {
    return props.children
  } else if (isAdmin === -1)
    return (
      <div>
        <img className="arnold" alt="STOP!!!" src={'../unauth.png'} />
      </div>
    )
  else return <h1 style={{ textAlign: 'center' }}>Loading...</h1>
}
