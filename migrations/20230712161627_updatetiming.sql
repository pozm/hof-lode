-- Add migration script here

CREATE TABLE updateTime (
    id int primary key,
    last_update datetime
);
INSERT into updateTime (id, last_update) values (1, '2019-01-01 00:00:00');