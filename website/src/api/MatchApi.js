import * as BaseApi from './BaseApi'

export const getHistory = () => BaseApi.get('history')

export const replyToMatch = (match_id, token, ans) =>
  BaseApi.post('respond-to-match', {
    match_notification_id: match_id,
    ans: ans,
    user_token: token,
  })

export const registerMatch = (winner, loser, token) =>
  BaseApi.post('register-match', {
    winner: winner,
    loser: loser,
    token: token,
  })
