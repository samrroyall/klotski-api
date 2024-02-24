// @generated automatically by Diesel CLI.

diesel::table! {
    boards (id) {
        id -> Int4,
        #[max_length = 20]
        state -> Varchar,
        blocks -> Text,
        filled -> Text,
        moves -> Text,
        next_moves -> Text,
    }
}
