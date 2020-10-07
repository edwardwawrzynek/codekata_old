# Codekata

A tool for running programming compeitions to play abstract board games. Currently, it is configured to run the game [gomoku](https://en.wikipedia.org/wiki/Gomoku).

Codekata is deployed at [https://codekata.herokuapp.com](https://codekata.herokuapp.com). If you are writing a client, it is recommended you test against this deployment.

If you need to run codekata locally, see the "Local Setup" section below.

## Gomoku
Gomoku is played on a 15 x 15 board. Players take turns alternating placing stones, and the first player to get five or more stones in a row, column, or diagonal wins.

## API Routes
API keys have to be included in all requests as an `X-API-KEY` http header (not as a request parameter). All routes return valid json.

#### `POST /api/game/<game_id>/join`
Join the given game. Returns:
```
{ "success": boolean }
```
If the `success` field is false, the response also includes a `error` field explaining what went wrong.

#### `GET /api/game/<game_id>/move_needed`
Check if you need to make a move.
Returns:
```
{ "needed": boolean }
```

#### `GET /api/game/<game_id>`
Returns that state of the board. Returns:
```
{
  "state" : {
    "board": [
      [0, 1, -1, -1, -1, -1, -1, -1, ...],
      [-1, -1, -1, -1, -1, -1, -1, -1, ...].
      ...
    ],
    "turn": 0,
  },
  /* other fields that can be ignored */
}
```
The `state.board` field is indexed `[x][y]`. A value of `-1` indicates the cell is empty, a `0` indicates it has your piece on it, and a `1` indicates it has your opponent's piece on it.

#### `POST /api/game/<game_id>/move - params(x: int, y: int)`

Make a move at the given x and y position. Returns:
```
{ "success": boolean }
```

If `success` is false, the response will include an `error` field describing what went wrong.

## Writing A Client
1. Get an API key and game id as input (probably from command line args or something).
2. Join the game: `POST /api/game/<game_id>/join`.
3. Check if a move is needed: `GET /api/game/<game_id>/move_needed`, and wait until a move is needed
4. Load the state of the game: `GET /api/game/<game_id>`. Use the `state.board` field.
5. Make a move: `POST /api/game/<game_id>/move`.
6. Goto #3

All requests need your api key sent as the `X-API-KEY` http header -- look at your library's documentation for how to do this.

## Local Setup

1. Install [node and npm](https://nodejs.org/en/download/), [rust](https://www.rust-lang.org/tools/install), and [postgres](https://www.postgresql.org/).
2. Clone this repo (`git clone https://github.com/edwardwawrzynek/codekata`)
3. Use nightly rust (`rustup default nightly`)
4. Install the diesel cli (`cargo install diesel_cli`)
5. Create the postgres database (`DATABASE_URL=postgres://postgres:@localhost/codekata_db diesel setup`)
6. Build and start the server (`ROCKET_PORT=8000 DATABASE_URL=postgres://postgres:@localhost/codekata_db ROCKET_DATABASES="{db={url=$DATABASE_URL}}" cargo run`)
7. At the same time, start the frontend (`cd frontend && npm i && npm start`)

The frontend serves on http://localhost:3000.