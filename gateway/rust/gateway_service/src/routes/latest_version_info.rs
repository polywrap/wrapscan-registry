use axum::extract::{Path, State};
use http::StatusCode;

use crate::{functions, models::Package, Repository};

use super::Dependencies;

pub async fn latest_version_info<T>(
    Path((user, package_and_version)): Path<(String, String)>,
    State(deps): State<Dependencies<T>>,
) -> Result<String, StatusCode>
where
    T: Repository<Package>,
{
    let Dependencies { package_repo } = deps;

    let info = functions::latest_version_info(user, package_and_version, &package_repo).await?;

    Ok(info)
}
