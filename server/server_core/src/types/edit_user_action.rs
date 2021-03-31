pub enum EditUserAction
{
    MakeUserActive,
    MakeUserRegular,
    MakeUserInactive,
    MakeUserSoftInactive,
    MakeUserSuperuser,
}

impl std::str::FromStr for EditUserAction
{
    type Err = ();

    fn from_str(action: &str) -> Result<Self, Self::Err>
    {
        match action
        {
            "MAKE_USER_ACTIVE" => Ok(EditUserAction::MakeUserActive),
            "MAKE_USER_REGULAR" => Ok(EditUserAction::MakeUserRegular),
            "MAKE_USER_INACTIVE" => Ok(EditUserAction::MakeUserInactive),
            "MAKE_USER_SOFT_INACTIVE" => Ok(EditUserAction::MakeUserSoftInactive),
            "MAKE_USER_SUPERUSER" => Ok(EditUserAction::MakeUserSuperuser),
            _ => Err(()),
        }
    }
}
