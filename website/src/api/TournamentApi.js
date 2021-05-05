import * as BaseApi from './BaseApi'

export const createTournament = (name, playerCount, image) =>
  BaseApi.postImage('create-tournament', {
    name: name,
    organizer_token: localStorage.getItem('token') ?? '',
    player_count: parseInt(playerCount),
    image: image,
  })
