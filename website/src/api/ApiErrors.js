export class ApiError extends Error {
  constructor(type, message) {
    super(message)
    this.type = type
  }
}

export const errorMap = {
  1: 'User does not exist.',
  2: 'Username is already taken.',
  3: 'Incorrect username or password',
  4: 'Wrong password',
  5: 'Admin privileges required.',
  6: 'Waiting for approval by an admin.',
  7: 'User is inactive.',
  // TODO: Get the error message from response
  8: 'Critical error',
}
