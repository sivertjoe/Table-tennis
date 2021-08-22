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
  11: 'Tournament has wrong state',
  12: 'Tournament with that id not found',
  13: 'User is not organizer',
  14: 'Game is invalid',
  15: 'Wrong tournament count',
  16: 'Already joined tournament',
  17: 'Game already played',
  18: 'Invalid token',
  // TODO: Get the error message from response
}
