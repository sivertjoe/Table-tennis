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

export const register = async (username) =>
  fetch(apiUrl + '/create-user/' + username, {
    method: 'POST',
  }).then((req) => req.status === 200)

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
