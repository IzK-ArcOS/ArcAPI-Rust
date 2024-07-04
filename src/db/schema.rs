diesel::table! {
    messages (id) {
        id -> Integer,
        sender_id -> Integer,
        receiver_id -> Integer,
        body -> Nullable<Text>,
        replying_id -> Integer,
        sent_time -> Timestamp,
        is_read -> Nullable<Bool>,
        is_deleted -> Bool,
    }
}

diesel::table! {
    tokens (value) {
        value -> Text,
        owner_id -> Integer,
        lifetime -> Nullable<Float>,
        creation_time -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Nullable<Text>,
        hashed_password -> Nullable<Text>,
        creation_time -> Timestamp,
        properties -> Nullable<Text>,  // todo somehow convey it that Json is convertible to serde's json (as per docs)
        is_deleted -> Bool,
    }
}

diesel::joinable!(tokens -> users (owner_id));

diesel::allow_tables_to_appear_in_same_query!(
    messages,
    tokens,
    users,
);
