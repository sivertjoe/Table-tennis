import * as BaseApi from './BaseApi'

export const getUsers = () => BaseApi.get('users')

export const getUser = (username) => BaseApi.get('user/' + username)

export const getMatchUsers = () => BaseApi.get('active-users')

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
