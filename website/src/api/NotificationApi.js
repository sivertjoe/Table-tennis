import * as BaseApi from './BaseApi'

export const getNotifications = () =>
  BaseApi.get('notifications/' + localStorage.getItem('token'))

export const getAdminNotifications = (token) =>
  BaseApi.post('admin-notifications', {
    token: token,
  })

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

export const replyToNewName= (id, token, ans) =>
  BaseApi.post('respond-to-new-name-notification', {
    id: id,
    ans: ans,
    token: token,
  })
