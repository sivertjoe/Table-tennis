import { ApiError, errorMap } from './ApiErrors'

const url = process.env.REACT_APP_URL ?? 'http://localhost'
const ip = process.env.REACT_APP_IP ?? '58642'
const apiUrl = url + ':' + ip + '/'

const parseResponse = (response) => {
  if (response.ok)
    return response.json().then((res) => {
      if (res.status === 0) return res.result
      throw new ApiError(res.status, errorMap[res.status])
    })

  throw new ApiError(response.status, response.statusText)
}

export const get = (url) =>
  fetch(apiUrl + url).then((res) => parseResponse(res))

export const post = (url, body) =>
  fetch(apiUrl + url, {
    method: 'POST',
    body: JSON.stringify(body),
  }).then((res) => parseResponse(res))
