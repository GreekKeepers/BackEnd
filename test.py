import asyncio
import requests


def main():
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

    res = requests.get(
        "http://127.0.0.1:8282/user/1",
        headers={
            "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6IlllYWhOb3RTZXdlclNpZGUiLCJleHAiOjEwMCwiaWF0IjoxMDAsImF1ZCI6IiJ9.5WZGk8qJFt0RBQG7yXxvNtIVjhXT1nrjeD7mkSMbRiY"
        },
    )

    print(res.content)

    res = requests.get(
        "http://127.0.0.1:8282/invoice/qr",
        headers={
            "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MSwiZXhwIjoxMDAsImlhdCI6MTcwODM3OTA3OCwiYXVkIjoiIn0.VwA0VVVPxMnXAHaVlxR0UnnXc1DMlWvTTva4RIBvz9M"
        },
        json={"data": "some data"},
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
    ws.send('{"type":"SubscribeBets", "payload":[0]}')

    ws.send(
        '{"type":"Auth", "token":"eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MywiZXhwIjoxMDAsImlhdCI6MTcwODgxMDQyMCwiYXVkIjoiIn0.0T8V8_ekmj6MJVl5EvyQS__vvAuqHRD_CL83IG1dBE4"}'
    )

    # creating user seed

    seed_data = {"type": "NewClientSeed", "seed": "Insane 100%rate win seed"}

    bet_data = {
        "type": "MakeBet",
        "game_id": 1,
        "coin_id": 1,
        "user_id": 0,
        "data": '{"is_heads":true}',
        "amount": "1",
        "difficulty": 0,
        "stop_loss": 0,
        "stop_win": 0,
    }
    ws.send(json.dumps(bet_data))


def web_sockets():
    websocket.enableTrace(True)
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
