use std::convert::Infallible;

use async_graphql::*;
use warp::{http::Response, Filter};

pub async fn run() {
    let schema = get_schema();

    let graphql_server = || {};

    let graphql_playground = || {
        Response::builder()
            .header("content-type", "text/html")
            .body(http::playground_source(http::GraphQLPlaygroundConfig::new(
                "/",
            )))
    };

    let routes = warp::path("graphql")
        .and(
            warp::post().and(async_graphql_warp::graphql(schema).and_then(
                |(schema, request): (MySchema, Request)| async move {
                    // Execute query
                    let resp = schema.execute(request).await;

                    // Return result
                    Ok::<_, Infallible>(async_graphql_warp::Response::from(resp))
                },
            )),
        )
        .or(warp::get().map(graphql_playground));

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

//
// Our schema stuff
//

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;

#[Object]
impl Query {
    async fn example(&self) -> i32 {
        3
    }
}

fn get_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::new(Query, EmptyMutation, EmptySubscription)
}
