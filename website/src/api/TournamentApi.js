import * as BaseApi from './BaseApi'

export const createTournament = (name, playerCount, image) =>
  BaseApi.postImage('create-tournament', {
    name: name,
    organizer_token: localStorage.getItem('token') ?? '',
    player_count: parseInt(playerCount),
    image: image,
  })

export const getTournaments = () => BaseApi.get('tournaments')

export const joinTournament = (id) =>
  BaseApi.post('join-tournament', {
    id: id,
    token: localStorage.getItem('token') ?? '',
  })

export const leaveTournament = (id) =>
  BaseApi.post('leave-tournament', {
    id: id,
    token: localStorage.getItem('token') ?? '',
  })
