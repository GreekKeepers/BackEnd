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

CREATE TABLE IF NOT EXISTS RefreshToken (
    token TEXT PRIMARY KEY,
    user_id BIGSERIAL NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    creation_date TIMESTAMP DEFAULT NOW()
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

CREATE UNIQUE INDEX state_unique_idx ON GameState(game_id, user_id, coin_id, userseed_id, serverseed_id);


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
    1000
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
    '{"profit_coef":"1.94"}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Rocket',
    '{"profit_coef":"1.94"}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Crash',
    '{"profit_coef":"1.94"}'
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
    'Thimbles',
    '{"profit_coef":"2.82", "cars_amount":3}'
);

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'CarRace',
    '{"profit_coef":"1.94", "cars_amount":2}'
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

INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Poker',
    '{
        "initial_deck": [
   {
      "number":1,
      "suit":0
   },
   {
      "number":2,
      "suit":0
   },
   {
      "number":3,
      "suit":0
   },
   {
      "number":4,
      "suit":0
   },
   {
      "number":5,
      "suit":0
   },
   {
      "number":6,
      "suit":0
   },
   {
      "number":7,
      "suit":0
   },
   {
      "number":8,
      "suit":0
   },
   {
      "number":9,
      "suit":0
   },
   {
      "number":10,
      "suit":0
   },
   {
      "number":11,
      "suit":0
   },
   {
      "number":12,
      "suit":0
   },
   {
      "number":13,
      "suit":0
   },
   {
      "number":1,
      "suit":1
   },
   {
      "number":2,
      "suit":1
   },
   {
      "number":3,
      "suit":1
   },
   {
      "number":4,
      "suit":1
   },
   {
      "number":5,
      "suit":1
   },
   {
      "number":6,
      "suit":1
   },
   {
      "number":7,
      "suit":1
   },
   {
      "number":8,
      "suit":1
   },
   {
      "number":9,
      "suit":1
   },
   {
      "number":10,
      "suit":1
   },
   {
      "number":11,
      "suit":1
   },
   {
      "number":12,
      "suit":1
   },
   {
      "number":13,
      "suit":1
   },
   {
      "number":1,
      "suit":2
   },
   {
      "number":2,
      "suit":2
   },
   {
      "number":3,
      "suit":2
   },
   {
      "number":4,
      "suit":2
   },
   {
      "number":5,
      "suit":2
   },
   {
      "number":6,
      "suit":2
   },
   {
      "number":7,
      "suit":2
   },
   {
      "number":8,
      "suit":2
   },
   {
      "number":9,
      "suit":2
   },
   {
      "number":10,
      "suit":2
   },
   {
      "number":11,
      "suit":2
   },
   {
      "number":12,
      "suit":2
   },
   {
      "number":13,
      "suit":2
   },
   {
      "number":1,
      "suit":3
   },
   {
      "number":2,
      "suit":3
   },
   {
      "number":3,
      "suit":3
   },
   {
      "number":4,
      "suit":3
   },
   {
      "number":5,
      "suit":3
   },
   {
      "number":6,
      "suit":3
   },
   {
      "number":7,
      "suit":3
   },
   {
      "number":8,
      "suit":3
   },
   {
      "number":9,
      "suit":3
   },
   {
      "number":10,
      "suit":3
   },
   {
      "number":11,
      "suit":3
   },
   {
      "number":12,
      "suit":3
   },
   {
      "number":13,
      "suit":3
   }
],
    "multipliers": ["0.0","0.0","0.0","0.0","0.0","0.0","0.0","0.0","0.0","0.0"]
    }'
);


INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Plinko',
    '{"multipliers":[[["20.5", "4.0", "0.9", "0.6", "0.4", "0.6", "0.9", "4.0", "20.5"], ["45.0", "8.0", "0.9", "0.6", "0.4", "0.4", "0.6", "0.9", "8.0", "45.0"], ["47.0", "8.0", "2.0", "0.9", "0.6", "0.4", "0.6", "0.9", "2.0", "8.0", "47.0"], ["65.0", "17.0", "4.0", "0.9", "0.6", "0.4", "0.4", "0.6", "0.9", "4.0", "17.0", "65.0"], ["70.0", "16.0", "3.0", "2.0", "0.9", "0.6", "0.4", "0.6", "0.9", "2.0", "3.0", "16.0", "70.0"], ["80.0", "17.0", "6.0", "4.0", "0.9", "0.6", "0.4", "0.4", "0.6", "0.9", "4.0", "6.0", "17.0", "80.0"], ["100.0", "45.0", "9.0", "3.0", "1.1", "0.9", "0.6", "0.4", "0.6", "0.9", "1.1", "3.0", "9.0", "45.0", "100.0"], ["110.0", "45.0", "13.0", "9.0", "1.1", "0.9", "0.6", "0.4", "0.4", "0.6", "0.9", "1.1", "9.0", "13.0", "45.0", "110.0"], ["120.0", "28.0", "24.0", "8.0", "2.0", "0.9", "0.9", "0.6", "0.4", "0.6", "0.9", "0.9", "2.0", "8.0", "24.0", "28.0", "120.0"]], [["50.0", "4.0", "0.5", "0.4", "0.2", "0.4", "0.5", "4.0", "50.0"], ["66.0", "12.0", "0.5", "0.4", "0.2", "0.2", "0.4", "0.5", "12.0", "66.0"], ["95.0", "10.0", "2.0", "0.9", "0.4", "0.2", "0.4", "0.9", "2.0", "10.0", "95.0"], ["150.0", "20.0", "5.0", "0.6", "0.5", "0.2", "0.2", "0.5", "0.6", "5.0", "20.0", "150.0"], ["175.0", "35.0", "4.0", "2.0", "0.6", "0.4", "0.2", "0.4", "0.6", "2.0", "4.0", "35.0", "175.0"], ["250.0", "44.0", "7.0", "4.0", "0.9", "0.4", "0.2", "0.2", "0.4", "0.9", "4.0", "7.0", "44.0", "250.0"], ["390.0", "55.0", "15.0", "4.0", "0.9", "0.8", "0.4", "0.2", "0.4", "0.8", "0.9", "4.0", "15.0", "55.0", "390.0"], ["500.0", "60.0", "22.0", "8.0", "2.0", "0.9", "0.4", "0.2", "0.2", "0.4", "0.9", "2.0", "8.0", "22.0", "60.0", "500.0"], ["520.0", "80.0", "15.0", "10.0", "3.0", "2.0", "0.5", "0.3", "0.2", "0.3", "0.5", "2.0", "3.0", "10.0", "15.0", "80.0", "520.0"]], [["100.0", "0.6", "0.2", "0.2", "0.1", "0.2", "0.2", "0.6", "100.0"], ["143.0", "5.0", "0.7", "0.3", "0.1", "0.1", "0.3", "0.7", "5.0", "143.0"], ["170.0", "15.0", "2.0", "0.3", "0.2", "0.1", "0.2", "0.3", "2.0", "15.0", "170.0"], ["290.0", "15.0", "2.0", "0.8", "0.5", "0.3", "0.3", "0.5", "0.8", "2.0", "15.0", "290.0"], ["380.0", "20.0", "4.0", "2.0", "0.8", "0.3", "0.1", "0.3", "0.8", "2.0", "4.0", "20.0", "380.0"], ["500.0", "68.0", "7.0", "2.0", "0.9", "0.4", "0.2", "0.2", "0.4", "0.9", "2.0", "7.0", "68.0", "500.0"], ["770.0", "65.0", "13.0", "3.0", "2.0", "0.5", "0.3", "0.1", "0.3", "0.5", "2.0", "3.0", "13.0", "65.0", "770.0"], ["800.0", "200.0", "50.0", "5.0", "0.8", "0.5", "0.3", "0.1", "0.1", "0.3", "0.5", "0.8", "5.0", "50.0", "200.0", "800.0"], ["1000.0", "280.0", "30.0", "15.0", "1.5", "0.6", "0.5", "0.4", "0.1", "0.4", "0.5", "0.6", "1.5", "15.0", "30.0", "280.0", "1000.0"]]]}'
);



INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Apples',
    '{
        "difficulties": [
          {
            "mines": 1,
            "total_spaces": 4
          },
          {
            "mines": 1,
            "total_spaces": 3
          },
          {
            "mines": 1,
            "total_spaces": 2
          },
          {
            "mines": 2,
            "total_spaces": 3
          },
          {
            "mines": 3,
            "total_spaces": 4
          }
        ],
        "multipliers":[
          [
            "1.32",
            "1.76",
            "2.34",
            "3.12",
            "4.17",
            "5.56",
            "7.41",
            "9.88",
            "13.18"
          ],
          [
            "1.48",
            "2.22",
            "3.34",
            "5.01",
            "7.51",
            "11.27",
            "16.91",
            "25.37",
            "38.05"
          ],
          [
            "1.98",
            "3.96",
            "7.92",
            "15.84",
            "31.68",
            "63.36",
            "126.72",
            "253.44",
            "506.88"
          ],
          [
            "2.97",
            "8.91",
            "26.73",
            "80.19",
            "240.57",
            "721.71",
            "2165.13",
            "6495.39",
            "19486.17"
          ],
          [
            "3.96",
            "15.84",
            "63.36",
            "253.44",
            "1013.76",
            "4055.04",
            "16220.16",
            "64880.64",
            "259522.56"
          ]
        ]
    }'
);



INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Slots',
    '{
        "num_outcomes": 343,
        "multipliers": ["5", "3", "3", "3", "3", "3", "3", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "2", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "10", "0", "0", "10", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "12", "0", "12", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "20", "20", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "45", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "100"] 
    }'
);


INSERT INTO Game(
    name,
    parameters
) VALUES (
    'Roulette',
    '{
        "zero_coef":"52",
        "num_coef":"34.8148",
        "num2_coef":"17.3752",
        "num4_coef":"8.6957",
        "num12_coef":"2.8986",
        "num18_coef":"1.9322"
    }'
);
