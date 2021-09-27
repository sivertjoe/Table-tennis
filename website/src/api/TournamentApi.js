import * as BaseApi from './BaseApi'

export const getTournamentTable = (id) => BaseApi.get('tournament-table/' + id)

export const GetTournamentInfos = () => BaseApi.GetHook('tournament-infos')

export const getTournamentInfosToggle = (arg = 'active') =>
  BaseApi.get('tournament-infos?query=' + arg)

export const getTournament = (id) => BaseApi.get('tournament/' + id)

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

export const recreateTournament = (tid) =>
  BaseApi.post('recreate-tournament', {
    token: localStorage.getItem('token') ?? '',
    tid: tid,
  })
