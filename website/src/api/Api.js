const apiUrl = 'http://localhost:58642'

export const getUsers = async () =>
  fetch(apiUrl + '/users')
    .then((req) => req.json())
    .then((res) => res)

export const getUser = async (username) =>
  fetch(apiUrl + '/user/' + username)
    .then((req) => req.json())
    .then((res) => res)

export const getHistory = async (username) =>
  fetch(apiUrl + '/history')
    .then((req) => req.json())
    .then((res) => res)

export const register = async (username, password) =>
  fetch(apiUrl + '/create-user', {
    method: 'POST',
    body: JSON.stringify({ username: username, password: password }),
  }).then((req) => req)

export const registerMatch = async (winner, loser, epoch) => {
  const req = await fetch(
    apiUrl +
      '/register-match?winner=' +
      winner +
      '&loser=' +
      loser +
      '&epoch=' +
      epoch,
    {
      method: 'POST',
    },
  )

  return await req
}
