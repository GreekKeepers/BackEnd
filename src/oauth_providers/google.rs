use crate::errors::ApiError;
use reqwest;

use crate::config;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeResponse {
    pub code: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OauthTokenResponse {
    pub access_token: String,
    #[serde(with = "ts_seconds")]
    pub expires_in: DateTime<Utc>,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String,
    pub id_token: String,
}

#[derive(Deserialize, Debug)]
pub struct GoogleUserResult {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

pub struct GoogleOauth {
    client: Arc<reqwest::Client>,
}

impl GoogleOauth {
    pub fn new() -> Self {
        GoogleOauth {
            client: Arc::new(reqwest::Client::new()),
        }
    }
    pub async fn request_token(
        &self,
        authorization_code: &str,
    ) -> Result<OauthTokenResponse, ApiError> {
        let redirect_url = "https://game.greekkeepers.io/api/oauth/google";
        let client_secret: String = config::GOOGLE_SECRET_KEY.clone();
        let client_id: String = config::GOOGLE_CLIENT_ID.clone();

        let root_url = "https://oauth2.googleapis.com/token";

        let params = [
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_url),
            ("client_id", &client_id),
            ("code", authorization_code),
            ("client_secret", &client_secret),
        ];
        let response = self
            .client
            .post(root_url)
            .form(&params)
            .send()
            .await
            .map_err(ApiError::ReqwestError)?;

        if !response.status().is_success() {
            return Err(ApiError::GoogleApiError(
                response.text().await.map_err(ApiError::ReqwestError)?,
            ));
        }
        Ok(response
            .json::<OauthTokenResponse>()
            .await
            .map_err(ApiError::ReqwestError)?)
    }

    pub async fn get_google_user(
        &self,
        access_token: &str,
        id_token: &str,
    ) -> Result<GoogleUserResult, ApiError> {
        let mut url = reqwest::Url::parse("https://www.googleapis.com/oauth2/v1/userinfo").unwrap();
        url.query_pairs_mut().append_pair("alt", "json");
        url.query_pairs_mut()
            .append_pair("access_token", access_token);

        let response = self
            .client
            .get(url)
            .bearer_auth(id_token)
            .send()
            .await
            .map_err(ApiError::ReqwestError)?;

        if !response.status().is_success() {
            return Err(ApiError::GoogleApiError(
                response.text().await.map_err(ApiError::ReqwestError)?,
            ));
        }
        Ok(response
            .json::<GoogleUserResult>()
            .await
            .map_err(ApiError::ReqwestError)?)
    }
}
