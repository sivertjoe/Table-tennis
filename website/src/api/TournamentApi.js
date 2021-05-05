import * as BaseApi from './BaseApi'

export const createTournament = (name, playerCount, image) =>
  BaseApi.postImage('upload-award', {
    name: name,
    organizer_token: localStorage.getItem('token') ?? '',
    player_count: playerCount,
    image: image,
  })

export const uploadAward = (tournamentId, image) =>
  BaseApi.postImage('upload-award', {
    tournamentId: tournamentId,
    image: image,
  })
