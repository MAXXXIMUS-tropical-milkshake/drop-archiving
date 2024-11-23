alter table files_metadata
add column if not exists user_id numeric;

drop table if exists beats;

drop table if exists images;


