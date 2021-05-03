import * as BaseApi from './BaseApi'

export const uploadAward = (tournamentId, image) =>
  BaseApi.postImage('upload-award', {
    tournamentId: tournamentId,
    image: image,
  })
