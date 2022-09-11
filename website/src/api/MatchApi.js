import * as BaseApi from './BaseApi'

export const getHistory = () => BaseApi.get('history')

export const getEditHistory = () => BaseApi.get('edit-history')

export const registerMatch = (winner, loser, token) =>
  BaseApi.post('register-match', {
    winner: winner,
    loser: loser,
    token: token,
  })

export const getSeasonLength = () => BaseApi.get('season_length')

export const getSeasons = () => BaseApi.get('get-seasons')

export const getLeaderboardInfo = () => BaseApi.get('leaderboard_info')

export const getStats = (user1, user2) =>
  BaseApi.post('stats', { user1: user1, user2: user2 })

export const getSeasonStart = () => BaseApi.get('season_start')
