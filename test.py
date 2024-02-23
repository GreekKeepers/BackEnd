import asyncio
import requests


def main():
    res = requests.post(
        "http://127.0.0.1:8585/api/user/register",
        json={
            "username": "YeahNotSewerSide",
            "password": "qw78as45QW&*AS$%!@#",
        },
    )
    print(res.content)

    res = requests.post(
        "http://127.0.0.1:8585/api/user/login",
        json={"login": "YeahNotSewerSide", "password": "qw78as45QW&*AS$%!@#"},
    )

    print(res.content)

    res = requests.get(
        "http://127.0.0.1:8585/api/user/1",
        headers={
            "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6IlllYWhOb3RTZXdlclNpZGUiLCJleHAiOjEwMCwiaWF0IjoxMDAsImF1ZCI6IiJ9.5WZGk8qJFt0RBQG7yXxvNtIVjhXT1nrjeD7mkSMbRiY"
        },
    )

    print(res.content)

    res = requests.get(
        "http://127.0.0.1:8585/api/invoice/qr",
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
        '{"type":"Auth", "token":"eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6MSwiZXhwIjoxMDAsImlhdCI6MTcwODY0MDgxMCwiYXVkIjoiIn0.J1WAxOeFkMZR2avbpsH4zmJQ5rdnjguVnTii7bZcpGs"}'
    )

    ws.send('{"type":"MakeBet", "game_id":0, "amount":"10000000", "difficulty":0}')


def web_sockets():
    websocket.enableTrace(True)
    ws = websocket.WebSocketApp(
        "ws://127.0.0.1:8585/api/updates",
        on_open=on_open,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close,
        # header={"X-Forwarded-For": "192.168.0.1:555"},
    )

    ws.run_forever(
        dispatcher=rel, reconnect=5
    )  # Set dispatcher to automatic reconnection, 5 second reconnect delay if connection closed unexpectedly
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()


if __name__ == "__main__":
    web_sockets()
