// @generated automatically by Diesel CLI.

diesel::table! {
    files_metadata (id) {
        id -> Int4,
        name -> Text,
        bitrate -> Numeric,
        duration -> Numeric,
        size -> Numeric,
        created -> Timestamp,
        updated -> Timestamp,
    }
}
