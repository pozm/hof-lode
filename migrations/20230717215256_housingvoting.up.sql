-- Add migration script here
CREATE TABLE IF NOT EXISTS house (
    id integer primary key AUTOINCREMENT not null,
    house_name text not null,
    address text not null,
    image text not null
);

create table if not exists poll (
    id integer primary key autoincrement not null,
    poll_name text not null,
    open BOOLEAN not null default 1,
    close_date datetime not null,
    type integer not null default 1
);
create table if not exists poll_house_option (
    id integer primary key autoincrement not null,
    poll_id integer not null,
    house_id integer not null,
    foreign key (poll_id) references poll(id),
    foreign key (house_id) references house(id)
);
create table if not exists poll_house (
    id integer primary key autoincrement not null,
    poll_id integer not null,
    option_id integer not null,
    player_name text not null,
    ip text not null unique, -- prevent multiple voting
    foreign key (poll_id) references poll(id),
    foreign key (option_id) references poll_house_option(id)
);