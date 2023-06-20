# Rust GraphQL with Juniper Lab
Creating a GraphQL application in Rust with Juniper.

## Debugging Axum Handlers

The error messages are terrible when the handler signatures are not correct.

    Unfortunately Rust gives poor error messages if you try to use a function that doesn’t quite match what’s required by Handler.

https://docs.rs/axum/latest/axum/handler/index.html#debugging-handler-type-errors


Example error

```
error[E0277]: the trait bound `fn(Request<Body>, Extension<Arc<Context>>, Extension<Arc<RootNode<'static, Query, EmptyMutation, EmptySubscription, _>>>) -> impl futures::Future<Output = impl IntoResponse> {graphql::<_>}: Handler<_, _, _>` is not satisfied
   --> src\main.rs:80:33
    |
80  |         .route("/graphql", post(graphql))
    |                            ---- ^^^^^^^ the trait `Handler<_, _, _>` is not implemented for fn item `fn(Request<Body>, Extension<Arc<Context>>, Extension<Arc<RootNode<'static, Query, EmptyMutation, EmptySubscription, _>>>) -> impl futures::Future<Output = impl IntoResponse> {graphql::<_>}`
    |                            |
    |                            required by a bound introduced by this call
    |
    = help: the following other types implement trait `Handler<T, S, B>`:
              <Layered<L, H, T, S, B, B2> as Handler<T, S, B2>>
              <MethodRouter<S, B> as Handler<(), S, B>>
note: required by a bound in `post`
   --> C:\Users\marti\.cargo\registry\src\index.crates.io-6f17d22bba15001f\axum-0.6.18\src\routing\method_routing.rs:407:1
    |
407 | top_level_handler_fn!(post, POST);
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `post`
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)
```

Use `axum-macros` crate and its `debug_handler` macro to get better error messages.

Then, with the macro, you get something like this:

```
error: `Json<_>` consumes the request body and thus must be the last argument to the handler function
   --> src\main.rs:150:8
    |
150 |     r: Json<GraphQLBatchRequest>,
    |        ^^^^

```

Much better!