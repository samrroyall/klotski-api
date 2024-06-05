// @generated automatically by Diesel CLI.

diesel::table! {
    boards (id) {
        id -> Int4,
        #[max_length = 20]
        state -> Varchar,
        blocks -> Text,
        grid -> Text,
        moves -> Text,
    }
}

diesel::table! {
    solutions (id) {
        id -> Int4,
        hash -> Int8,
        moves -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(boards, solutions,);
