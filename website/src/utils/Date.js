export const getDateTime = (ms) => {
  const d = new Date(ms)
  return (
    `${d.getDate()}/${d.getMonth() + 1} ` + `${d.getHours()}:${d.getMinutes()}`
  )
}

export const getShortDate = (ms) => {
  const d = new Date(ms)
  return `${d.getFullYear()}/${d.getMonth() + 1}/${d.getDate()}`
}

export const sameDay = (d1, d2) =>
  d1.getFullYear() === d2.getFullYear() &&
  d1.getMonth() === d2.getMonth() &&
  d1.getDate() === d2.getDate()

export const getPreviousDate = (days) => {
  const offset = 24 * 3600 * 1000 * days
  const d = new Date(Date.now() - offset)
  return new Date(d.getFullYear(), d.getMonth(), d.getDate(), 0, 0, 0)
}
