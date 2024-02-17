// @generated automatically by Diesel CLI.

diesel::table! {
    board_states (id) {
        id -> Int4,
        #[max_length = 20]
        state -> Varchar,
        blocks -> Text,
        filled -> Text,
        moves -> Text,
    }
}
