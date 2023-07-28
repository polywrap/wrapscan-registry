use axum::extract::{Path, State};
use http::StatusCode;

use crate::{functions, models::Package, Repository};

use super::Dependencies;

pub async fn package_info<T>(
    Path((user, package)): Path<(String, String)>,
    State(deps): State<Dependencies<T>>,
) -> Result<String, StatusCode>
where
    T: Repository<Package>,
{
    let Dependencies { package_repo } = deps;

    let info = functions::package_info(user, package, &package_repo).await?;

    Ok(info)
}
