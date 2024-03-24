# klotski-api

This API supports [Klotski UI](https://github.com/samrroyall/klotski). It is build using the Axum framework and uses Diesel to communicate with a Postgres database.

## Project Structure

```bash
.
└── src
    ├── docs.rs
    ├── errors
    │   ├── board.rs
    │   ├── handler.rs
    │   ├── http.rs
    │   └── mod.rs
    ├── handlers
    │   ├── board.rs
    │   └── mod.rs
    ├── main.rs
    ├── models
    │   ├── api
    │   │   ├── mod.rs
    │   │   ├── request.rs
    │   │   └── response.rs
    │   ├── db
    │   │   ├── mod.rs
    │   │   ├── schema.rs
    │   │   └── tables.rs
    │   ├── game
    │   │   ├── blocks.rs
    │   │   ├── board.rs
    │   │   ├── mod.rs
    │   │   ├── moves.rs
    │   │   └── utils.rs
    │   └── mod.rs
    ├── repositories
    │   ├── boards.rs
    │   ├── mod.rs
    │   └── solutions.rs
    └── services
        ├── db.rs
        ├── mod.rs
        ├── randomizer.rs
        └── solver.rs
```

- `docs.rs` - Contains the OpenAPI specification for the API for use in the RapiDoc webpage

- `errors/`
    - `board.rs` - Contains the `Error` structure used for error handling related to board operations
    - `handler.rs` - Contains the `Error` structure used for error handling related to validtion of request parameters
    - `http.rs` - Contains the `Error` structure related HTTP failure responses along with `From` implementations for the other error structures

- `handlers/` 
    - `board.rs` - Contains handlers for board and block operations

- `main.rs` - The entry point of the API

- `models/`
    - `api/`
        - `request.rs` - Contains structures related to request types
        - `response.rs` - Contains structures related to response types
    - `db/`
        - `schema.rs` - Contains the Diesel-generated schema for the two database tables
        - `tables.rs` - Contains structures for the insertable and selectable representations of records for each of the two databse tables
    - `game/`
        - `blocks.rs` - Contains the `Block` enumeration and the `Positioned` structure used for block representation
        - `board.rs` - Contains the `Board` and `BoardState` structures as well as logic related to board operations
        - `moves.rs` - Contains the `Step` enumeration and the `FlatMove` and `FlatBoardMove` structures related to block movement
        - `utils.rs` - Contains the `Position` enumeration representing cell coordinates

- `repositories/`
    - `board.rs` - Contains CRUD operations for records in the `boards` database table
    - `solutions.rs` - Contains CRUD operations for records in the `solutions` database table

- `services/`
    - `db.rs` - Contains utility methods related to database connection
    - `randomizer.rs` - Exposes the `randomize()` function used for generating random block configurations on boards
    - `solver.rs` - Exposes the `solve()` function used for finding optimal solutions for boards

## Endpoints

### Documentation

- Path: `GET /rapidoc`
- Description: RapiDoc dashboard

### Board Operations

#### Create Board 

- Path: `POST /api/board/`
- Description: Creates a new empty board and will optionally randomly place blocks. *Note*: Randomly generated board may be unsolvable.
- Request Body: The type of board to create

    ```json
    {
        "type": "empty" | "random"
    }
    ```

- Response Body: The new board

    ```json
    {
        "id": number,
        // current state of the board
        "state": "building" | "ready_to_solve" | "solving" | "solved",
        // list of placed blocks
        "blocks": [
            {
                "block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
                // top-left position of block
                "min_position": {row: number, col: number},
                // bottom-right position of block
                "max_position": {row: number, col: number},
                // list of positions covered by block
                "range": [
                    {row: number, col: number},
                    ...
                ]
            },
            ...
        ],
        // Flat list of blocks covering each cell in the 5x4 board
        "grid": [
            "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two" | null,
            ...
        ],
        // list of available moves for each placed block
        "next_moves": [
            [
                {"row_diff": number, "col_diff": number},
                ...
            ]
            ...
        ]
    }
    ```

#### Alter Board 
- Path: `PUT api/board/:board_id`
- Description: Modifies the board be either **a)** changing its state, **b)** undoing the last move, or **c)** resetting the board by undoing all moves that have taken place. Note: rules for 
- Request Body: The type of board alteration to be performed

    ```json
    {
        type: "change_state" | "undo_move" | "reset",
        // if type is "change_state" the below must be provided
        new_state: "building" | "ready_to_solve" | "solving" | "solved"
    }
    ```

