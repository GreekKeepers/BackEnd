use std::{io, sync::Arc};

use crate::api_documentation::{serve_swagger, ApiDoc};
use crate::communication::*;
use crate::game_engine::{Engine, StatefulGameEngine};
//use api_documentation::{serve_swagger, ApiDoc};
use config::DatabaseSettings;
use db::DB;
use futures::future::join_all;
use rejection_handler::handle_rejection;
use std::env;
use thedex::TheDex;
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::hyper::header::HeaderName;
use warp::Filter;

mod api_documentation;
mod communication;
mod config;
mod db;
mod errors;
mod filters;
mod game_engine;
mod games;
mod handlers;
mod jwt;
mod models;
mod oauth_providers;
mod rejection_handler;
mod tools;

#[tokio::main]
async fn main() {
    //load .env file
    dotenvy::dotenv()
        .map_err(|e| {
            error!(error = e.to_string(), "Error loading .env");
            e
        })
        .unwrap();

    // load log config
    let env_filter = EnvFilter::from_default_env()
        .add_directive("backend=debug".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap());
    let collector = tracing_subscriber::registry().with(env_filter).with(
        fmt::Layer::new()
            .with_writer(io::stdout)
            .with_thread_names(true),
    );
    let file_appender = tracing_appender::rolling::minutely("logs", "backend_log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let collector = collector.with(
        fmt::Layer::new()
            .with_writer(non_blocking)
            .with_thread_names(true),
    );
    tracing::subscriber::set_global_default(collector).unwrap();

    info!("Starting rest api");

    let db_settings = DatabaseSettings {
        username: env::var("DB_USERNAME").unwrap(),
        password: env::var("DB_PASSWORD").unwrap(),
        host: env::var("DB_HOST").unwrap(),
        port: env::var("DB_PORT").unwrap().parse().unwrap(),
        database_name: env::var("DB_NAME").unwrap(),
    };

    debug!("Connecting to DB with settings {:?}", db_settings);

    let db = DB::new(&db_settings).await;

    info!(
        "The rest api is starting on the {:?}:{:?}",
        *config::SERVER_HOST,
        *config::SERVER_PORT
    );

    // let (bet_sender, bet_receiver) = channel(10000);
    // let (ws_data_feed, _bet_receiver) = channel(10000);

    // info!("Staring networks handlers");
    // network_handler::start_network_handlers(db.clone(), bet_sender.clone()).await;
    // tokio::spawn(network_handler::bet_listener(
    //     db.clone(),
    //     bet_receiver,
    //     ws_data_feed.clone(),
    // ));

    //api UI
    let api_config = Arc::new(Config::from("/api/api-doc.json"));
    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&ApiDoc::openapi()));
    let swagger_ui = warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || api_config.clone()))
        .and_then(serve_swagger);

    let cors = warp::cors()
        // .allow_origin("http://localhost:3000/")
        .allow_any_origin()
        .allow_methods(vec!["GET", "OPTIONS", "POST"])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("access-control-allow-origin"),
            HeaderName::from_static("accept"),
        ]);

    let dex = TheDex::new(config::X_EX_APIKEY.clone(), config::X_EX_SECRETKEY.clone()).await;
    let p2way = p2way::P2Way::new(
        config::P2WAY_APIKEY.clone(),
        config::P2WAY_SECRETKEY.clone(),
        p2way::SANDBOX_URL,
    );
    let hcap = hcaptcha::HCaptcha::new(config::HCAPTCHA_SECRET.clone());

    let (ws_manager_tx, ws_manager_rx) = unbounded_channel::<WsManagerEvent>();
    let ws_manager = Manager::new(ws_manager_rx, &db).await;

    let (engine_tx, engine_rx) = async_channel::unbounded();

    let (stateful_engine_tx, stateful_engine_rx) = unbounded_channel::<EnginePropagatedBet>();

    info!("Starting `{}` engines", *config::ENGINES);
    let mut engines: Vec<_> = Vec::with_capacity(*config::ENGINES as usize);
    for _ in 0..*config::ENGINES {
        engines.push(
            Engine::new(
                db.clone(),
                ws_manager_tx.clone(),
                engine_rx.clone(),
                stateful_engine_tx.clone(),
            )
            .await
            .run(),
        );
    }
    let engines_handle = join_all(engines);

    let statefull_engine =
        StatefulGameEngine::new(db.clone(), ws_manager_tx.clone(), stateful_engine_rx)
            .await
            .run();

    info!("Server started, waiting for CTRL+C");
    tokio::select! {
        r = ws_manager.run() => {
            warn!("WS Manager stopped: `{:?}`", r);
        }
        _ = warp::serve(
            filters::init_filters(db, dex, p2way, ws_manager_tx, engine_tx, hcap).or(api_doc)
            .or(swagger_ui).recover(handle_rejection).with(cors),
        )
        .run((*config::SERVER_HOST, *config::SERVER_PORT)) => {},
        _ = signal::ctrl_c() => {
            warn!("CTRL+C received, stopping process...")
        }
        _ = engines_handle => {
            warn!("Engine stopped");
        }
        _ = statefull_engine => {
            warn!("Statefull engine stopped");
        }
    }
}
