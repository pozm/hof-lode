-- Add migration script here
CREATE TABLE fcMembers (
    id INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    avatar VARCHAR(512) NOT NULL,
    rank VARCHAR(128) NOT NULL,
    left BOOLEAN NOT NULL,
    leftDate DATETIME,
    entryDate DATETIME,
    PRIMARY KEY (id)
)