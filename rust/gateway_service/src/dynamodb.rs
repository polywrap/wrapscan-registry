use async_trait::async_trait;

use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::get_item::{GetItemOutput, GetItemError};
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;

use crate::{Package, Repository, RepositoryError};

pub struct PackageRepository {
    client: Client,
    table_name: String,
}

impl PackageRepository {
    pub fn new(client: Client, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
}

const KEY_NAME: &'static str = "id";
#[async_trait]
impl Repository<Package> for PackageRepository {

    async fn read(&self, key: &str) -> Result<Package, RepositoryError> {
        let response = self.client
            .get_item()
            .table_name(&self.table_name)
            .key(KEY_NAME, AttributeValue::S(key.to_string()))
            .send()
            .await
            .map_err(|error| {
                RepositoryError::Unknown(error.to_string())
            })?;

        let item = response.item.ok_or(RepositoryError::NotFound)?;
        let package_json = item.get(KEY_NAME).and_then(|v| v.as_s().ok()).ok_or(RepositoryError::NotFound)?;

        let package: Package = serde_json::from_str(package_json).map_err(|_| RepositoryError::NotFound)?;

        Ok(package)
    }

    async fn update(&self, entity: &Package) -> Result<(), RepositoryError> {
        let item = serde_json::to_string(entity).map_err(|_| RepositoryError::Unknown("Failed to serialize package".to_string()))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
            .item(KEY_NAME, AttributeValue::S(item))
            .send()
            .await
            .map_err(|error| {
                RepositoryError::Unknown(error.to_string())
            })?;
        
        Ok(())
    }
}


#[async_trait]
pub trait DynamoDbClient {
    async fn get_item(&self, user: String, package_name: String) -> Result<GetItemOutput, SdkError<GetItemError>>;
}

pub struct DynamoDbClientImpl {
    client: Client,
}

#[async_trait]
impl DynamoDbClient for DynamoDbClientImpl {
    async fn get_item(&self, user: String, package_name: String) -> Result<GetItemOutput, SdkError<GetItemError>> {
        self.client
            .get_item()
            .table_name(user)
            .key("name", AttributeValue::S(package_name.to_string()))
            .send()
            .await
    }
}
