import os

from locust import HttpUser, task, between, constant

TOKEN = ""


class QuickstartUser(HttpUser):
    wait_time = between(1, 3)

    def on_start(self):
        response = self.client.post(
            "/api/user/login",
            json={"login": "bruh", "password": "bruh"},
        )
        TOKEN = response.json()["body"]["access_token"]

    @task
    def test_bets(self):
        self.client.get("/api/bets/list")

    @task
    def test_totals(self):
        self.client.get(
            "/api/general/totals",
        )

    @task
    def user(self):
        self.client.get("/api/user", headers={"authorization": "Bearer " + TOKEN})

    @task
    def user(self):
        self.client.get(
            "/api/user/seed/server", headers={"authorization": "Bearer " + TOKEN}
        )

    @task
    def user(self):
        self.client.get(
            "/api/user/seed/client", headers={"authorization": "Bearer " + TOKEN}
        )

    @task
    def user(self):
        self.client.get("/api/game/list", headers={"authorization": "Bearer " + TOKEN})
