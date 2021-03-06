import * as BaseApi from './BaseApi'

export const getHistory = () => BaseApi.get('history')

export const getEditHistory = () => BaseApi.get('edit-history')

export const replyToMatch = (match_id, token, ans) =>
  BaseApi.post('respond-to-match', {
    match_notification_id: match_id,
    ans: ans,
    token: token,
  })

export const registerMatch = (winner, loser, token) =>
  BaseApi.post('register-match', {
    winner: winner,
    loser: loser,
    token: token,
  })

export const getSeasonLength = () => BaseApi.get('season_length')

export const getLeaderboardInfo = () => BaseApi.get('leaderboard_info')

export const getStats = (user1, user2) =>
  BaseApi.post('stats', { user1: user1, user2: user2 })
