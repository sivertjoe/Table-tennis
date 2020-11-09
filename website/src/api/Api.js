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

export const getNotifications = async () =>
  fetch(apiUrl + '/notifications/' + '2501b80e-45c2-4de8-894a-ca950b7ba638')
    .then((req) => req.json())
    .then((req) => req)

export const replyToMatch = async (match_id, token, ans) =>
  fetch(apiUrl + '/respond-to-match', {
    method: 'POST',
    body: JSON.stringify({
      match_notification_id: match_id,
      ans: ans,
      user_token: token,
    }),
  }).then((req) => req)

export const registerMatch = async (winner, loser) =>
  fetch(apiUrl + '/register-match', {
    method: 'POST',
    body: JSON.stringify({ winner: winner, loser: loser }),
  }).then((req) => req)
