use std::convert::Infallible;

use async_graphql::{guard::Guard, *};
use warp::{http::Response, Filter};

pub async fn run() {
    let schema = get_schema();

    let graphql_playground = || {
        Response::builder()
            .header("content-type", "text/html")
            .body(http::playground_source(http::GraphQLPlaygroundConfig::new(
                "/",
            )))
    };

    let routes = warp::path("graphql")
        .and(
            warp::post()
                .and(warp::header::optional::<String>("x-session-id"))
                .and(async_graphql_warp::graphql(schema))
                .and_then(
                    |session_id, (schema, mut request): (MySchema, Request)| async move {
                        request.data.insert(ClientData { session_id });

                        // Execute query
                        let resp = schema.execute(request).await;

                        // Return result
                        Ok::<_, Infallible>(async_graphql_warp::Response::from(resp))
                    },
                ),
        )
        .or(warp::get().map(graphql_playground));

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

//
// Our schema stuff
//

struct ClientData {
    session_id: Option<String>,
}

/// Guard that makes sure the user has a session
struct SessionGuard;

#[async_trait::async_trait]
impl Guard for SessionGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let client_data: &ClientData = ctx.data().expect("Missing client data!");

        if let Some(id) = &client_data.session_id {
            if matches!(id.parse(), Ok(7)) {
                return Ok(());
            }
        }

        Err("You're totally forbidden, dude. Try setting `x-session-id` header to `7`".into())
    }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

struct Query;

#[Object]
impl Query {
    /// Just get the number `3`
    async fn example(&self) -> i32 {
        3
    }

    /// You **must** have the `x-session-id` header set to 7 to be able to run this query
    #[graphql(guard(SessionGuard()))]
    async fn protected_query(&self) -> String {
        "You have gained access into the protected query".into()
    }
}

fn get_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::new(Query, EmptyMutation, EmptySubscription)
}
