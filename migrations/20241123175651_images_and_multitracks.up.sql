alter table files_metadata
drop column if exists user_id;

create table if not exists beats (
    id serial primary key,
    beat_id integer references files_metadata(id),
    name text not null,
    description text not null,
    beatmaker_id numeric not null,
    genre text not null
);

create table if not exists images (
    id serial primary key,
    beat_id integer references files_metadata(id),
    image text not null
);
