use rusqlite::params;
use serde_derive::Serialize;
use server_core::{
    constants::{INVALID_TOKEN, VALID_TOKEN},
    types::*,
};
use server_macro::Sql;
use uuid::Uuid;

use super::_params;
use crate::server::{DataBase, ParamsType};

#[derive(Serialize)]
pub struct Payload
{
    pub username: String,
}

#[derive(Sql)]
pub struct Tokens
{
    pub id:            i64,
    pub access_token:  String,
    pub refresh_token: String,
    pub valid:         i64,
}

impl DataBase
{
    pub fn generate_tokens(&self) -> (String, String)
    {
        let (access_token, refresh_token) =
            (Uuid::new_v4().to_string(), Uuid::new_v4().to_string());
        (access_token, refresh_token)
    }

    pub fn insert_tokens_in_db(&self, (a, r): &(String, String)) -> ServerResult<i64>
    {
        self.conn.execute(
            "insert into tokens (access_token, refresh_token, valid) values (?1, ?2, 1)",
            params![a, r],
        )?;
        self.sql_one::<Tokens, _>("select * from tokens order by id desc limit 1", None)
            .map(|tokens| tokens.id)
    }

    pub fn invalidate_token(&self, user_id: i64) -> ServerResult<()>
    {
        self.conn.execute(
            "update tokens set valid = ?2 where id = (select token_id from users where id = ?1)",
            params![user_id, INVALID_TOKEN],
        )?;
        Ok(())
    }

    pub fn get_token_status(&self, token: &String) -> ServerResult<bool>
    {
        self.sql_one::<Tokens, _>("select * from tokens where access_token = ?1", _params![token])
            .map(|tokens| tokens.valid == 1)
    }

    pub fn renew_tokens(&self, refresh_token: &String) -> ServerResult<(Payload, Tokens)>
    {
        match self.sql_one::<Tokens, _>("select * from tokens where refresh_token = ?1", _params![
            refresh_token
        ])
        {
            Ok(tokens) =>
            {
                let tokens = self._renew_tokens(tokens.id)?;
                let name = self.get_user_from_token(&tokens.access_token)?.name;
                Ok((
                    Payload {
                        username: name
                    },
                    tokens,
                ))
            },
            Err(_) => Err(ServerError::InvalidToken),
        }
    }

    pub fn _renew_tokens(&self, tid: i64) -> ServerResult<Tokens>
    {
        let tokens = self.generate_tokens();
        self.conn.execute(
            "update tokens set access_token = ?1, refresh_token = ?2, valid = ?4 where id = ?3",
            params![&tokens.0, &tokens.1, tid, VALID_TOKEN],
        )?;
        Ok(Tokens {
            id:            tid,
            valid:         1,
            access_token:  tokens.0,
            refresh_token: tokens.1,
        })
    }
}


#[cfg(test)]
mod test
{
    use rusqlite::NO_PARAMS;

    use super::*;
    use crate::{server::DataBase, test_util::*};

    #[test]
    fn generate_and_insert_tokens()
    {
        let db_file = "tempToken1.db";
        let s = DataBase::new(db_file);

        let tokens = s.generate_tokens();
        let res = s.insert_tokens_in_db(&tokens);
        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(res.is_ok());
    }

    #[test]
    fn can_invalidate_token()
    {
        let db_file = "tempToken2.db";
        let s = DataBase::new(db_file);
        create_user(&s, "Sivert");
        let tokens = s.login("Sivert".to_string(), "password".to_string()).unwrap();

        let res = s.invalidate_token(1);
        let ts = s.get_token_status(&tokens.access_token).unwrap();

        std::fs::remove_file(db_file).expect("Removing file tempH");
        assert!(res.is_ok());
        assert_eq!(ts, false);
    }

    #[test]
    fn user_without_tokens_gets_tokens_generated_on_login()
    {
        let db_file = "tempToken3.db";
        let s = DataBase::new(db_file);

        create_user(&s, "Sivert");
        s.conn
            .execute("update users set token_id = -1 where id = 1", NO_PARAMS)
            .unwrap();
        s.conn.execute("delete from tokens where id = 1", NO_PARAMS).unwrap();

        let tokens = s.login("Sivert".to_string(), "password".to_string());

        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(tokens.is_ok());
    }

    #[test]
    fn can_renew_tokens()
    {
        let db_file = "tempToken4.db";
        let s = DataBase::new(db_file);

        create_user(&s, "Sivert");
        let tokens = s.login("Sivert".to_string(), "password".to_string()).unwrap();

        s.invalidate_token(1).unwrap();

        let user = s.get_user_from_token(&tokens.access_token);

        let res = s.renew_tokens(&tokens.refresh_token);
        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert_eq!(user.unwrap_err(), ServerError::InvalidToken);
        assert!(res.is_ok());
    }

    #[test]
    fn login_with_invalid_token_renew_token()
    {
        let db_file = "tempToken5.db";
        let s = DataBase::new(db_file);

        create_user(&s, "Sivert");
        s.invalidate_token(1).unwrap();

        let tokens = s.login("Sivert".to_string(), "password".to_string()).unwrap();

        let user = s.get_user_from_token(&tokens.access_token);
        std::fs::remove_file(db_file).expect("Removing file tempH");

        assert!(user.is_ok());
    }
}
