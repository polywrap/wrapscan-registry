mod home;
pub use home::*;

mod publish;
pub use publish::*;

mod resolve;
pub use resolve::*;

use crate::{models::Package, Repository};

#[derive(Clone)]
pub struct Dependencies<T>
where
    T: Repository<Package>,
{
    pub package_repo: T,
}
