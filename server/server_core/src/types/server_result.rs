pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug)]
pub enum ServerError
{
    Rusqlite(rusqlite::Error),
    Critical(String),
    UserNotExist,
    UsernameTaken,
    WrongUsernameOrPassword,
    PasswordNotMatch,
    Unauthorized,
    WaitingForAdmin,
    InactiveUser,
    ResetPasswordDuplicate,
    InvalidUsername,
}

impl From<rusqlite::Error> for ServerError
{
    fn from(error: rusqlite::Error) -> Self
    {
        Self::Rusqlite(error)
    }
}

impl From<ServerError> for rusqlite::Error
{
    fn from(error: ServerError) -> Self
    {
        match error
        {
            ServerError::Rusqlite(e) => e,
            _ => unreachable!(),
        }
    }
}
