# API

- `POST /authorize`

  - request:
    ```json
    {
      "username": string,
      "password": string
    }
    ```
  - response:
    ```json
    {
      "access_token": string,
      "token_type": string
    }
    ```

- `POST /logout`

  - request:
    (no body)  
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "session_id": string,
      "ok": boolean
    }
    ```

- `POST /renew`
  - request:
    ```json
    {
      "refresh_token": string
    }
    ```
  - response:
    ```json
    {
      "access_token": string,
      "token_type": string
    }
    ```
- `POST /tabs`
  - request:
    ```json
    {
      "user_id": string,
      "url": string,
      "notes": string | null
    }
    ```
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "url": string,
      "notes": string | null
    }
    ```
- `POST /tabs/with-tags`
  - request:
    ```json
    {
      "tab": {
      "user_id": string,
      "url": string,
      "notes": string | null,
      },
      "tags": {
        "id": string | null,
        "user_id": string,
        "tag": string
      }[]
    }
    ```
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "tab": {
        "id": string,
        "user_id": string,
        "url": string,
        "notes": string | null
      },
      "tags": {
        "id": string,
        "user_id": string,
        "tag": string
      }[]
    }
    ```
- `GET /tabs/:tab_id`
  - request:  
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "url": string,
      "notes": string | null
    }
    ```
- `GET /tabs/:tab_id/with-tags`
  - request:  
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "tab": {
        "id": string,
        "user_id": string,
        "url": string,
        "notes": string | null
      },
      "tags": {
        "id": string,
        "user_id": string,
        "tag": string
      }[]
    }
    ```
- `GET /users/tabs`
  - request:  
    optional query parameters:
    - `page`: number
    - `page_size`: number
      requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "results": {
        "id": string,
        "user_id": string,
        "url": string,
        "notes": string | null
      }[],
      "has_more": boolean
    }
    ```
- `POST /tabs/:tab_id/tags`
  - request:
    ```json
    {
      "user_id": string,
      "tab_id": string
      "tag_id": string
    }
    ```
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "tag": string
    }
    ```
- `DELETE /tabs/:tab_id/tags/:tag_id`
  - request:
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "tag": string
    }
    ```
- `POST /tags`
  - request:
    ```json
    {
      "user_id": string,
      "tag": string
    }
    ```
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "tag": string
    }
    ```
- `DELETE /tags/:tag_id`
  - request:
    requires `Authorization` header with `Bearer` token
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "tag": string
    }
    ```
- `GET /users/:user_id/tags`
  - request:  
    requires `Authorization` header with `Bearer` token  
    optional query parameters:
    - `page`: number
    - `page_size`: number
  - response:
    ```json
    {
      "results": {
        "id": string,
        "user_id": string,
        "tag": string
      }[],
      "has_more": boolean
    }
    ```
- `GET /users/:user_id/tags/fuzzy`
  - request:
    requires `Authorization` header with `Bearer` token  
    required query parameter:
    - `fragment`: string
  - response:
    ```json
    {
      "matches": {
        "id": string,
        "user_id": string,
        "tag": string
      }[],
    }
    ```
- `POST /users`
  - request:
    ```json
    {
      "email": string,
      "password": string
    }
    ```
  - response:
    ```json
    {
      "email": string,
      "invite_id": string,
      "user_id": string
    }
    ```
- `PUT /users/invites/:invite_id`
  - request:
    ```json
    {
      "id": string,
      "status": string
    }
    ```
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "email": string,
      "status": string,
      "expires": number
    }
    ```
- `GET /users/invites/:invite_id`
  - request:
    (no body)
  - response:
    ```json
    {
      "id": string,
      "user_id": string,
      "email": string,
      "status": string,
      "expires": number
    }
    ```
- `POST /users/:user_id`
  - request:
    ```json
    {
      "email": string,
      "invite_id": string
    }
    ```
  - response:
  ```json
   {
     "id": string,
     "email": string,
     "password": string,
     "confirmed": boolean
   }
  ```
