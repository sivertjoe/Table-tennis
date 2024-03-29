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

export const editMatch = (newWinner, newLoser, epoch, id) =>
  BaseApi.post('edit-match', {
    winner: newWinner,
    loser: newLoser,
    epoch: epoch,
    token: localStorage.getItem('token'),
    id: id,
  })

export const deleteMatch = (id) =>
  BaseApi.post('delete-match', {
    id: id,
    token: localStorage.getItem('token'),
  })

export const setSeasonLength = (num) =>
  BaseApi.post('season_length', {
    new_val: num,
    token: localStorage.getItem('token'),
  })

export const stopSeason = () =>
  BaseApi.post('stop_season', {
    token: localStorage.getItem('token'),
  })

export const startSeason = () =>
  BaseApi.post('start_season', {
    token: localStorage.getItem('token'),
  })

export const cancelSeason = () =>
    BaseApi.post('cancel_season', {
        token: localStorage.getItem('token'),
    })

export const executeSql = (str) =>
  BaseApi.post('admin/execute-sql', {
    token: localStorage.getItem('token'),
    command: str,
  })

export const getVariable = (variable) =>
  BaseApi.post('admin/get-variable', {
    variable: variable,
  })

export const setVariable = (variable, val) =>
  BaseApi.post('admin/set-variable', {
    variable: variable,
    new_val: val,
    token: localStorage.getItem('token'),
  })
