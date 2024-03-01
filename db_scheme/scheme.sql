DROP TABLE IF EXISTS Users CASCADE;
DROP TABLE IF EXISTS Coin CASCADE;
DROP TABLE IF EXISTS Amount CASCADE;
DROP TABLE IF EXISTS Game CASCADE;
DROP TABLE IF EXISTS UserSeed CASCADE;
DROP TABLE IF EXISTS ServerSeed CASCADE;
DROP TABLE IF EXISTS Bet CASCADE;

CREATE TABLE IF NOT EXISTS Users(
    id BIGSERIAL PRIMARY KEY,
    registration_time TIMESTAMP DEFAULT NOW(),

    login TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL,
    password char(128) NOT NULL
);

CREATE TABLE IF NOT EXISTS Coin(
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    price NUMERIC(1000, 4) NOT NULL
);

CREATE TABLE IF NOT EXISTS Amount(
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    coin_id BIGSERIAL NOT NULL REFERENCES Coin(id) ON DELETE CASCADE,

    amount NUMERIC(1000, 4) DEFAULT 0
);

CREATE UNIQUE INDEX amount_unique_idx ON Amount(user_id, coin_id);

CREATE TABLE IF NOT EXISTS Game(
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,

    parameters TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS UserSeed(
    id BIGSERIAL PRIMARY KEY,
    --relative_id BIGINT NOT NULL,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,

    user_seed char(64) NOT NULL
);

CREATE UNIQUE INDEX user_seed_unique_idx ON UserSeed(user_id, user_seed);

CREATE TABLE IF NOT EXISTS ServerSeed(
    id BIGSERIAL PRIMARY KEY,
    --relative_id BIGINT NOT NULL,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,

    server_seed char(128) NOT NULL,
    revealed boolean NOT NULL
);

CREATE TABLE IF NOT EXISTS Bet(
    id BIGSERIAL PRIMARY KEY,
    --relative_id BIGINT,
    timestamp TIMESTAMP DEFAULT NOW(),
    amount NUMERIC(1000, 4),
    profit NUMERIC(1000, 4),
    num_games INTEGER NOT NULL,
    outcomes TEXT NOT NULL,

    bet_info TEXT NOT NULL,

    game_id BIGSERIAL NOT NULL REFERENCES Game(id) ON DELETE CASCADE,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    coin_id BIGSERIAL NOT NULL REFERENCES Coin(id) ON DELETE CASCADE,
    userseed_id BIGSERIAL NOT NULL REFERENCES UserSeed(id) ON DELETE CASCADE,
    serverseed_id BIGSERIAL NOT NULL REFERENCES ServerSeed(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX bet_unique_idx ON Bet(userseed_id, serverseed_id, id);


CREATE TABLE IF NOT EXISTS Invoice(
    id TEXT NOT NULL PRIMARY KEY,
    merchant_id TEXT NOT NULL,
    order_id TEXT NOT NULL UNIQUE,
    create_date TIMESTAMP DEFAULT NOW(),
    status INTEGER NOT NULL,
    pay_url TEXT NOT NULL,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    amount NUMERIC(1000, 4),
    currency TEXT NOT NULL
);


-- DATA


-- COINS
INSERT INTO Coin(
    name,
    price
) VALUES (
    'DraxBonus',
    1
);
INSERT INTO Coin(
    name,
    price
) VALUES (
    'Drax',
    10
);


-- GAMES
INSERT INTO Game(
    name,
    parameters
) VALUES (
    'CoinFlip',
    '{"profit_coef":"1.98"}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Dice',
    '{"profit_coef":"1.98"}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'RPS',
    '{"profit_coef":"1.98", "draw_coef":"0.99"}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Race',
    '{"profit_coef":"4.9", "cars_amount":5}'
);