use std::any::{type_name, TypeId};
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Extension, extract, Router, routing::{get, post}};
use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use axum::body::Body;
use axum::handler::Handler;
use axum::http::Request;
use axum::response::Html;
use futures::future;
use juniper::{EmptyMutation, EmptySubscription, FieldError, FieldResult, graphql_object, graphql_subscription, GraphQLType, RootNode};
use juniper::http::{GraphQLBatchRequest, GraphQLBatchResponse, GraphQLRequest, GraphQLResponse};
use juniper::http::graphiql::graphiql_source;

#[derive(Clone, Copy, Debug)]
pub struct Context;

impl juniper::Context for Context {}

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
    let tid = TypeId::of::<i32>();

    // build our GraphQL schema
    // TODO: add mutations and subscriptions
    let schema = Arc::new(AppSchema::new(Query, EmptyMutation::new(), EmptySubscription::new()));
    let context = Arc::new(Context {});

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

async fn graphql(extract::Json(request): extract::Json<GraphQLBatchRequest>) {}

async fn juniper_subscriptions() {}

