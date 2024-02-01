diesel::table! {
    board_states (id) {
        id -> Text,
        is_ready_to_solve -> Bool,
        is_solved -> Bool,
        blocks -> Text,
        filled -> Text,
        moves -> Text,
    }
}
