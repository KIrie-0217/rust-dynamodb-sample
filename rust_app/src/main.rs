use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error as OtherError};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    account_id: String,
    account_name: String,
    password: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(
    dynamodb_client: &Client,
    event: Request,
) -> Result<Response<Body>, Error> {
    let body = event.body();
    let body_str = std::str::from_utf8(body).expect("invalid body");

    info!(payload = %body_str,"Json Payload received");

    let account = match serde_json::from_str::<Account>(body_str) {
        Ok(account) => account,
        Err(err) => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;
            return Ok(resp);
        }
    };

    // Insert into the table.
    add_account(dynamodb_client, account.clone(), "rust-lambda-example").await?;

    // Desirialize into json to return in the Responese
    let body_out = serde_json::to_string(&account)?;

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(body_out.into())
        .map_err(Box::new)?;
    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    info!("Getting DynamoDB client");
    let config = ::aws_config::load_from_env().await;
    let client = Client::new(&config);

    run(service_fn(|event: Request| async {
        function_handler(&client, event).await
    }))
    .await
}

// Add an item into a table.
pub async fn add_account(
    dynamodb_client: &Client,
    account: Account,
    table: &str,
) -> Result<(), OtherError> {
    // Set Attribute
    let account_id = AttributeValue::N(account.account_id);
    let account_name = AttributeValue::S(account.account_name);
    let password = AttributeValue::S(account.password);

    // Create request
    let request = dynamodb_client
        .put_item()
        .table_name(table)
        .item("account_id", account_id)
        .item("account_name", account_name)
        .item("password", password);

    info!("adding item to DynamoDB");

    // send
    let _resp = request.send().await?;

    Ok(())
}
