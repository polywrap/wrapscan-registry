use lambda_http::Error as HttpError;

#[tokio::main]
async fn main() -> Result<(), HttpError> {
    gateway_service::setup_routes().await
}
