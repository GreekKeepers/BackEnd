use crate::{
    config::DatabaseSettings,
    models::db_models::{Amount, Bet, Coin, Game, ServerSeed, User, UserSeed},
    tools::blake_hash,
};

use rust_decimal::Decimal;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

#[derive(Debug, Clone)]
pub struct DB {
    db_pool: PgPool,
}

impl DB {
    pub async fn new(settings: &DatabaseSettings) -> Self {
        let connection_string = settings.connection_string();
        info!("Connecting to database: {}", &connection_string);

        let db_pool = PgPoolOptions::new()
            .max_connections(10)
            .connect_lazy(&connection_string)
            .expect("URI string should be correct");
        Self { db_pool }
    }

    pub async fn fetch_bets_for_gamename(
        &self,
        game_name: &str,
        limit: i64,
    ) -> Result<Vec<Bet>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Bet,
            r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.game_id,
                Bet.user_id,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes
            FROM Bet
            INNER JOIN Game ON Bet.game_id=Game.id
            WHERE Game.name=$1
            ORDER BY Bet.id DESC
            LIMIT $2
            "#,
            game_name,
            limit
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn fetch_all_latest_bets(&self, limit: i64) -> Result<Vec<Bet>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Bet,
            r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.game_id,
                Bet.user_id,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes
            FROM Bet
            ORDER BY Bet.id DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn fetch_user(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            User,
            r#"
            SELECT * 
            FROM Users
            WHERE id=$1
            "#,
            id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn fetch_all_games(&self) -> Result<Vec<Game>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Game,
            r#"
            SELECT *
            FROM Game
            "#
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn add_invoice(
        &self,
        id: &str,
        merchant_id: &str,
        order_id: &str,
        status: i32,
        pay_url: String,
        user_id: i64,
        amount: Decimal,
        currency: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Invoice(
                id,
                merchant_id,
                order_id,
                status,
                pay_url,
                user_id,
                amount,
                currency
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8
            )
            "#,
            id,
            merchant_id,
            order_id,
            status,
            pay_url,
            user_id,
            amount,
            currency
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_coins(&self) -> Result<Vec<Coin>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Coin,
            r#"
            SELECT *
            FROM Coin
            "#
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn login_user(
        &self,
        login: &str,
        password: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            User,
            r#"
            SELECT * 
            FROM Users
            WHERE login=$1 AND password=$2
            LIMIT 1
            "#,
            login,
            password
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn change_username(&self, id: i64, username: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE Users
            SET username = $1
            WHERE id = $2
            "#,
            username,
            id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn register_user(
        &self,
        login: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Users(
                login,
                username,
                password
            ) VALUES (
                $1,
                $1,
                $2
            )
            "#,
            login,
            password_hash
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query_as_unchecked!(
            User,
            r#"
            SELECT *
            FROM Users
            WHERE Users.login=$1
            LIMIT 1
            "#,
            login,
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn fetch_coin(&self, name: &str) -> Result<Option<Coin>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Coin,
            r#"
            SELECT * 
            FROM Coin
            WHERE name=$1
            "#,
            name
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn fetch_amounts(&self, id: i64) -> Result<Vec<Amount>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Amount,
            r#"
            SELECT Coin.name, Amount.amount 
            FROM Amount
            INNER JOIN Coin ON Amount.coin_id = Coin.id
            INNER JOIN Users ON Amount.user_id = Users.id
            WHERE Users.id = $1
            "#,
            id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn fetch_amount(
        &self,
        user_id: i64,
        coin_id: i64,
    ) -> Result<Option<Decimal>, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT Amount.amount as amount
            FROM Amount
            WHERE user_id=$1 AND coin_id=$2
            LIMIT 1
            "#,
            user_id,
            coin_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map(|v| v.amount)
    }

    pub async fn fetch_game(&self, game_id: i64) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Game,
            r#"
            SELECT *
            FROM Game
            WHERE id=$1
            LIMIT 1
            "#,
            game_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn init_amount(
        &self,
        user_id: i64,
        coin_id: i64,
        amount: Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Amount(
                user_id,
                coin_id,
                amount
            ) VALUES (
                $1,
                $2,
                $3
            )
            "#,
            user_id,
            coin_id,
            amount
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn new_user_seed(&self, user_id: i64, seed: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO UserSeed(
                user_id,
                user_seed
            ) VALUES (
                $1,
                $2
            )
            "#,
            user_id,
            seed
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_current_user_seed(&self, user_id: i64) -> Result<UserSeed, sqlx::Error> {
        sqlx::query_as_unchecked!(
            UserSeed,
            r#"
            SELECT * FROM UserSeed
            WHERE user_id = $1
            ORDER BY id DESC
            LIMIT 1
            "#,
            user_id,
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn fetch_current_server_seed(&self, user_id: i64) -> Result<ServerSeed, sqlx::Error> {
        let mut res = sqlx::query_as_unchecked!(
            ServerSeed,
            r#"
            SELECT * FROM ServerSeed
            WHERE user_id = $1 AND revealed = FALSE
            LIMIT 1
            "#,
            user_id,
        )
        .fetch_one(&self.db_pool)
        .await?;

        res.server_seed = blake_hash(&res.server_seed);

        Ok(res)
    }

    pub async fn new_server_seed(&self, user_id: i64, seed: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO ServerSeed(
                user_id,
                server_seed,
                revealed
            ) VALUES (
                $1,
                $2,
                False
            )
            "#,
            user_id,
            seed
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn reveal_last_seed(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE ServerSeed
            SET revealed = True
            WHERE user_id = $1 AND revealed = False
            "#,
            user_id,
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn decrease_balance(
        &self,
        user_id: i64,
        coin_id: i64,
        amount: Decimal,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query!(
            r#"
            UPDATE Amount
            SET amount = amount - $3
            WHERE user_id = $1 AND coin_id = $2 and amount >= amount
            "#,
            user_id,
            coin_id,
            amount
        )
        .execute(&self.db_pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn increase_balance(
        &self,
        user_id: i64,
        coin_id: i64,
        amount: &Decimal,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query!(
            r#"
            UPDATE Amount
            SET amount = amount + $3
            WHERE user_id = $1 AND coin_id = $2 
            "#,
            user_id,
            coin_id,
            amount
        )
        .execute(&self.db_pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn place_bet(
        &self,
        amount: Decimal,
        profit: Decimal,
        num_games: i32,
        outcomes: &str,
        bet_info: &str,
        game_id: i64,
        user_id: i64,
        coin_id: i64,
        userseed_id: i64,
        serverseed_id: i64,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Bet(
                amount,
                profit,
                num_games,
                outcomes,
                bet_info,
                game_id,
                user_id,
                coin_id,
                userseed_id,
                serverseed_id
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10
            ) RETURNING id
            "#,
            amount,
            profit,
            num_games,
            outcomes,
            bet_info,
            game_id,
            user_id,
            coin_id,
            userseed_id,
            serverseed_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map(|v| v.id)
    }
}
