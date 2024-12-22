alter table beats
drop column if exists link;
alter table beats
add column if not exists link text not null;