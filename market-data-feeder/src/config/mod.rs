use std::{
    collections::BTreeMap,
    env::{self, var},
};

use serde::Deserialize;
use thiserror::Error as ThisError;

use chain_comms::config::Node;

pub(crate) use self::currencies::Currencies;

mod currencies;

pub(crate) type TickerUnsized = str;
pub(crate) type Ticker = String;

pub(crate) type Symbol = String;

#[derive(Debug, Deserialize)]
#[must_use]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) struct Config {
    tick_time: u64,
    providers: BTreeMap<String, ProviderWithComparison>,
    comparison_providers: BTreeMap<String, ComparisonProvider>,
    oracle_addr: String,
    gas_limit: u64,
    node: Node,
}

impl Config {
    #[must_use]
    pub fn tick_time(&self) -> u64 {
        self.tick_time
    }

    pub fn providers(&self) -> &BTreeMap<String, ProviderWithComparison> {
        &self.providers
    }

    pub fn comparison_providers(&self) -> &BTreeMap<String, ComparisonProvider> {
        &self.comparison_providers
    }

    #[must_use]
    pub fn oracle_addr(&self) -> &str {
        &self.oracle_addr
    }

    #[must_use]
    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }
}

impl AsRef<Node> for Config {
    fn as_ref(&self) -> &Node {
        &self.node
    }
}

pub(crate) trait ProviderConfig: Sync + Send {
    fn name(&self) -> &str;

    fn misc(&self) -> &BTreeMap<String, toml::Value>;
}

pub(crate) trait ProviderConfigExt<const COMPARISON: bool>: ProviderConfig {
    fn fetch_from_env(id: &str, name: &str) -> Result<String, EnvError>;
}

impl<T> ProviderConfigExt<true> for T
where
    T: ProviderConfig + ?Sized,
{
    fn fetch_from_env(id: &str, name: &str) -> Result<String, EnvError> {
        let name: String = format!(
            "COMPARISON_PROVIDER_{id}_{field}",
            id = id.to_ascii_uppercase(),
            field = name.to_ascii_uppercase()
        );

        var(&name).map_err(|error: env::VarError| EnvError(name, error))
    }
}

#[derive(Debug, ThisError)]
#[error("Variable name: \"{0}\". Cause: {1}")]
pub(crate) struct EnvError(String, env::VarError);

#[derive(Debug, Clone, Deserialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub(crate) struct Provider {
    name: String,
    #[serde(flatten)]
    pub misc: BTreeMap<String, toml::Value>,
}

impl ProviderConfig for Provider {
    fn name(&self) -> &str {
        &self.name
    }

    fn misc(&self) -> &BTreeMap<String, toml::Value> {
        &self.misc
    }
}

impl ProviderConfigExt<false> for Provider {
    fn fetch_from_env(id: &str, name: &str) -> Result<String, EnvError> {
        let name: String = format!(
            "PROVIDER_{id}_{field}",
            id = id.to_ascii_uppercase(),
            field = name.to_ascii_uppercase()
        );

        var(&name).map_err(|error: env::VarError| EnvError(name, error))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub(crate) struct ProviderWithComparison {
    pub comparison: Option<ComparisonProviderIdAndMaxDeviation>,
    #[serde(flatten)]
    pub provider: Provider,
}

#[derive(Debug, Clone, Deserialize)]
#[must_use]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) struct ComparisonProviderIdAndMaxDeviation {
    pub provider_id: String,
    pub max_deviation_exclusive: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[must_use]
#[serde(rename_all = "snake_case")]
pub(crate) struct ComparisonProvider {
    #[serde(flatten)]
    pub provider: Provider,
}
