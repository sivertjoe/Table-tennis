import { ApiError, errorMap } from './ApiErrors'
import useFetch from 'use-http'

const url = process.env.REACT_APP_URL ?? 'http://localhost'
const ip = process.env.REACT_APP_IP ?? '58642'
const apiUrl = url + ':' + ip + '/'

export const GetHook = (url) => {
  const { loading, error, data = [] } = useFetch(apiUrl + url, {}, [])

  if (!loading) {
    if (data.status !== 0) {
      throw new ApiError(data.status, errorMap[data.status])
    } else {
      return [false, data.result]
    }
  } else {
    return [true, {}]
  }
}

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

export const getImageUrl = (url) => apiUrl + 'assets/' + url

const _post = (url, body) =>
  fetch(apiUrl + url, {
    method: 'POST',
    body: JSON.stringify(body),
  }).then((res) => parseResponse(res))

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
