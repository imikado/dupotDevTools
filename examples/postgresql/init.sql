CREATE TABLE author (
    id         SERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    country    TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE book (
    id           SERIAL PRIMARY KEY,
    author_id    INTEGER NOT NULL REFERENCES author(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    published_on DATE,
    price        NUMERIC(6,2)
);

INSERT INTO author (name, country) VALUES
    ('Victor Hugo', 'France'),
    ('Jules Verne', 'France'),
    ('Agatha Christie', 'United Kingdom');

INSERT INTO book (author_id, title, published_on, price) VALUES
    (1, 'Les Misérables', '1862-04-03', 12.90),
    (1, 'Notre-Dame de Paris', '1831-01-14', 9.50),
    (2, 'Vingt mille lieues sous les mers', '1870-06-20', 8.00),
    (2, 'Le Tour du monde en quatre-vingts jours', '1872-12-22', 7.50),
    (3, 'Murder on the Orient Express', '1934-01-01', 10.00),
    (3, 'And Then There Were None', '1939-11-06', NULL);
