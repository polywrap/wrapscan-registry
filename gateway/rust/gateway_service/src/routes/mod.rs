mod home;
pub use home::*;

mod publish;
pub use publish::*;

mod resolve;
pub use resolve::*;

mod latest_version_info;
pub use latest_version_info::*;

mod package_info;
pub use package_info::*;

use crate::{models::Package, Repository};

#[derive(Clone)]
pub struct Dependencies<T>
where
    T: Repository<Package>,
{
    pub package_repo: T,
}
