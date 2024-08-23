diesel::table! {
    user {
        id -> Integer,
        hpassword -> VarChar,
        username -> VarChar,
        firstname -> Nullable<VarChar>,
        patronym -> Nullable<VarChar>,
        surname -> Nullable<VarChar>,
        rt -> Nullable<VarChar>
    }
}

diesel::table! {
    user_change {
        id -> Integer,
        created -> Double,
        action -> VarChar,
        user_id -> Integer
    }
}

diesel::joinable!(user_change -> user (user_id));
