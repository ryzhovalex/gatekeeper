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

