import * as BaseApi from './BaseApi'

export const getNotifications = () =>
  BaseApi.get('notifications?type=match&token=' + localStorage.getItem('token'))

export const getAdminNotifications = (token) =>
  BaseApi.get('notifications?type=admin&token=' + localStorage.getItem('token'))

export const registerMatch = (winner, loser, token) =>
  BaseApi.post('register-match', {
    winner: winner,
    loser: loser,
    token: token,
  })

const respondBase = (id, token, ans, type) => 
  BaseApi.post('notifications', {
    id: id,
    ans: ans,
    token: token,
    type: type
  })

export const replyToMatch = (id, token, ans) => 
    respondBase(id, token, ans, "match")

export const replyToResetPassword = (id, token, ans) =>
    respondBase(id, token, ans, "reset_password")

export const replyToNewUser = (id, token, ans) =>
    respondBase(id, token, ans, "new_user")
