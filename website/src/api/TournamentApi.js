import * as BaseApi from './BaseApi'

export const uploadAward = (tournamentId, image) => {
  const data = new FormData()
  data.append('tournamentId', tournamentId)
  data.append('image', image)
  return BaseApi.postFormData('upload-award', data)
}
