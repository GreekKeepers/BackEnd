import json
import rel
import time
import _thread
import websocket
import asyncio
import requests


def main():

    res = requests.post(
        "https://rew.greekkeepers.io/api/user/login",
        json={"login": "TestSewer", "password": "qweqwe"},
    )
    print(res.json())
    access_token = res.json()['body']['access_token']

    res = requests.patch(
        "https://rew.greekkeepers.io/api/user/username",
        json={
            "nickname": "YeahNotSewerSide",
        },
        headers={
            "Authorization": f"Bearer {access_token}"
        },
    )
    print(res.content)
    # res = requests.post(
    #    "http://127.0.0.1:8282/api/game/CoinFlip",
    # )
    # print(res.content)
    # res = requests.post(
    #    "http://127.0.0.1:8282/general/leaderboard/volume/all",
    # )
    # print(res.content)

    # res = requests.post(
    #    "http://127.0.0.1:8282/game/list",
    # )
    # print(res.content)

    # res = requests.post(
    #    "http://127.0.0.1:8282/user/register",
    #    json={
    #        "username": "YeahNotSewerSide",
    #        "password": "qw78as45QW&*AS$%!@#",
    #    },
    # )
    # print(res.content)

    # res = requests.post(
    #    "http://127.0.0.1:8282/user/login",
    #    json={"login": "YeahNotSewerSide", "password": "qw78as45QW&*AS$%!@#"},
    # )

    # print(res.content)

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

    # res = requests.post(
    #    "https://game.greekkeepers.io/api/invoice/create",
    #    headers={
    #        "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MSwiZXhwIjoxMDAsImlhdCI6MTcxMDM1NjcxNywiYXVkIjoiIn0.AmtAwkj-RDX1jDnxghHr_va_86BvSbZYIlP7bMQlNyg"
    #    },
    #    json={"amount": 10, "currency": "BTC_BITCOIN"},
    # )

    # print(res.content)

    # res = requests.get(
    #    "http://127.0.0.1:8282/invoice/qr/79255f6b4ac72de420c01e265f161b3baf3179ac8894131e8a1a68a26515cd75cf02a3636e4e816ccbbbb4386c154ed45ce8fb37511ba540a83dca6638246ab3",
    # )

    # print(res.content)


def on_message(ws, message):
    print(message)

    # bet_data = {
    #    "type": "MakeBet",
    #    "game_id": 17,
    #    "coin_id": 1,
    #    "user_id": 0,
    #    "data": "{\"buy_free_spins\": false, \"use_free_spins\": false}",
    #    "amount": "10000",
    #    "stop_loss": 0,
    #    "stop_win": 0,
    #    "num_games": 1,
    # }
    # ws.send(json.dumps(bet_data))

    # bet_data = {
    #    "type": "ContinueGame",
    #    "game_id": 17,
    #    "coin_id": 1,
    #    "data": "{\"buy_free_spins\": false, \"use_free_spins\": true}",
    # }
    # ws.send(json.dumps(bet_data))
    # input()


def on_error(ws, error):
    print(error)


def on_close(ws, close_status_code, close_msg):
    print("### closed ###")


def on_open(ws):
    res = requests.post(
        "http://127.0.0.1:8282/user/login",
        json={"login": "TestSewer", "password": "qweqwe"},
    )
    print(res.json())
    access_token = res.json()['body']['access_token']
    print("Opened connection")

    # chat
    ws.send('{"type":"SubscribeChatRoom", "room":17}')

    ws.send(
        json.dumps({"type": "Auth", "token": access_token})
    )

    # creating user seed

    # seed_data = {"type": "NewClientSeed", "seed": "Insane 100%rate win seed"}
    # ws.send(json.dumps(seed_data))

    # seed_data = {"type": "NewServerSeed"}
    # ws.send(json.dumps(seed_data))

    # chat

    msg_data = {
        "type": "NewMessage",
        "message": "Hello, World",
        "mentions": [],
        "chat_room": 17
    }
    ws.send(json.dumps(msg_data))

    # bet_data = {
    #    "type": "MakeBet",
    #    "game_id": 17,
    #    "coin_id": 1,
    #    "user_id": 0,
    #    "data": "{\"buy_free_spins\": true, \"use_free_spins\": false}",
    #    "amount": "20000",
    #    "stop_loss": 0,
    #    "stop_win": 0,
    #    "num_games": 1,
    # }
    # ws.send(json.dumps(bet_data))


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
    # main()
    web_sockets()
