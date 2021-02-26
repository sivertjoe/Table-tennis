import * as BaseApi from './BaseApi'

export const getUsers = () => BaseApi.get('users')

export const getUser = (username) => BaseApi.get('user/' + username)

export const getActiveUsers = () => BaseApi.get('active-users')

export const getMultipleUsers = (users) =>
  BaseApi.post('get-multiple-users', {
    users: users,
  })

export const register = (username, password) =>
  BaseApi.post('create-user', {
    username: username,
    password: password,
  })

export const login = (username, password) =>
  BaseApi.post('login', {
    username: username,
    password: password,
  })

export const changePassword = (username, password, newPassword) =>
  BaseApi.post('change-password', {
    username: username,
    password: password,
    new_password: newPassword,
  })

export const requestResetPassword = (name) =>
  BaseApi.post('request-reset-password', { name: name })

export const requestNewName = (name) =>
  BaseApi.post('request-new-name', {
    token: localStorage.getItem('token'),
    new_name: name,
  })
