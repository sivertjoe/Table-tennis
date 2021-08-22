import { ApiError, errorMap } from './ApiErrors'

const url = process.env.REACT_APP_URL ?? 'http://localhost'
const ip = process.env.REACT_APP_IP ?? '58642'
const apiUrl = url + ':' + ip + '/'

// @TODO: This shit does not work, why is url and body undefined
const parseResponse = (response, url, body) => {
  if (response.ok)
    return response.json().then((res) => {
      if (res.status === 18) {
        renewToken()

        console.log(url)
        console.log(body)
        return _post(url, body)
      }
      if (res.status === 0) return res.result
      throw new ApiError(res.status, errorMap[res.status])
    })

  throw new ApiError(response.status, response.statusText)
}

export const get = (url) =>
  fetch(apiUrl + url).then((res) => parseResponse(res))

export const getImageUrl = (url) => apiUrl + 'assets/' + url

const storePayload = (response) => {
  if (response.ok)
    response.json().then((res) => {
      const payload = res.result[0]

      if (!payload.username) {
        localStorage.setItem('username', payload.username)
      }
      localStorage.setItem('token', res.result[1])
      localStorage.setItem('refreshToken', res.result[2])
    })
}

const renewToken = () =>
  fetch(apiUrl + 'refresh', {
    method: 'POST',
    body: JSON.stringify({
      refreshToken: localStorage.getItem('refreshToken'),
    }),
  }).then((res) => storePayload(res))

const _post = (url, body) =>
  fetch(apiUrl + url, {
    method: 'POST',
    body: JSON.stringify(body),
  }).then((res) => parseResponse(res, url, body))

export const post = (url, body) => _post(url, body)

const convertImage = (image) =>
  new Promise((resolve, _) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result)
    reader.readAsDataURL(image)
  })

export const postImage = (url, body) =>
  body.image
    ? convertImage(body.image).then((img) =>
        _post(url, { ...body, image: img }),
      )
    : _post(url, { ...body, image: '' })