- Response Body: The altered board

    ```json
    {
        "id": number,
        // current state of the board
        "state": "building" | "ready_to_solve" | "solving" | "solved",
        // list of placed blocks
        "blocks": [
            {
                "block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
                // top-left position of block
                "min_position": {row: number, col: number},
                // bottom-right position of block
                "max_position": {row: number, col: number},
                // list of positions covered by block
                "range": [
                    {row: number, col: number},
                    ...
                ]
            },
            ...
        ],
        // Flat list of blocks covering each cell in the 5x4 board
        "grid": [
            "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two" | null,
            ...
        ],
        // list of available moves for each placed block
        "next_moves": [
            [
                {"row_diff": number, "col_diff": number},
                ...
            ]
            ...
        ]
    }
    ```

#### Delete Board 

- Path: `DELETE api/board/:board_id`
- Description: Deletes the board
- Request Body: None
- Response Body: None

#### Solve Board

- Path: `POST api/board/:board_id/solve`
- Description: Solves the board
- Request Body: None
- Response Body: An optimal list of moves required to solve the board if solvable

    ```json
    {
        "type": "unable_to_solve" | "solved",
        // If the type is "solved", the below will be provided
        "moves": [
            {
                "block_idx": number,
                "row_diff": number,
                "col_diff": number
            },
            ...
        ]
    }
    ```

### Block operations

#### Add Block 

- Path: `POST /api/board/:board_id/block`
- Description: Adds the block to the board
- Request Body: The block and the board position where where it should be placed, as represented by the top-left cell of that position

    ```json
    {
        "block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
        "min_row": number,
        "min_col": number
    }
    ```

- Response Body: The updated board

    ```json
    {
        "id": number,
        // current state of the board
        "state": "building" | "ready_to_solve" | "solving" | "solved",
        // list of placed blocks
        "blocks": [
            {
                "block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
                // top-left position of block
                "min_position": {row: number, col: number},
                // bottom-right position of block
                "max_position": {row: number, col: number},
                // list of positions covered by block
                "range": [
                    {row: number, col: number},
                    ...
                ]
            },
            ...
        ],
        // Flat list of blocks covering each cell in the 5x4 board
        "grid": [
            "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two" | null,
            ...
        ],
        // list of available moves for each placed block
        "next_moves": [
            [
                {"row_diff": number, "col_diff": number},
                ...
            ]
            ...
        ]
    }
    ```

#### Alter Block 

- Path: `PUT /api/board/:board_id/block/:block_idx`
- Description: Modifies a block by either changing it into a different block variation or moving it the specified amount
- Request Body: The type of modification to apply to the block

    ```json
    {
        "type": "change_block" | "move_block",
        // if the type is "change_block", the below must be specified
        "new_block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
        // if the type is "move_block", the below must be specified
        "row_diff": number,
        "col_diff": number
    }    
    ```

- Response Body: The updated board

    ```json
    {
        "id": number,
        // current state of the board
        "state": "building" | "ready_to_solve" | "solving" | "solved",
        // list of placed blocks
        "blocks": [
            {
                "block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
                // top-left position of block
                "min_position": {row: number, col: number},
                // bottom-right position of block
                "max_position": {row: number, col: number},
                // list of positions covered by block
                "range": [
                    {row: number, col: number},
                    ...
                ]
            },
            ...
        ],
        // Flat list of blocks covering each cell in the 5x4 board
        "grid": [
            "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two" | null,
            ...
        ],
        // list of available moves for each placed block
        "next_moves": [
            [
                {"row_diff": number, "col_diff": number},
                ...
            ]
            ...
        ]
    }
    ```

#### Remove Block 

- Path: `DELETE /api/board/:board_id/block/:block_idx`
- Description: Removes the specified block from the board
- Request Body: None
- Response Body: The updated board

    ```json
    {
        "id": number,
        // current state of the board
        "state": "building" | "ready_to_solve" | "solving" | "solved",
        // list of placed blocks
        "blocks": [
            {
                "block": "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two",
                // top-left position of block
                "min_position": {row: number, col: number},
                // bottom-right position of block
                "max_position": {row: number, col: number},
                // list of positions covered by block
                "range": [
                    {row: number, col: number},
                    ...
                ]
            },
            ...
        ],
        // Flat list of blocks covering each cell in the 5x4 board
        "grid": [
            "one_by_one" | "one_by_two" | "two_by_one" | "two_by_two" | null,
            ...
        ],
        // list of available moves for each placed block
        "next_moves": [
            [
                {"row_diff": number, "col_diff": number},
                ...
            ]
            ...
        ]
    }
    ```

