export class ApiError extends Error {
  constructor(type, message) {
    super(message)
    this.type = type
  }
}

export const errorMap = {
  1: 'Critical error',
  2: 'User does not exist.',
  3: 'Username is already taken.',
  4: 'Incorrect username or password',
  5: 'Wrong password',
  6: 'Admin privileges required.',
  7: 'Waiting for approval by an admin.',
  8: 'User is inactive.',
  9: 'Password reset has already been requested.',
  10: "Invalid username. Must be valid by the following regex: '^[a-zåA-ZæøåÆØÅ0-9_-]*$'",
  // TODO: Get the error message from response
}
