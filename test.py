import asyncio
import requests


def main():

    # res = requests.post(
    #    "http://127.0.0.1:8282/api/game/CoinFlip",
    # )
    # print(res.content)

    res = requests.post(
        "http://127.0.0.1:8282/game/list",
    )
    print(res.content)

    res = requests.post(
        "http://127.0.0.1:8282/user/register",
        json={
            "username": "YeahNotSewerSide",
            "password": "qw78as45QW&*AS$%!@#",
        },
    )
    print(res.content)

    res = requests.post(
        "http://127.0.0.1:8282/user/login",
        json={"login": "YeahNotSewerSide", "password": "qw78as45QW&*AS$%!@#"},
    )

    print(res.content)

    # res = requests.get(
    #    "http://127.0.0.1:8282/user",
    #    headers={
    #        "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MywiZXhwIjoxMDAsImlhdCI6MTcwOTExNzY0OCwiYXVkIjoiIn0.hZB78_osuq8nSCakxRWVfOiCuFWnckQJ4KEetUlFqO4"
    #    },
    # )

    # print(res.content)

    # for i in range(10):
    #    res = requests.get(
    #        "http://127.0.0.1:8282/invoice/prices",
    #        headers={
    #            "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MywiZXhwIjoxMDAsImlhdCI6MTcwOTExNzY0OCwiYXVkIjoiIn0.hZB78_osuq8nSCakxRWVfOiCuFWnckQJ4KEetUlFqO4"
    #        },
    #        json={"amount": 10, "currency": "BTC_BITCOIN"},
    #    )

    #    print(res.content)

    res = requests.post(
        "https://game.greekkeepers.io/api/invoice/create",
        headers={
            "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MSwiZXhwIjoxMDAsImlhdCI6MTcxMDM1NjcxNywiYXVkIjoiIn0.AmtAwkj-RDX1jDnxghHr_va_86BvSbZYIlP7bMQlNyg"
        },
        json={"amount": 10, "currency": "BTC_BITCOIN"},
    )

    print(res.content)

    res = requests.get(
        "http://127.0.0.1:8282/invoice/qr/79255f6b4ac72de420c01e265f161b3baf3179ac8894131e8a1a68a26515cd75cf02a3636e4e816ccbbbb4386c154ed45ce8fb37511ba540a83dca6638246ab3",
    )

    print(res.content)


import websocket
import _thread
import time
import rel
import json


def on_message(ws, message):
    print(message)


def on_error(ws, error):
    print(error)


def on_close(ws, close_status_code, close_msg):
    print("### closed ###")


def on_open(ws):
    print("Opened connection")
    ws.send('{"type":"SubscribeBets", "payload":[1,2,3,4,5,6,7,8,10,11,12]}')

    ws.send(
        '{"type":"Auth", "token":"eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MSwiZXhwIjoxMDAsImlhdCI6MTcxMDQyNzczMSwiYXVkIjoiIn0.ioWWihcI8wx2AAoIVJf1siTs2krU5gFXA1TQZME5f3w"}'
    )

    # creating user seed

    seed_data = {"type": "NewClientSeed", "seed": "Insane 100%rate win seed"}
    ws.send(json.dumps(seed_data))

    seed_data = {"type": "NewServerSeed"}
    ws.send(json.dumps(seed_data))

    bet_data = {
        "type": "MakeBet",
        "game_id": 1,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"is_heads":true}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 100,
    }
    ws.send(json.dumps(bet_data))

    # DICE bet
    bet_data = {
        "type": "MakeBet",
        "game_id": 3,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"roll_over":true, "multiplier":"2.0204"}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 5,
    }
    ws.send(json.dumps(bet_data))

    # RPS bet
    bet_data = {
        "type": "MakeBet",
        "game_id": 4,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"action":0}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 5,
    }
    ws.send(json.dumps(bet_data))

    # Race bet
    bet_data = {
        "type": "MakeBet",
        "game_id": 5,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"car":0}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 100,
    }
    ws.send(json.dumps(bet_data))

    # Wheel bet
    bet_data = {
        "type": "MakeBet",
        "game_id": 6,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"risk":2, "num_sectors":4}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 100,
    }
    ws.send(json.dumps(bet_data))

    # Plinko
    bet_data = {
        "type": "MakeBet",
        "game_id": 12,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"num_rows":16, "risk":0}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 100,
    }
    ws.send(json.dumps(bet_data))

    return

    # Statefull bet
    bet_data = {
        "type": "MakeBet",
        "game_id": 7,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"num":12321322, "end_game": false}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 100,
    }
    ws.send(json.dumps(bet_data))

    bet_data = {
        "type": "ContinueGame",
        "game_id": 7,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"num":12321322, "end_game": false}',
    }
    ws.send(json.dumps(bet_data))

    # Mines bet

    bet_data = {
        "type": "GetState",
        "game_id": 8,
        "coin_id": 1,
    }
    ws.send(json.dumps(bet_data))

    tiles = [False] * 25
    tiles[0] = True
    tiles[6] = True

    bet_data = {
        "type": "MakeBet",
        "game_id": 8,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"num_mines":1, "cashout":false, "tiles": [false,false,false,false,true,false,false,false,false,false,false,false,false,true,false,false,false,false,false,false,false,false,false,false,false]}',
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 100,
    }
    ws.send(json.dumps(bet_data))

    bet_data = {
        "type": "ContinueGame",
        "game_id": 8,
        "coin_id": 1,
        "user_id": 0,
        "data": '{ "cashout":false, "tiles": [false,true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false]}',
    }
    ws.send(json.dumps(bet_data))

    # Poker bet
    bet_data = {
        "type": "MakeBet",
        "game_id": 11,
        "coin_id": 1,
        "user_id": 0,
        "data": "{}",  # always empty for poker
        "amount": "1",
        "stop_loss": 0,
        "stop_win": 0,
        "num_games": 1,
    }
    ws.send(json.dumps(bet_data))

    bet_data = {
        "type": "ContinueGame",
        "game_id": 11,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"to_replace":[false,false,true,false,false]}',
    }
    ws.send(json.dumps(bet_data))


def web_sockets():
    # websocket.enableTrace(True)
    ws = websocket.WebSocketApp(
        "ws://127.0.0.1:8282/updates",
        on_open=on_open,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        header={"X-Forwarded-For": "192.168.0.1:555"},
    )

    ws.run_forever(
        dispatcher=rel, reconnect=5
    )  # Set dispatcher to automatic reconnection, 5 second reconnect delay if connection closed unexpectedly
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()


if __name__ == "__main__":
    web_sockets()
