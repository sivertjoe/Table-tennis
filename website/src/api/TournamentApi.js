import * as BaseApi from './BaseApi'

export const createTournament = (name, playerCount, image, type) =>
  BaseApi.postImage('create-tournament', {
    name: name,
    organizer_token: localStorage.getItem('token') ?? '',
    player_count: parseInt(playerCount),
    image: image,
    ttype: type,
  })

export const getTournaments = (arg = '') =>
  BaseApi.get('tournaments?query=' + arg)

export const joinTournament = (id) =>
  BaseApi.post('join-tournament', {
    tid: id,
    token: localStorage.getItem('token') ?? '',
  })

export const leaveTournament = (id) =>
  BaseApi.post('leave-tournament', {
    tid: id,
    token: localStorage.getItem('token') ?? '',
  })

export const registerTournamentMatch = (winner, loser, tournament_game) =>
  BaseApi.post('register-tournament-match', {
    organizer_token: localStorage.getItem('token') ?? '',
    winner: winner,
    loser: loser,
    tournament_game: tournament_game,
  })
export const deleteTournament = (tid) =>
  BaseApi.post('delete-tournament', {
    token: localStorage.getItem('token') ?? '',
    tid: tid,
  })
