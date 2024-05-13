use crate::{
    config::DatabaseSettings,
    models::{
        db_models::{
            Amount, Bet, BillineInvoice, BillineInvoiceStatus, Coin, ConnectedWallet, Game,
            GameState, Invoice, Leaderboard, OauthProvider, Partner, PartnerContact,
            PartnerProgram, PartnerSite, RefClicks, ReferalLink, RefreshToken, ServerSeed,
            SiteSubId, TimeBoundaries, Totals, User, UserSeed, UserTotals, Withdrawal,
        },
        json_requests::WithdrawRequest,
        json_responses::{AmountConnectedWallets, BetExpanded},
    },
    tools::blake_hash,
};

use chrono::{DateTime, Utc};
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

    pub async fn fetch_game_state(
        &self,
        game_id: i64,
        user_id: i64,
        coin_id: i64,
    ) -> Result<Option<GameState>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            GameState,
            r#"SELECT *
            FROM GameState
            WHERE game_id=$1 AND
                user_id=$2 AND
                coin_id=$3
            "#,
            game_id,
            user_id,
            coin_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn remove_game_state(
        &self,
        game_id: i64,
        user_id: i64,
        coin_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE 
            FROM GameState
            WHERE game_id=$1 AND
                user_id=$2 AND
                coin_id=$3
            "#,
            game_id,
            user_id,
            coin_id
        )
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }

    pub async fn change_game_state(
        &self,
        game_id: i64,
        user_id: i64,
        coin_id: i64,
        new_state: &str,
    ) -> Result<bool, sqlx::Error> {
        Ok(sqlx::query!(
            r#"UPDATE GameState
            SET state=$4
            WHERE game_id=$1 AND user_id=$2 AND coin_id=$3
            "#,
            game_id,
            user_id,
            coin_id,
            new_state
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected()
            > 0)
    }

    pub async fn insert_game_state(
        &self,
        game_id: i64,
        user_id: i64,
        uuid: &str,
        coin_id: i64,
        bet_info: &str,
        new_state: &str,
        amount: &Decimal,
        userseed_id: i64,
        serverseed_id: i64,
    ) -> Result<bool, sqlx::Error> {
        Ok(sqlx::query!(
            r#"INSERT INTO GameState(
                bet_info,
                game_id,
                user_id,
                uuid,
                coin_id,
                amount,
                userseed_id,
                serverseed_id,
                state
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9
            )
            "#,
            bet_info,
            game_id,
            user_id,
            uuid,
            coin_id,
            amount,
            userseed_id,
            serverseed_id,
            new_state
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected()
            > 0)
    }

    pub async fn fetch_bet(
        &self,
        game_id: i64,
        user_id: i64,
        uuid: &str,
        coin_id: i64,
    ) -> Result<Option<Bet>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Bet,
            r#"SELECT *
            FROM Bet
            WHERE game_id=$1 AND
                user_id=$2 AND
                uuid=$3 AND num_games=0
                AND coin_id=$4
            "#,
            game_id,
            user_id,
            uuid,
            coin_id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn fetch_bets_for_gamename(
        &self,
        game_name: &str,
        limit: i64,
    ) -> Result<Vec<BetExpanded>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BetExpanded,
            r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.state,
                Bet.uuid,
                Bet.game_id,
                Bet.user_id,
                Users.username,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes,
                Bet.profits
            FROM Bet
            INNER JOIN Game ON Bet.game_id=Game.id
            INNER JOIN Users ON Bet.user_id=Users.id
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

    pub async fn fetch_all_latest_bets(&self, limit: i64) -> Result<Vec<BetExpanded>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BetExpanded,
            r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.state,
                Bet.uuid,
                Bet.game_id,
                Bet.user_id,
                Users.username,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes,
                Bet.profits
            FROM Bet
            INNER JOIN Users ON bet.user_id = Users.id
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
            SELECT  
                id,
                registration_time,
                login,
                username,
                password,
                user_level,
                provider
            FROM Users
            WHERE id=$1
            "#,
            id
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    pub async fn fetch_user_by_login(&self, login: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            User,
            r#"
            SELECT * 
            FROM Users
            WHERE login=$1
            "#,
            login
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

    pub async fn fetch_invoice(&self, id: &str) -> Result<Invoice, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Invoice,
            r#"
            SELECT * 
            FROM Invoice
            WHERE id=$1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn fetch_tokens_to_track(&self) -> Result<String, sqlx::Error> {
        let res = sqlx::query!(
            r#"
            SELECT *
            FROM TokensToTrack
            LIMIT 1
            "#,
        )
        .fetch_one(&self.db_pool)
        .await?;
        Ok(res.tokens)
    }

    pub async fn invoice_update_status(&self, id: &str, status: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE Invoice
            SET status = $1
            WHERE id=$2
            "#,
            status,
            id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn add_invoice(
        &self,
        id: &str,
        merchant_id: &str,
        order_id: &str,
        status: i32,
        pay_url: &str,
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

    /// pending success failed
    pub async fn add_billine_invoice(
        &self,
        id: &str,
        merchant_id: &str,
        order_id: &str,
        user_id: i64,
        amount: Decimal,
        currency: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO InvoiceBilline(
                id,
                merchant_id,
                order_id,
                user_id,
                amount,
                currency
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6
            )
            "#,
            id,
            merchant_id,
            order_id,
            user_id,
            amount,
            currency
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn billine_invoice_update_status(
        &self,
        id: &str,
        status: BillineInvoiceStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE InvoiceBilline
            SET status = ($1::text)::billine_status
            WHERE id=$2
            "#,
            status.to_string(),
            id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_billine_invoice(&self, id: &str) -> Result<BillineInvoice, sqlx::Error> {
        sqlx::query_as_unchecked!(
            BillineInvoice,
            r#"
            SELECT * 
            FROM InvoiceBilline
            WHERE id=$1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await
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

    pub async fn change_password(&self, id: i64, password_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE Users
            SET password = $1
            WHERE id = $2
            "#,
            password_hash,
            id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn register_user(
        &self,
        login: &str,
        username: &str,
        provider: OauthProvider,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as_unchecked!(
            User,
            r#"
            INSERT INTO Users(
                login,
                username,
                provider,
                password
            ) VALUES (
                $1,
                $2,
                $3,
                $4
            )
            RETURNING id, registration_time, login, username, password, provider, user_level 
            "#,
            login,
            username,
            provider,
            password_hash,
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

    pub async fn fetch_coin_by_id(&self, id: i64) -> Result<Option<Coin>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Coin,
            r#"
            SELECT * 
            FROM Coin
            WHERE id=$1
            "#,
            id
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

    pub async fn create_referal_link(
        &self,
        refer_to: i64,
        link_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Referal(
                refer_to,
                link_name
                ) VALUES (
                    $1,
                    $2
                )
            ON CONFLICT (refer_to) DO UPDATE
            SET link_name=$2
            "#,
            refer_to,
            link_name
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_referal_link(&self, link_name: &str) -> Result<ReferalLink, sqlx::Error> {
        sqlx::query_as_unchecked!(
            ReferalLink,
            r#"
            SELECT *
            FROM Referal
            WHERE link_name=$1
            "#,
            link_name
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn new_referal(&self, refer_to: i64, referal: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Referals(
                refer_to,
                referal,
		refer_name
                ) VALUES (
                    $1,
                    $2,
		    (SELECT id FROM referal WHERE refer_to=$1)
                );
            "#,
            refer_to,
            referal
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn new_refresh_token(&self, user_id: i64, token: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO RefreshToken(
                token, user_id
                ) VALUES (
                    $1,
                    $2
                )
            "#,
            token,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_refresh_token(
        &self,
        token: &str,
        user_id: i64,
    ) -> Result<RefreshToken, sqlx::Error> {
        let token = sqlx::query_as_unchecked!(
            RefreshToken,
            r#"
            SELECT * FROM RefreshToken WHERE token=$1 AND user_id=$2
            "#,
            token,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(token)
    }

    pub async fn remove_refresh_token(
        &self,
        token: &str,
        user_id: i64,
    ) -> Result<bool, sqlx::Error> {
        let res = sqlx::query!(
            r#"
            DELETE FROM RefreshToken WHERE token=$1 AND user_id=$2
            "#,
            token,
            user_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(res.rows_affected() != 0)
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

    pub async fn fetch_totals(&self) -> Result<Totals, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Totals,
            r#"
            SELECT 
            	COUNT(bet.id) as bets_amount,
            	(SELECT COUNT(Users.id) FROM Users) as player_amount,
            	SUM((Bet.amount*Bet.num_games)/Coin.price) as sum
            FROM Bet
            INNER JOIN Coin ON Coin.id=Bet.coin_id
            "#
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn fetch_bets_for_user(
        &self,
        user_id: i64,
        last_id: Option<i64>,
        page_size: i64,
    ) -> Result<Vec<BetExpanded>, sqlx::Error> {
        if let Some(last_id) = last_id {
            sqlx::query_as_unchecked!(
                BetExpanded,
                r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.state,
                Bet.uuid,
                Bet.game_id,
                Bet.user_id,
                Users.username,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes,
                Bet.profits
            FROM Bet
            INNER JOIN Users ON bet.user_id = Users.id
            WHERE bet.user_id = $1 and bet.id < $3 
            ORDER BY Bet.id DESC
            LIMIT $2 
            "#,
                user_id,
                page_size,
                last_id
            )
            .fetch_all(&self.db_pool)
            .await
        } else {
            sqlx::query_as_unchecked!(
                BetExpanded,
                r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.state,
                Bet.uuid,
                Bet.game_id,
                Bet.user_id,
                Users.username,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes,
                Bet.profits
            FROM Bet
            INNER JOIN Users ON bet.user_id = Users.id
            WHERE bet.user_id = $1 
            ORDER BY Bet.id DESC
            LIMIT $2 
            "#,
                user_id,
                page_size
            )
            .fetch_all(&self.db_pool)
            .await
        }
    }

    pub async fn fetch_bets_for_user_inc(
        &self,
        user_id: i64,
        last_id: Option<i64>,
        page_size: i64,
    ) -> Result<Vec<BetExpanded>, sqlx::Error> {
        if let Some(last_id) = last_id {
            sqlx::query_as_unchecked!(
                BetExpanded,
                r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.state,
                Bet.uuid,
                Bet.game_id,
                Bet.user_id,
                Users.username,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes,
                Bet.profits
            FROM Bet
            INNER JOIN Users ON bet.user_id = Users.id
            WHERE bet.user_id = $1 and bet.id > $3 
            ORDER BY Bet.id ASC 
            LIMIT $2 
            "#,
                user_id,
                page_size,
                last_id
            )
            .fetch_all(&self.db_pool)
            .await
        } else {
            sqlx::query_as_unchecked!(
                BetExpanded,
                r#"
            SELECT 
                Bet.id,
                Bet.timestamp,
                Bet.amount,
                Bet.profit,
                Bet.num_games,
                Bet.bet_info,
                Bet.state,
                Bet.uuid,
                Bet.game_id,
                Bet.user_id,
                Users.username,
                Bet.coin_id,
                Bet.userseed_id,
                Bet.serverseed_id,
                Bet.outcomes,
                Bet.profits
            FROM Bet
            INNER JOIN Users ON bet.user_id = Users.id
            WHERE bet.user_id = $1 
            ORDER BY Bet.id ASC 
            LIMIT $2 
            "#,
                user_id,
                page_size
            )
            .fetch_all(&self.db_pool)
            .await
        }
    }

    pub async fn latest_games(&self, user_id: i64) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query!(
            r#"
            SELECT game.name FROM game RIGHT JOIN 
                (SELECT * from bet where bet.user_id=$1 ORDER BY timestamp DESC LIMIT 2) as bets ON bets.game_id = game.id
            "#,
            user_id
        ).fetch_all(&self.db_pool).await.map(|rows| rows.into_iter().map(|row| row.name.unwrap()).collect())
    }

    pub async fn increase_amounts_by_usdt_amount(
        &self,
        user_id: i64,
        amount: &Decimal,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE Amount
        SET amount = amount + ($2*(SELECT price FROM coin WHERE id=Amount.coin_id))
        WHERE user_id = $1"#,
            user_id,
            amount
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_user_totals(&self, user_id: i64) -> Result<UserTotals, sqlx::Error> {
        sqlx::query_as_unchecked!(
            UserTotals,
            r#"
            SELECT
	            COUNT(bet.id) AS bets_amount,
	            COUNT(case when bet.amount*bet.num_games > bet.profit then 1 else null end) as lost_bets,
	            COUNT(case when bet.amount*bet.num_games <= bet.profit then 1 else null end) as won_bets,
	            SUM((bet.amount*num_games)/coin.price) as total_wagered_sum,
	            SUM(bet.profit/coin.price) as gross_profit,
	            SUM(bet.profit/coin.price) - SUM((bet.amount*num_games)/coin.price)as net_profit,
	            MAX(bet.profit/coin.price) as highest_win
            FROM Bet
            INNER JOIN Coin ON Bet.coin_id=Coin.id
            WHERE Bet.user_id=$1
            "#,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn place_bet(
        &self,
        amount: Decimal,
        profit: Decimal,
        num_games: i32,
        outcomes: &str,
        profits: &str,
        bet_info: &str,
        state: Option<&str>,
        uuid: &str,
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
                profits,
                bet_info, 
                uuid,
                game_id,
                user_id,
                coin_id,
                userseed_id,
                serverseed_id,
                state
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
                $10,
                $11,
                $12,
                $13
            ) RETURNING id
            "#,
            amount,
            profit,
            num_games,
            outcomes,
            profits,
            bet_info,
            uuid,
            game_id,
            user_id,
            coin_id,
            userseed_id,
            serverseed_id,
            state
        )
        .fetch_one(&self.db_pool)
        .await
        .map(|v| v.id)
    }

    pub async fn fetch_leaderboard_volume(
        &self,
        time_boundaries: TimeBoundaries,
        limit: i64,
    ) -> Result<Vec<Leaderboard>, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM((bet.amount*bet.num_games)/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        WHERE bet.timestamp > now() - interval '1 day'
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM((bet.amount*bet.num_games)/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        WHERE bet.timestamp > now() - interval '1 week'
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM((bet.amount*bet.num_games)/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        WHERE bet.timestamp > now() - interval '1 month'
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM((bet.amount*bet.num_games)/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1 
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    pub async fn fetch_leaderboard_profit(
        &self,
        time_boundaries: TimeBoundaries,
        limit: i64,
    ) -> Result<Vec<Leaderboard>, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM(bet.profit/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        WHERE bet.timestamp > now() - interval '1 day'
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM(bet.profit/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        WHERE bet.timestamp > now() - interval '1 week'
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM(bet.profit/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        WHERE bet.timestamp > now() - interval '1 month'
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    Leaderboard,
                    r#"
                    SELECT bet.user_id, bet.total, Users.username FROM (
                        SELECT 
                            bet.user_id, 
                            SUM(bet.profit/Coin.price) as total
                        FROM bet
                        INNER JOIN Coin ON Coin.id=bet.coin_id
                        GROUP BY bet.user_id) as bet
                INNER JOIN Users ON Users.id=bet.user_id
                ORDER BY total DESC
                LIMIT $1 
                "#,
                    limit
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    /// Partner functions
    pub async fn create_partner(
        &self,
        partner: Partner,
        contacts: &[(String, String)],
    ) -> Result<i64, sqlx::Error> {
        let res = sqlx::query!(
            r#"
            INSERT INTO Partner(
                name,
                country,
                traffic_source,
                users_amount_a_month,
                program,
                is_verified,
                login,
                password
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                FALSE,
                $6,
                $7
            )
            RETURNING id
            "#,
            partner.name,
            partner.country,
            partner.traffic_source,
            partner.users_amount_a_month,
            partner.program as PartnerProgram,
            partner.login,
            partner.password
        )
        .fetch_one(&self.db_pool)
        .await?;

        self.add_partner_contacts(res.id, contacts).await?;

        Ok(res.id)
    }

    pub async fn get_partner(&self, id: i64) -> Result<Partner, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Partner,
            r#"
            SELECT * 
            FROM Partner
            WHERE id=$1
            LIMIT 1
            "#,
            id
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_by_login(&self, login: &str) -> Result<Partner, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Partner,
            r#"
            SELECT * 
            FROM Partner
            WHERE login=$1
            LIMIT 1
            "#,
            login
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn create_withdraw_request(
        &self,
        partner_id: i64,
        withdraw_request: &WithdrawRequest,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO Withdrawal(
                token,
                network,
                amount,
                wallet_address,
                partner_id
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            )
            "#,
            withdraw_request.token,
            withdraw_request.network,
            withdraw_request.amount,
            withdraw_request.wallet_address,
            partner_id
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn get_partner_withdrawal_requests(
        &self,
        partner: i64,
        time_boundaries: TimeBoundaries,
    ) -> Result<Vec<Withdrawal>, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    Withdrawal,
                    r#"
                    SELECT 
                        *
                    FROM Withdrawal
                    WHERE partner_id=$1 AND
                        start_time > now() - interval '1 day'
                    "#,
                    partner
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    Withdrawal,
                    r#"
                    SELECT 
                        *
                    FROM Withdrawal
                    WHERE partner_id=$1 AND
                        start_time > now() - interval '1 week'
                    "#,
                    partner
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    Withdrawal,
                    r#"
                    SELECT 
                        *
                    FROM Withdrawal
                    WHERE partner_id=$1 AND
                        start_time > now() - interval '1 month'
                    "#,
                    partner
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    Withdrawal,
                    r#"
                    SELECT 
                        *
                    FROM Withdrawal
                    WHERE partner_id=$1
                    "#,
                    partner
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    pub async fn login_partner(
        &self,
        login: &str,
        password: &str,
    ) -> Result<Option<Partner>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            Partner,
            r#"
            SELECT * 
            FROM Partner
            WHERE login=$1 AND password=$2
            LIMIT 1
            "#,
            login,
            password
        )
        .fetch_optional(&self.db_pool)
        .await
    }

    //pub async fn submit_question(
    //    &self,
    //    name: &str,
    //    email: &str,
    //    message: &str,
    //) -> Result<(), sqlx::Error> {
    //    sqlx::query!(
    //        r#"
    //        INSERT INTO QuestionRequest(
    //            name,
    //            email,
    //            message
    //        ) VALUES (
    //            $1,
    //            $2,
    //            $3
    //        )
    //        "#,
    //        name,
    //        email,
    //        message
    //    )
    //    .execute(&self.db_pool)
    //    .await
    //    .map(|_| ())
    //}

    pub async fn partner_change_password(
        &self,
        partner_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE Partner
            SET password=$1
            WHERE id=$2 AND password=$3
            "#,
            new_password,
            partner_id,
            old_password
        )
        .execute(&self.db_pool)
        .await
        .map(|r| (r.rows_affected() > 0))
    }

    pub async fn add_partner_contacts(
        &self,
        partner_id: i64,
        contacts: &[(String, String)],
    ) -> Result<(), sqlx::Error> {
        for (name, url) in contacts {
            sqlx::query!(
                r#"
                INSERT INTO PartnerContact(
                    name,
                    url,
                    partner_id
                ) VALUES (
                    $1,
                    $2,
                    $3
                )
                "#,
                name,
                url,
                partner_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    pub async fn delete_partner_contacts(
        &self,
        partner_id: i64,
        contact_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        for contact_id in contact_ids.iter() {
            sqlx::query!(
                r#"
                DELETE FROM partnercontact where id = $1 AND partner_id = $2
                "#,
                contact_id,
                partner_id
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_partner_contacts(
        &self,
        partner_id: i64,
    ) -> Result<Vec<PartnerContact>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            PartnerContact,
            r#"
            SELECT * 
            FROM PartnerContact
            WHERE partner_id=$1
            "#,
            partner_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn add_partner_site(
        &self,
        partner_id: i64,
        url: &str,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO PartnerSite(
                id,
                name,
                url,
                partner_id
            ) 
            SELECT 
                COALESCE(MAX(id)+1,0),
                $1,
                $2,
                $3
            FROM PartnerSite
            WHERE partner_id=$3
            "#,
            name,
            url,
            partner_id
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn get_partner_sites(
        &self,
        partner_id: i64,
    ) -> Result<Vec<PartnerSite>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            PartnerSite,
            r#"
            SELECT *
            FROM PartnerSite
            WHERE partner_id=$1
            "#,
            partner_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    // pub async fn get_partner_site(&self, wallet: &str) -> Result<PartnerSite, sqlx::Error> {
    //     sqlx::query_as_unchecked!(
    //         PartnerSite,
    //         r#"
    //         SELECT *
    //         FROM PartnerSite
    //         WHERE partner_id=$1
    //         LIMIT 1
    //         "#,
    //         wallet
    //     )
    //     .fetch_one(&self.db_pool)
    //     .await
    // }

    pub async fn add_partner_subid(
        &self,
        internal_site_id: i64,
        patner_id: i64,
        url: &str,
        name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO SiteSubId(
                id,
                name,
                url,
                site_id,
                partner_id
            ) 
            SELECT 
                COALESCE(MAX(id)+1,0),
                $1,
                $2,
                $3,
                $4
            FROM SiteSubId
            WHERE site_id=$3
            "#,
            name,
            url,
            internal_site_id,
            patner_id
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn get_site_subids(
        &self,
        internal_site_id: i64,
    ) -> Result<Vec<SiteSubId>, sqlx::Error> {
        sqlx::query_as_unchecked!(
            SiteSubId,
            r#"
            SELECT *
            FROM SiteSubId
            WHERE site_id=$1
            "#,
            internal_site_id
        )
        .fetch_all(&self.db_pool)
        .await
    }

    pub async fn get_subid(
        &self,
        partner_id: i64,
        site_id: i64,
        sub_id: i64,
    ) -> Result<SiteSubId, sqlx::Error> {
        sqlx::query_as_unchecked!(
            SiteSubId,
            r#"
            SELECT 
                sitesubid.internal_id,
                sitesubid.id,
                sitesubid.name,
                sitesubid.url,
                sitesubid.site_id,
                sitesubid.partner_id
            FROM partnersite 
            INNER JOIN sitesubid ON site_id=partnersite.internal_id AND partnersite.partner_id=sitesubid.partner_id
            WHERE partnersite.partner_id=$1 AND partnersite.id=$2 AND sitesubid.id=$3
            "#,
            partner_id,
            site_id,
            sub_id
        ).fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_subid_clicks(
        &self,
        partner_id: i64,
        site_id: i64,
        sub_id: i64,
    ) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
            SELECT 
                COUNT(refclick.id) as clicks
            FROM refclick
            INNER JOIN (SELECT 
                sitesubid.internal_id
            FROM partnersite 
            INNER JOIN sitesubid ON site_id=partnersite.internal_id AND partnersite.partner_id=sitesubid.partner_id
            WHERE partnersite.partner_id=$1 
                        AND partnersite.id=$2 
                        AND sitesubid.id=$3) AS subids ON subids.internal_id=refclick.sub_id_internal;
            "#,
            partner_id,
            site_id,
            sub_id
        ).fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_site_clicks(
        &self,
        partner_id: i64,
        site_id: i64,
    ) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
            SELECT 
                COUNT(clicks.timestamp) as clicks
            FROM partnersite
            INNER JOIN (SELECT * FROM refclick
                    INNER JOIN sitesubid ON sitesubid.internal_id=refclick.sub_id_internal
                    WHERE refclick.partner_id=$1) as clicks
            ON partnersite.internal_id=clicks.site_id
            WHERE partnersite.id = $2;
            "#,
            partner_id,
            site_id
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_clicks(&self, partner_id: i64) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
            SELECT 
                COUNT(refclick.id) as clicks
            FROM refclick
            WHERE partner_id=$1
            "#,
            partner_id
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_connected_wallets_amount_exact_date(
        &self,
        partner_id: i64,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<AmountConnectedWallets, sqlx::Error> {
        sqlx::query_as_unchecked!(
            AmountConnectedWallets,
            r#"
                SELECT CAST(COUNT(connectedusers.user_id) as BIGINT) as connected_users 
                FROM connectedusers 
                WHERE partner_id=$1 AND
                    connectedusers.timestamp >= $2 AND
                    connectedusers.timestamp <= $3
            "#,
            partner_id,
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_connected_wallets_with_bets_amount_exact_date(
        &self,
        partner_id: i64,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<AmountConnectedWallets, sqlx::Error> {
        sqlx::query_as_unchecked!(
            AmountConnectedWallets,
            r#"
                SELECT CAST(COUNT(DISTINCT connectedusers.user_id) as BIGINT) as connected_users 
                FROM connectedusers
                INNER JOIN bet ON connectedusers.user_id=bet.user_id
                WHERE partner_id=$1 AND
                    connectedusers.timestamp >= $2 AND
                    connectedusers.timestamp <= $3
            "#,
            partner_id,
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_clicks_exact_date(
        &self,
        partner_id: i64,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<RefClicks, sqlx::Error> {
        sqlx::query_as_unchecked!(
            RefClicks,
            r#"
                SELECT CAST(COUNT(refclick.id) as BIGINT) as clicks 
                FROM refclick 
                WHERE partner_id=$1 AND
                    refclick.timestamp >= $2 AND
                    refclick.timestamp <= $3
            "#,
            partner_id,
            start,
            end
        )
        .fetch_one(&self.db_pool)
        .await
    }

    pub async fn get_partner_connected_wallets_info(
        &self,
        partner_id: i64,
        time_boundaries: TimeBoundaries,
    ) -> Result<Vec<ConnectedWallet>, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    ConnectedWallet,
                    r#"
                    SELECT 
                        connectedusers.id,
                        connectedusers.user_id,
                        connectedusers.timestamp,
                        partnersite.id as site_id,
                        sitesubid.id as sub_id
                    FROM connectedusers
                    INNER JOIN sitesubid ON sitesubid.internal_id=connectedusers.sub_id_internal
                    INNER JOIN partnersite ON sitesubid.site_id=partnersite.internal_id
                    WHERE connectedusers.partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 day'
                    "#,
                    partner_id
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    ConnectedWallet,
                    r#"
                    SELECT 
                        connectedusers.id,
                        connectedusers.user_id,
                        connectedusers.timestamp,
                        partnersite.id as site_id,
                        sitesubid.id as sub_id
                    FROM connectedusers
                    INNER JOIN sitesubid ON sitesubid.internal_id=connectedusers.sub_id_internal
                    INNER JOIN partnersite ON sitesubid.site_id=partnersite.internal_id
                    WHERE connectedusers.partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 week'
                    "#,
                    partner_id
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    ConnectedWallet,
                    r#"
                    SELECT 
                        connectedusers.id,
                        connectedusers.user_id,
                        connectedusers.timestamp,
                        partnersite.id as site_id,
                        sitesubid.id as sub_id
                    FROM connectedusers
                    INNER JOIN sitesubid ON sitesubid.internal_id=connectedusers.sub_id_internal
                    INNER JOIN partnersite ON sitesubid.site_id=partnersite.internal_id
                    WHERE connectedusers.partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 month'
                    "#,
                    partner_id
                )
                .fetch_all(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    ConnectedWallet,
                    r#"
                    SELECT 
                        connectedusers.id,
                        connectedusers.user_id,
                        connectedusers.timestamp,
                        partnersite.id as site_id,
                        sitesubid.id as sub_id
                    FROM connectedusers
                    INNER JOIN sitesubid ON sitesubid.internal_id=connectedusers.sub_id_internal
                    INNER JOIN partnersite ON sitesubid.site_id=partnersite.internal_id
                    WHERE connectedusers.partner_id=$1
                    "#,
                    partner_id
                )
                .fetch_all(&self.db_pool)
                .await
            }
        }
    }

    // pub async fn partner_exists(&self, partner: &str) -> Result<bool, sqlx::Error> {
    //     sqlx::query_as_unchecked!(
    //         Partner,
    //         r#"
    //         SELECT * FROM partner where main_wallet=$1
    //         "#,
    //         partner
    //     )
    //     .fetch_optional(&self.db_pool)
    //     .await
    //     .map(|part| part.is_some())
    // }

    pub async fn get_partner_connected_wallets_with_deposits_amount(
        &self,
        partner_id: i64,
        time_boundaries: TimeBoundaries,
    ) -> Result<AmountConnectedWallets, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                    SELECT CAST(COUNT(DISTINCT connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers
                        INNER JOIN bet ON bet.user_id = connectedusers.user_id
                    WHERE partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 day'
                    "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                    SELECT CAST(COUNT(DISTINCT connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers
                        INNER JOIN bet ON bet.user_id = connectedusers.user_id
                    WHERE partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 week'
                "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                    SELECT CAST(COUNT(DISTINCT connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers
                        INNER JOIN bet ON bet.user_id = connectedusers.user_id
                    WHERE partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 month'
                    "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                    SELECT CAST(COUNT(DISTINCT connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers
                        INNER JOIN bet ON bet.user_id = connectedusers.user_id
                    WHERE partner_id=$1
                    "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
        }
    }

    pub async fn get_partner_connected_wallets_amount(
        &self,
        partner_id: i64,
        time_boundaries: TimeBoundaries,
    ) -> Result<AmountConnectedWallets, sqlx::Error> {
        match time_boundaries {
            TimeBoundaries::Daily => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                        SELECT CAST(COUNT(connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers 
                        WHERE partner_id=$1 AND
                            connectedusers.timestamp > now() - interval '1 day'
                    "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
            TimeBoundaries::Weekly => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                        SELECT CAST(COUNT(connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers 
                        WHERE partner_id=$1 AND
                    connectedusers.timestamp > now() - interval '1 week'
            "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
            TimeBoundaries::Monthly => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                        SELECT CAST(COUNT(connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers 
                        WHERE partner_id=$1 AND
                           connectedusers.timestamp > now() - interval '1 month'
                    "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
            TimeBoundaries::All => {
                sqlx::query_as_unchecked!(
                    AmountConnectedWallets,
                    r#"
                        SELECT CAST(COUNT(connectedusers.user_id) as BIGINT) as connected_users 
                        FROM connectedusers 
                        WHERE partner_id=$1
                    "#,
                    partner_id
                )
                .fetch_one(&self.db_pool)
                .await
            }
        }
    }

    pub async fn add_click(&self, partner_id: i64, sub_id: i64) -> Result<(), sqlx::Error> {
        // sqlx::query!(
        //     r#"
        //     INSERT INTO refclicks(
        //         clicks,
        //         sub_id_internal,
        //         partner_id
        //     )
        //      VALUES (
        //          1,
        //          $1,
        //          $2
        //      )
        //      ON CONFLICT(sub_id_internal,partner_id) DO UPDATE
        //      SET clicks = refclicks.clicks+1;
        //     "#,
        //     sub_id,
        //     partner
        // )
        // .execute(&self.db_pool)
        // .await
        // .map(|_| ())
        sqlx::query!(
            r#"
            INSERT INTO refclick(
                timestamp,
                sub_id_internal,
                partner_id
            )
             VALUES (
                 NOW(), 
                 $1,
                 $2
             )
            "#,
            sub_id,
            partner_id
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }

    pub async fn add_ref_wallet(
        &self,
        user_id: i64,
        timestamp: DateTime<Utc>,
        sub_id_internal: i64,
        partner_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO ConnectedUsers(
                user_id,
                timestamp,
                sub_id_internal,
                partner_id
            ) VALUES(
                $1,
                $2,
                $3,
                $4
            )
            "#,
            user_id,
            timestamp.naive_utc(),
            sub_id_internal,
            partner_id
        )
        .execute(&self.db_pool)
        .await
        .map(|_| ())
    }
}
