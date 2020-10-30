import React from 'react'

const apiUrl = 'http://localhost:58642'

export const getUsers = async () => {
  const req = await fetch(apiUrl + '/users')
  return await req.json()
}

export const getUser = async (username) => {
  const req = await fetch(apiUrl + '/user/' + username)
  return await req.json()
}
