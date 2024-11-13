create table files_metadata (
    id serial primary key,
    name text not null,
    bitrate numeric not null,
    duration numeric not null,
    size numeric not null,
    created timestamp not null default current_timestamp,
    updated timestamp not null default current_timestamp
);
