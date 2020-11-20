const url =
  process.env.NODE_ENV === 'prod' ? 'https://sivert.dev' : 'http://localhost'
const ip = '58642'
const apiUrl = url + ':' + ip

export const getUsers = async () =>
  fetch(apiUrl + '/users')
    .then((req) => req.json())
    .then((res) => res)

export const getAllUsers = async () =>
  fetch(apiUrl + '/all-users/' + localStorage.getItem('token'))
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

export const replyToMatch = async (match_id, token, ans) =>
  fetch(apiUrl + '/respond-to-match', {
    method: 'POST',
    body: JSON.stringify({
      match_notification_id: match_id,
      ans: ans,
      user_token: token,
    }),
  }).then((req) => req)

export const registerMatch = async (winner, loser, token) =>
  fetch(apiUrl + '/register-match', {
    method: 'POST',
    body: JSON.stringify({
      winner: winner,
      loser: loser,
      token: token,
    }),
  }).then((req) => req)

export const login = async (username, password) =>
  fetch(apiUrl + '/login', {
    method: 'POST',
    body: JSON.stringify({ username: username, password: password }),
  }).then((req) => req)

export const getNotifications = async () =>
  fetch(apiUrl + '/notifications/' + localStorage.getItem('token'))
    .then((req) => req.json())
    .then((req) => req)

export const changePassword = async (username, password, newPassword) =>
  fetch(apiUrl + '/change-password', {
    method: 'POST',
    body: JSON.stringify({
      username: username,
      password: password,
      new_password: newPassword,
    }),
  }).then((req) => req)

export const isAdmin = async (token) =>
  fetch(apiUrl + '/is-admin/' + token)
    .then((req) => req.json())
    .then((res) => res)

export const getNewUserNotification = async (token) =>
  fetch(apiUrl + '/user-notification/' + token)
    .then((req) => req.json())
    .then((res) => res)

export const replyToNewUser = async (id, token, ans) =>
  fetch(apiUrl + '/respond-to-user-notification', {
    method: 'POST',
    body: JSON.stringify({
      id: id,
      ans: ans,
      token: token,
    }),
  }).then((req) => req)

export const editUsers = async (users, action) =>
  fetch(apiUrl + '/edit-users', {
    method: 'POST',
    body: JSON.stringify({
      users: users,
      action: action,
      token: localStorage.getItem('token') ?? '',
    }),
  }).then((req) => req)
