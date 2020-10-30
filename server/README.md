# REST API

## Get Users

----
  Returns a json list with all users.

* **URL**

  /users

* **Method:**

  `GET`
  
*  **URL Params**

    None

* **Uri Params**

    None

* **Success Response:**

    * **Code:** 200 <br />
        **Content:** 

```json
{
  "message": "success",
  "data": [
  {
    "id": 2,
    "elo": 1501.4695015289756,
    "name": "Ola",
    "match_history": [
      {
        "winner": "Kari",
        "loser": "Ola",
        "epoch": 1604050993
      },
      {
        "winner": "Ola",
        "loser": "Kari",
        "epoch": 1604050993
      }
    ]
  },
  {
    "id": 1,
    "elo": 1498.5304984710244,
    "name": "Kari",
    "match_history": [
      {
        "winner": "Kari",
        "loser": "Ola",
        "epoch": 1604050993
      },
      {
        "winner": "Ola",
        "loser": "Kari",
        "epoch": 1604050993
      }
    ]
  }
  ]
}
```
 ## Get Specific user

----
  Returns a json object of the user

* **URL**

  /user/{name}

* **Method:**

  `GET`
  
*  **URL Params**

    Name of the user

* **Uri Params**

    None

* **Success Response:**

    * **Code:** 200 <br />
        **Content:** 

```json
{
  "message": "success",
  "data": {
    "id": 2,
    "elo": 1501.4695015289756,
    "name": "Ola",
    "match_history": [
      {
        "winner": "Kari",
        "loser": "Ola",
        "epoch": 1604050993
      },
      {
        "winner": "Ola",
        "loser": "Kari",
        "epoch": 1604050993
      }
    ]
  },
}
```
 
## Create New User

----
  Creates a new user based on the name specified in the url

* **URL**

  /create-user/{name}

* **Method:**

  `POST`
  
*  **URL Params**

   name of the new user

*  **Uri Params**

    None 

* **Success Response:**

    * **Code:** 200 <br />
        **Content:** 

* **Error Response:**

  * **Code:** 403 ALREADY EXISTS <br />

## Register New Match

----
  Creates a new match based on the match object contained in the request body.

* **URL**

  /register-match

* **Method:**

  `POST`

* **URL Params**

    None
  
* **Uri Params**
?winner={name}&loser={name}&epoch={epoch}

Example: ?winner=Ola&loser=Kari&epoch=1604050993


* **Success Response:**

    * **Code:** 200 <br />

* **Error Response:**

  * **Code:** 404 NOT FOUND <br />
