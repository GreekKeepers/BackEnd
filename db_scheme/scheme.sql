DROP TABLE IF EXISTS Users CASCADE;
DROP TABLE IF EXISTS Coin CASCADE;
DROP TABLE IF EXISTS Amount CASCADE;
DROP TABLE IF EXISTS Game CASCADE;
DROP TABLE IF EXISTS UserSeed CASCADE;
DROP TABLE IF EXISTS ServerSeed CASCADE;
DROP TABLE IF EXISTS Bet CASCADE;
DROP TABLE IF EXISTS GameState CASCADE;

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
    profits TEXT NOT NULL,

    bet_info TEXT NOT NULL,
    state TEXT,
    uuid TEXT NOT NULL,

    game_id BIGSERIAL NOT NULL REFERENCES Game(id) ON DELETE CASCADE,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    coin_id BIGSERIAL NOT NULL REFERENCES Coin(id) ON DELETE CASCADE,
    userseed_id BIGSERIAL NOT NULL REFERENCES UserSeed(id) ON DELETE CASCADE,
    serverseed_id BIGSERIAL NOT NULL REFERENCES ServerSeed(id) ON DELETE CASCADE
);


CREATE TABLE IF NOT EXISTS GameState(
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMP DEFAULT NOW(),
    amount NUMERIC(1000, 4),

    bet_info TEXT NOT NULL,
    state TEXT NOT NULL,
    uuid TEXT NOT NULL,

    game_id BIGSERIAL NOT NULL REFERENCES Game(id) ON DELETE CASCADE,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    coin_id BIGSERIAL NOT NULL REFERENCES Coin(id) ON DELETE CASCADE,
    userseed_id BIGSERIAL NOT NULL REFERENCES UserSeed(id) ON DELETE CASCADE,
    serverseed_id BIGSERIAL NOT NULL REFERENCES ServerSeed(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX state_unique_idx ON Bet(game_id, user_id, coin_id, userseed_id, serverseed_id);


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

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'StatefullTest',
    '{"multiplier":"1.98"}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Wheel',
    '{"max_risk":2,
     "max_num_sectors":4,
     "multipliers": [
        [
            [
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0"
            ],
            [
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0"
            ],
            [   
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0"
            ],
            [
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0"
            ],
            [
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.5",
                "1.2",
                "1.2",
                "1.2",
                "0.0",
                "1.2",
                "1.2",
                "1.2",
                "1.2",
                "0.0"
            ]
        ],
        [
            [
                "0.0",
                "1.9",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0"
            ],
            [
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.8",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0"
            ],
            [
                "1.5",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "1.7",
                "0.0",
                "4.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0"
            ],
            [
                "2.0",
                "0.0",
                "3.0",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "1.6",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0"
            ],
            [
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "3.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0",
                "1.5",
                "0.0",
                "5.0",
                "0.0",
                "1.5",
                "0.0",
                "2.0",
                "0.0",
                "1.5",
                "0.0"
            ]
        ],
        [
            [
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "9.9"
            ],
            [
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "19.8"
            ],
            [
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "29.7"
            ],
            [
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "39.6"
            ],
            [
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "0.0",
                "49.5"
            ]
        ]
     ]
     }'
);




INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Mines',
    '{"max_reveal":[
        24, 21, 17, 14, 12, 10, 9, 8, 7, 6, 5, 5, 4, 4, 3, 3, 3, 2, 2, 2, 2, 1, 1, 1 
    ],
    "multipliers":[["1.0312", "1.076", "1.125", "1.1785", "1.2375", "1.3026", "1.375", "1.4558", "1.5468", "1.65", "1.7678", "1.9038", "2.0625", "2.25", "2.475", "2.75", "3.0937", "3.5357", "4.125", "4.95", "6.1875", "8.25", "12.375", "24.75"], ["1.076", "1.1739", "1.2857", "1.4142", "1.5631", "1.7368", "1.9411", "2.1838", "2.475", "2.8285", "3.2637", "3.8076", "4.5", "5.4", "6.6", "8.25", "10.6071", "14.1428", "19.8", "29.7", "49.5", "99.0", "297.0"], ["1.125", "1.2857", "1.4785", "1.712", "1.9973", "2.3498", "2.7904", "3.3485", "4.066", "5.0043", "6.2554", "7.9615", "10.35", "13.8", "18.975", "27.1071", "40.6607", "65.0571", "113.85", "227.7", "569.2501", "2277.0031"], ["1.1785", "1.4142", "1.712", "2.0924", "2.5848", "3.231", "4.0926", "5.2619", "6.881", "9.1747", "12.5109", "17.5153", "25.3", "37.95", "59.6357", "99.3928", "178.9071", "357.8143", "834.9005", "2504.7058", "12523.5607"], ["1.2375", "1.5631", "1.9973", "2.5848", "3.3925", "4.5234", "6.1389", "8.5001", "12.0418", "17.5153", "26.273", "40.8692", "66.4125", "113.85", "208.725", "417.45", "939.2628", "2504.7058", "8766.4925", "52600.8182"], ["1.3026", "1.7368", "2.3498", "3.231", "4.5234", "6.462", "9.4445", "14.1668", "21.8942", "35.0307", "58.3846", "102.173", "189.75", "379.5", "834.9005", "2087.2513", "6261.7803", "25047.4383", "175345.3772"], ["1.375", "1.9411", "2.7904", "4.0926", "6.1389", "9.4445", "14.9539", "24.47", "41.599", "73.9538", "138.6634", "277.3269", "600.875", "1442.1017", "3965.79", "13219.3884", "59488.0423", "475961.5384"], ["1.4558", "2.1838", "3.3485", "5.2619", "8.5001", "14.1668", "24.47", "44.046", "83.198", "166.3961", "356.5632", "831.981", "2163.1542", "6489.4628", "23795.2169", "118976.0846", "1071428.5714"], ["1.5468", "2.475", "4.066", "6.881", "12.0418", "21.8942", "41.599", "83.198", "176.7959", "404.105", "1010.2628", "2828.7411", "9193.3956", "36774.2654", "202288.5165", "2024539.8773"], ["1.65", "2.8285", "5.0043", "9.1747", "17.5153", "35.0307", "73.9538", "166.3961", "404.105", "1077.6143", "3232.843", "11315.0616", "49031.7468", "294205.052", "3245901.6393"], ["1.7678", "3.2637", "6.2554", "12.5109", "26.273", "58.3846", "138.6634", "356.5632", "1010.2628", "3232.843", "12123.2901", "56577.8946", "367756.315", "4419642.8571"], ["1.9038", "3.8076", "7.9615", "17.5153", "40.8692", "102.173", "277.3269", "831.981", "2828.7411", "11315.0616", "56577.8946", "396158.4633", "5156250.0"], ["2.0625", "4.5", "10.35", "25.3", "66.4125", "189.75", "600.875", "2163.1542", "9193.3956", "49031.7468", "367756.315", "5156250.0"], ["2.25", "5.4", "13.8", "37.95", "113.85", "379.5", "1442.1017", "6489.4628", "36774.2654", "294205.052", "4419642.8571"], ["2.475", "6.6", "18.975", "59.6357", "208.725", "834.9005", "3965.79", "23795.2169", "202288.5165", "3245901.6393"], ["2.75", "8.25", "27.1071", "99.3928", "417.45", "2087.2513", "13219.3884", "118976.0846", "2024539.8773"], ["3.0937", "10.6071", "40.6607", "178.9071", "939.2628", "6261.7803", "59488.0423", "1071428.5714"], ["3.5357", "14.1428", "65.0571", "357.8143", "2504.7058", "25047.4383", "475961.5384"], ["4.125", "19.8", "113.85", "834.9005", "8766.4925", "175345.3772"], ["4.95", "29.7", "227.7", "2504.7058", "52600.8182"], ["6.1875", "49.5", "569.2501", "12523.5607"], ["8.25", "99.0", "2277.0031"], ["12.375", "297.0"], ["24.75"]] 
    }'
);