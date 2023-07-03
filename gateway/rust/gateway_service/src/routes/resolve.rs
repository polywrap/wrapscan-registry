use axum::{
    body::BoxBody,
    extract::{Path, State},
    response::Response,
};
use http::StatusCode;

use crate::{constants, debug_println, functions, models::Package, Repository};

use super::Dependencies;

pub async fn resolve<T>(
    Path((user, package_and_version, file_path)): Path<(String, String, String)>,
    State(deps): State<Dependencies<T>>,
) -> Result<Response, StatusCode>
where
    T: Repository<Package>,
{
    let Dependencies { package_repo } = deps;

    let uri = functions::resolve(user, package_and_version, file_path, package_repo).await?;

    let response: Response = Response::builder()
        .status(StatusCode::OK)
        .header(constants::WRAP_URI_HEADER, uri.to_string())
        .body(BoxBody::default())
        .map_err(|e| {
            debug_println!("Error publishing package: {}", &e);
            eprintln!("INTERNAL_SERVER_ERROR constructing response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}
