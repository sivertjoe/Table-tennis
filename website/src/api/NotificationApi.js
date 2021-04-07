import * as BaseApi from './BaseApi'

export const getNotifications = () =>
  BaseApi.get('notifications?type=match&token=' + localStorage.getItem('token'))

export const getAdminNotifications = (token) =>
  BaseApi.get('notifications?type=admin&token=' + localStorage.getItem('token'))

export const replyToNewUser = (id, token, ans) =>
  BaseApi.post('respond-to-user-notification', {
    id: id,
    ans: ans,
    token: token,
  })

export const replyToResetPassword = (id, token, ans) =>
  BaseApi.post('respond-to-reset-password-notification', {
    id: id,
    ans: ans,
    token: token,
  })
