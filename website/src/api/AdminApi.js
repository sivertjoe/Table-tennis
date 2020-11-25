import * as BaseApi from './BaseApi'

export const isAdmin = (token) => BaseApi.get('is-admin/' + token)

export const getAllUsers = () =>
  BaseApi.get('all-users/' + localStorage.getItem('token'))

export const editUsers = (users, action) =>
  BaseApi.post('edit-users', {
    users: users,
    action: action,
    token: localStorage.getItem('token') ?? '',
  })

export const rollBack = () => 
    BaseApi.get('admin/roll-back/' + localStorage.getItem('token'))
