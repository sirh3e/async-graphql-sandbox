use std::io::Write;
use async_graphql::{Schema, SimpleObject, EmptyMutation, EmptySubscription, Object, ID};
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use rocket::response::content;
use rocket::{routes, State};

#[derive(SimpleObject)]
pub struct Market {
    id: ID,
    
    #[graphql(external)]
    name: String,

    #[graphql(external)]
    version: u64,
}

#[derive(SimpleObject)]
pub struct MarketHashName {
    #[graphql(shareable)]
    value: String,
    #[graphql(external)]
    markets: Vec<Market>,
    
    #[graphql(external)]
    version: u64
}

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn inventory(&self) -> MarketHashName {
        MarketHashName {
            value: "AK-47 | Redline (Field-Tested)".to_string(),
            markets: vec![],
            version: 6
        }
    }
    
    #[graphql(entity)]
    async fn find_market_by_name_id(&self, id: ID) -> Market {
        Market { id, name: "".to_string(), version: 1 }
    }
    
    #[graphql(entity)]
    async fn find_market_hash_name_by_value(&self, value: String) -> MarketHashName {
        MarketHashName { value, markets: vec![], version: 1 }
    }
}

type InventorySchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[rocket::get("/graphql?<query..>")]
async fn graphql_query(schema: &State<InventorySchema>, query: GraphQLQuery) -> GraphQLResponse {
    query.execute(schema.inner()).await
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
async fn graphql_mutation(schema: &State<InventorySchema>, request: GraphQLRequest) -> GraphQLResponse {
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
        .enable_federation() // Enable federation to allow type extensions
        .finish();

    let schema_sdl = schema.sdl();
    let mut file = std::fs::File::create("schema_inventory.graphql")?;
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
