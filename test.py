import asyncio
import requests


def main():
    res = requests.post(
        "https://game.greekkeepers.io/api/user/register",
        json={
            "username": "YeahNotSewerSide",
            "password": "qw78as45QW&*AS$%!@#",
        },
    )
    print(res.content)

    res = requests.post(
        "http://127.0.0.1:8282/user/login",
        json={"login": "YeahNotSewerSide", "password": "password"},
    )

    print(res.content)

    res = requests.get(
        "http://127.0.0.1:8282/user/1",
        # headers={
        #     "Authorization": "Bearer eyJhbGciOiJIUzI1NiJ9.eyJpc3MiOm51bGwsInN1YiI6IlllYWhOb3RTZXdlclNpZGUiLCJleHAiOjEwMCwiaWF0IjoxMDAsImF1ZCI6IiJ9.5WZGk8qJFt0RBQG7yXxvNtIVjhXT1nrjeD7mkSMbRiY"
        # },
    )

    print(res.content)


if __name__ == "__main__":
    main()
