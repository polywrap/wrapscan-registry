use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_sdk_dynamodb::{
    config::Region,
    meta::PKG_VERSION,
    types::{
        AttributeDefinition, KeySchemaElement, KeyType, ProvisionedThroughput, ScalarAttributeType,
    },
    Client,
};

use clap::Parser;

use crate::constants;

pub async fn get_dynamodb_client() -> Client {
    let config = make_config(Opt::parse()).await.unwrap();
    let config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(
            // 8000 is the default dynamodb port
            "http://localhost:8000",
        )
        .build();

    Client::from_conf(config)
}

pub async fn setup_local_db() {
    println!("Setting up local DynamoDB...");

    let table_name =
        std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");

    let config = make_config(Opt::parse()).await.unwrap();
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(
            // 8000 is the default dynamodb port
            "http://localhost:8000",
        )
        .build();

    let client = Client::from_conf(dynamodb_local_config);

    match client.describe_table().table_name(&table_name).send().await {
        Ok(_) => {
            println!("Table `{}` already exists. Skipping.", &table_name);
            return;
        }
        Err(_) => {
            println!("Table `{}` does not exist. Creating it...", &table_name);
        }
    }

    let ad = AttributeDefinition::builder()
        .attribute_name(constants::PACKAGES_TABLE_KEY_NAME)
        .attribute_type(ScalarAttributeType::S)
        .build();
    let ks = KeySchemaElement::builder()
        .attribute_name(constants::PACKAGES_TABLE_KEY_NAME)
        .key_type(KeyType::Hash)
        .build();
    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(5)
        .write_capacity_units(5)
        .build();

    client
        .create_table()
        .table_name(&table_name)
        .attribute_definitions(ad)
        .key_schema(ks)
        .provisioned_throughput(pt)
        .send()
        .await
        .unwrap();

    println!("DynamoDB table `{}` created.", &table_name);
}

#[derive(Debug, Parser)]
pub struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    pub region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    pub verbose: bool,
}

pub fn make_region_provider(region: Option<String>) -> RegionProviderChain {
    RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"))
}

pub async fn make_config(opt: Opt) -> Result<SdkConfig, aws_sdk_dynamodb::Error> {
    let region_provider = make_region_provider(opt.region);

    println!();
    if opt.verbose {
        println!("DynamoDB client version: {}", PKG_VERSION);
        println!(
            "Region:                  {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!();
    }

    Ok(aws_config::from_env().region(region_provider).load().await)
}
