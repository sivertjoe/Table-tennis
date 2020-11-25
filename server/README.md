# REST API


## Create User
Creates a new user.\
The user must be accepted by an admin.

* **URL**\
    /create-user

* **Method:**\
    `POST`

* **Parameters**\
    username: Username of new user\
    password: Password of new user

* **Success Response:**
    ```json
    {
        "status": 0
    }
    ```

* **Possible Error Codes**\
    2: Username is alredy taken


## Edit User
Edits a list of users\
Must be admin to use this function.

* **URL**\
    /edit-users

* **Method:**\
    `POST`

* **Parameters**\
    token: The token of the currently logged in user\
    users: A list of usernames\
    action: The edit action to be performed

  * **Possible Actions**\
      MAKE_USER_ACTIVE\
      MAKE_USER_REGULAR\
      MAKE_USER_INACTIVE\
      MAKE_USER_SUPERUSER

* **Success Response:**
    ```json
    {
        "status": 0
    }
    ```

* **Possible Error Codes**\
    5: User is not admin


## Get Users
Returns a list of all active users.

* **URL**\
    /users

* **Method:**\
    `GET`

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": [
            {
                "id": 1,
                "name": "Markus",
                "elo": 1701.4695015289756,
                "user_role": 0,
                "match_history": []
            },
            {
                "id": 2,
                "name": "Sivert",
                "elo": 1498.5304984710244,
                "user_role": 2,
                "match_history": []
            }
        ]
    }
    ```


## Get All Users
Returns a list of all users.\
Must be admin to use this function.

* **URL**\
    /all-users/<token>

* **Method:**\
    `GET`

* **URL Parameters**\
    token: The token of the currently logged in user.

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": [
            {
                "id": 1,
                "name": "Markus",
                "elo": 1701.4695015289756,
                "user_role": 0,
                "match_history": []
            },
            {
                "id": 2,
                "name": "Sivert",
                "elo": 1498.5304984710244,
                "user_role": 2,
                "match_history": []
            }
        ]
    }
    ```

* **Possible Error Codes**\
    5: User is not admin


## Get Specific user
Returns a specific user

* **URL**\
    /user/<name>

* **Method:**\
    `GET`

* **URL Parameters**\
    name: Username of the user

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": {
            "id": 2,
            "elo": 1498.4695015289756,
            "name": "Sivert",
            "match_history": [
                {
                    "winner": "Markus",
                    "loser": "Sivert",
                    "epoch": 1606310183219
                },
                {
                    "winner": "Sivert",
                    "loser": "Bernt",
                    "epoch": 1606310223219
                }
            ]
        },
    }
    ```


## Register Match
Creates a new match.\
The match must be accepted by the other player (or both if a third user registered the match).

* **URL**\
    /register-match

* **Method:**\
    `POST`

* **Parameters**\
    token: The token of the currently logged in user\
    winner: Username of the winner\
    loser: Username of the loser

* **Success Response:**
    ```json
    {
        "status": 0
    }
    ```


## Respond To Match
Accepts or declines a match.\

* **URL**\
    /respond-to-match

* **Method:**\
    `POST`

* **Parameters**\
    token: The token of the currently logged in user\
    match_notification_id: Id of the match notification to respond to\
    ans: Either 1 or 2 (1 = accept, 2 = decline)

* **Success Response:**
    ```json
    {
        "status": 0
    }
    ```


## Respond To New User
Accepts or declines a new user.\
Must be admin to use this function.

* **URL**\
    /respond-to-user-notification

* **Method:**\
    `POST`

* **Parameters**\
    token: The token of the currently logged in user
    id: Id of the user notification to respond to
    ans: Either 1 or 2 (1 = accept, 2 = decline)

* **Success Response:**
    ```json
    {
        "status": 0
    }
    ```

* **Possible Error Codes**\
    5: User is not admin


## Get Match History
Get the history of all matches.\

* **URL**\
    /history

* **Method:**\
    `GET`

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": [
            {
                "winner": "Markus",
                "loser": "Sivert",
                "elo_diff": 17.4695015289756,
                "winner_elo": 1701.6695716489756,
                "loser_elo": 1410.2495015289857,
                "epoch": 1606310183219
            },
            {
                "winner": "Bernt",
                "loser": "Ella",
                "elo_diff": 19.4695015289756,
                "winner_elo": 1501.6695716489756,
                "loser_elo": 1510.2495015289857,
                "epoch": 1606310083219
            }
        ]
    }
    ```

## Get Notifications
Get the logged in users match notifications.

* **URL**\
    /notifications/<token>

* **Method:**\
    `GET`

* **URL Parameters**\
    token: The token of the currently logged in user.

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": [
            {
                "id": 1,
                "winner": "Markus",
                "loser": "Sivert",
                "epoch": 1606310183219
            },
            {
                "id": 5,
                "winner": "Markus",
                "loser": "Ella",
                "epoch": 1606310083219
            }
        ]
    }
    ```


## Get New User Notifications
Get all new user notifications.\
Must be admin to use this function.

* **URL**\
    /user-notifications/<token>

* **Method:**\
    `GET`

* **URL Parameters**\
    token: The token of the currently logged in user.

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": [
            {
                "id": 3,
                "name": "Sigurd"
            },
            {
                "id": 5,
                "name": "Lars"
            }
        ]
    }
    ```

* **Possible Error Codes**\
    5: User is not admin


## Check If Admin
Checks if the logged in user is an admin.

* **URL**\
    /notifications/<token>

* **Method:**\
    `GET`

* **URL Parameters**\
    token: The token of the currently logged in user.

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": true
    }
    ```


## Login
Log in.\
Returns a token that represents the user being logged in.

* **URL**\
    /login

* **Method:**\
    `POST`

* **Parameters**\
    username: Username of the user logging in.\
    password: Password of the user logging in.

* **Success Response:**
    ```json
    {
        "status": 0,
        "result": "25517cc6-bb8a-4b04-a4ec-bc096a42b80a"
    }
    ```

* **Possilbe Error Codes**\
    1: The user does not exist\
    3: Wrong username or password\
    6: The user must be accepted by an admin\
    7: The user is inactive


## Change Password
Change the password of a user.

* **URL**\
    /change-password

* **Method:**\
    `POST`

* **Parameters**\
    username: Username of the user.\
    password: The old password.\
    new_password: The new password.

* **Success Response:**
    ```json
    {
        "status": 0,
    }
    ```

* **Possilbe Error Codes**\
    1: The user does not exist\
    4: The old password is incorrect

