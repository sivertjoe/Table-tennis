import * as BaseApi from './BaseApi'

export const getNotifications = () =>
  BaseApi.get('notifications/' + localStorage.getItem('token'))

export const getNewUserNotification = (token) =>
  BaseApi.get('user-notification/' + token)


export const replyToNewUser = (id, token, ans) =>
  BaseApi.post('respond-to-user-notification', {
    id: id,
    ans: ans,
    token: token,
  })
