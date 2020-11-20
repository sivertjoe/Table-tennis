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
  4: 'Admin privileges required.',
  5: 'Waiting for approval by an admin.',
  6: 'User is inactive.',
  // TODO: Get the error message from response
  7: 'Critical error',
}
