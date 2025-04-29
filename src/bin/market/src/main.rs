use std::io::Write;
use async_graphql::{ComplexObject, Context, EmptyMutation, EmptySubscription, InputObject, Object, Schema, SimpleObject, ID};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use rocket::response::content;
use rocket::{routes, State};

#[derive(SimpleObject)]
#[graphql(complex)]
struct Market {
    id: ID,
    name: String,
}

#[ComplexObject]
impl Market {
    async fn version(&self) -> u64 {
        1
    }
}

/*
#[derive(SimpleObject)]
#[graphql(complex)]
struct MarkerHashName {
    #[graphql(shareable)]
    value: String,
}

#[ComplexObject]
impl MarkerHashName {
    async fn markets(&self, ctx: &Context<'_>) -> Vec<String> {
        vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
        ]
    }
}
*/

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct MarketHashName {
    #[graphql(shareable)]
    value: String,
}

#[ComplexObject]
impl MarketHashName {
    async fn markets(&self) -> Vec<Market> {
        vec![
            Market {
                id: ID::from("id".to_string()),
                name: "1".to_string()
            }
        ]
    }
    
    async fn version(&self) -> u64 {
        7
    }
}

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn ping(&self, _ctx: &Context<'_>) -> &str {
        "pong"
    }
    
    async fn markets(&self) -> Vec<Market> {
        vec![
            Market {id: ID("A".to_string()), name: "name a".to_string()},
            Market {id: ID("B".to_string()), name: "name b".to_string()},
        ]
    }

    #[graphql(entity)]
    async fn find_market_hash_name_by_value(&self, value: String) -> MarketHashName {
        MarketHashName {
            value,
        }
    }
    
    #[graphql(entity)]
    async fn find_market_by_name_id(&self, id: ID) -> Market {
        Market { id, name: "".to_string() }
    }
}

type MarketSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[rocket::get("/graphql?<query..>")]
async fn graphql_query(schema: &State<MarketSchema>, query: GraphQLQuery) -> GraphQLResponse {
    query.execute(schema.inner()).await
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_mutation(schema: &State<MarketSchema>, request: GraphQLRequest) -> GraphQLResponse {
    request.execute(schema.inner()).await
}

#[rocket::get("/")]
async fn graphql_playground() -> content::RawHtml<String> {
    content::RawHtml(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .enable_federation() // Enable federation to link with other services
        .finish();

    let schema_sdl = schema.sdl();
    let mut file = std::fs::File::create("schema_market.graphql")?;
    file.write_all(schema_sdl.as_bytes())?;

    rocket::build()
        .manage(schema)
        .mount("/", routes![graphql_query, graphql_mutation, graphql_playground])
        .ignite()
        .await?
        .launch()
        .await?;
    Ok(())
}
