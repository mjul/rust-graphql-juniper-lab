use std::collections::HashMap;
use std::sync::Arc;

use axum::{Extension, Router, routing::{get, post}};
use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use axum::response::Html;
use axum_macros::debug_handler;
use futures::future;
use juniper::{EmptyMutation, EmptySubscription, FieldError, FieldResult,
              graphql_object, graphql_subscription, graphql_value, GraphQLEnum,
              RootNode};
use juniper::http::{GraphQLBatchRequest, GraphQLBatchResponse};
use juniper::http::graphiql::graphiql_source;

#[derive(Clone, Debug)]
struct Player {
    id: String,
    name: String,
    instrument: Instrument,
}

impl Player {
    fn new(id: String, name: String, instrument: Instrument) -> Self {
        Self { id, name: name, instrument }
    }
}

#[graphql_object(context = Context)]
impl Player {
    /// Get the id of the player
    fn id(&self) -> &str {
        &self.id
    }
    /// Get the name of the player
    fn name(&self) -> &str {
        &self.name
    }
    /// Get the instrument of the player
    fn instrument(&self) -> Instrument {
        self.instrument
    }
}

#[derive(GraphQLEnum, Clone, Copy, Debug, Eq, PartialEq)]
enum Instrument {
    Guitar,
    Piano,
}


#[derive(Clone, Debug)]
pub struct Context {
    players: HashMap<String, Player>,
}


impl juniper::Context for Context {}

impl Context {
    pub fn new() -> Self {
        let mut players = HashMap::new();
        vec![
            Player::new(String::from("1000"), String::from("Steve"), Instrument::Guitar),
            Player::new(String::from("1001"), String::from("Stevie"), Instrument::Guitar),
            Player::new(String::from("1002"), String::from("Jimmy"), Instrument::Guitar),
            Player::new(String::from("1003"), String::from("Eric"), Instrument::Guitar),
            Player::new(String::from("1004"), String::from("Jimi"), Instrument::Guitar),
            Player::new(String::from("1005"), String::from("Chuck"), Instrument::Guitar),
            Player::new(String::from("1006"), String::from("Eddie"), Instrument::Guitar),
            Player::new(String::from("2000"), String::from("Jerry"), Instrument::Piano),
            Player::new(String::from("2001"), String::from("Ray"), Instrument::Piano),
            Player::new(String::from("2002"), String::from("Billy"), Instrument::Piano),
            Player::new(String::from("2003"), String::from("Elton"), Instrument::Piano),
        ].into_iter().for_each(
            |p|
                { players.insert(p.id.clone(), p); });
        Self { players }
    }
    /// Get a player by id
    fn get_player(&self, id: &str) -> Option<&Player> {
        self.players.get(id)
    }
    /// Get all players
    fn get_players(&self) -> Vec<&Player> {
        self.players.values().collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    /// Add two numbers a and b
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }
    /// Get the hello message
    fn hello(&self) -> FieldResult<&str> {
        Ok("Hello, World!")
    }
    /// Get the players
    fn players(
        #[graphql(context)] context: &Context) -> Vec<&Player> {
        context.get_players()
    }
    /// Get a player by id
    fn player(
        #[graphql(context)] context: &Context,
        #[graphql(description = "id of the player")] id: String,
    ) -> Option<&Player> {
        context.get_player(&id)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Subscription;


/// A wrapper around [`GraphQLRequest`]
#[derive(Debug, PartialEq)]
pub struct JuniperRequest(pub GraphQLBatchRequest);

/// A wrapper around [`GraphQLBatchResponse`] that implements [`IntoResponse`]
/// so it can be returned from axum handlers.
pub struct JuniperResponse<'a>(pub GraphQLBatchResponse<'a>);

impl<'a> IntoResponse for JuniperResponse<'a> {
    fn into_response(self) -> Response {
        if !self.0.is_ok() {
            return (StatusCode::BAD_REQUEST, Json(self.0)).into_response();
        }
        Json(self.0).into_response()
    }
}

type AppSchema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;


#[tokio::main]
async fn main() {
    // build our GraphQL schema
    // TODO: add mutations and subscriptions
    let schema = Arc::new(AppSchema::new(Query, EmptyMutation::new(), EmptySubscription::new()));
    let context = Arc::new(Context::new());

    // build our application
    let app = Router::new()
        //.route("/", get(|| async { "Hello, World!" }))
        .route("/", get(playground("/graphql", "/subscriptions")))
        .route("/graphql", post(graphql))
        .route("/subscriptions", get(juniper_subscriptions))
        .layer(Extension(schema))
        .layer(Extension(context));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}


// The following function is from https://github.com/graphql-rust/juniper/pull/1088/files#
// It is a workaround for the fact that juniper::http::playground::playground_source() is not async
// and therefore cannot be used in an axum::routing::get() handler.
pub fn playground<'a>(
    graphql_endpoint_url: &str,
    subscriptions_endpoint_url: impl Into<Option<&'a str>>,
) -> impl FnOnce() -> future::Ready<Html<String>> + Clone + Send {
    let html = Html(juniper::http::playground::playground_source(
        graphql_endpoint_url,
        subscriptions_endpoint_url.into(),
    ));

    || future::ready(html)
}

async fn juniper_subscriptions() {}

#[debug_handler]
async fn graphql(
    Extension(context): Extension<Arc<Context>>,
    Extension(schema): Extension<Arc<AppSchema>>,
    Json(request): Json<GraphQLBatchRequest>,
) -> impl IntoResponse
{
    let response = request.execute(&schema, &context).await;
    // The `.into_response()` makes the borrows that go into response go out of scope, so that the response can be returned.
    // Maybe there is a better way to do this.
    JuniperResponse(response).into_response()
}