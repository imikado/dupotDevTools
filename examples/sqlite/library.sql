-- Demo database of the SQL Viewer (screenshots and demo GIF):
--   sqlite3 library.db < library.sql
CREATE TABLE author (
    id      INTEGER PRIMARY KEY,
    name    TEXT NOT NULL,
    country TEXT,
    born    INTEGER
);

CREATE TABLE publisher (
    id   INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    city TEXT
);

CREATE TABLE book (
    id           INTEGER PRIMARY KEY,
    author_id    INTEGER NOT NULL REFERENCES author(id),
    publisher_id INTEGER REFERENCES publisher(id),
    title        TEXT NOT NULL,
    published_on TEXT,
    price        REAL
);

INSERT INTO author (id, name, country, born) VALUES
    (1, 'Victor Hugo', 'France', 1802),
    (2, 'Jane Austen', 'United Kingdom', 1775),
    (3, 'Fiodor Dostoïevski', 'Russia', 1821),
    (4, 'Gabriel García Márquez', 'Colombia', 1927),
    (5, 'Toni Morrison', 'United States', 1931);

INSERT INTO publisher (id, name, city) VALUES
    (1, 'Gallimard', 'Paris'),
    (2, 'Penguin Classics', 'London'),
    (3, 'Vintage', 'New York');

INSERT INTO book (id, author_id, publisher_id, title, published_on, price) VALUES
    (1, 1, 1, 'Les Misérables', '1862-04-03', 14.5),
    (2, 1, 1, 'Notre-Dame de Paris', '1831-03-16', 9.9),
    (3, 2, 2, 'Pride and Prejudice', '1813-01-28', 8.5),
    (4, 2, 2, 'Emma', '1815-12-23', 7.9),
    (5, 3, 2, 'Crime and Punishment', '1866-01-01', 12.0),
    (6, 3, 2, 'The Brothers Karamazov', '1880-11-01', 16.0),
    (7, 4, 3, 'One Hundred Years of Solitude', '1967-05-30', 13.5),
    (8, 5, 3, 'Beloved', '1987-09-02', 11.0);
