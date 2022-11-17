use std::{borrow::Cow, collections::BTreeMap};

use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, RequestBuilder, StatusCode, Url};
use serde::{Deserialize, Deserializer};
use tracing::error;

use crate::{
    configuration::{Symbol, Ticker},
    cosmos::Client as CosmosClient,
    provider::{get_supported_denom_pairs, FeedProviderError, Price, Provider},
};

#[derive(Debug, Deserialize)]
struct AssetPrice {
    spot_price: Ratio,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Ratio {
    numerator: u128,
    denominator: u128,
}

impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let point;

        let spot_price = {
            let mut spot_price = String::deserialize(deserializer)?;

            point = if let Some(point) = spot_price.find('.') {
                spot_price = spot_price.trim_end_matches('0').into();

                spot_price.remove(point);

                point
            } else {
                spot_price.len()
            };

            spot_price
        };

        Ok(Ratio {
            numerator: spot_price
                .trim_start_matches('0')
                .parse()
                .map_err(serde::de::Error::custom)?,
            denominator: 10_u128
                .checked_pow(
                    (spot_price.len() - point)
                        .try_into()
                        .map_err(serde::de::Error::custom)?,
                )
                .ok_or_else(|| {
                    serde::de::Error::custom("Couldn't calculate ratio! Exponent too big!")
                })?,
        })
    }
}

pub struct Client {
    base_url: Url,
    currencies: BTreeMap<Ticker, Symbol>,
}

impl Client {
    pub fn new(
        url_str: &str,
        currencies: &BTreeMap<Ticker, Symbol>,
    ) -> Result<Self, FeedProviderError> {
        match Url::parse(url_str) {
            Ok(base_url) => Ok(Self {
                base_url,
                currencies: currencies.clone(),
            }),
            Err(err) => {
                eprintln!("{:?}", err);

                Err(FeedProviderError::InvalidProviderURL(url_str.to_string()))
            }
        }
    }

    fn get_request_builder(&self, url_str: &str) -> Result<RequestBuilder, FeedProviderError> {
        let http_client = ReqwestClient::new();

        self.base_url
            .join(url_str)
            .map(|url| http_client.get(url))
            .map_err(|_| FeedProviderError::URLParsingError)
    }
}

#[async_trait]
impl Provider for Client {
    fn name(&self) -> Cow<'static, str> {
        "Osmosis".into()
    }

    async fn get_spot_prices(
        &self,
        cosm_client: &CosmosClient,
    ) -> Result<Box<[Price]>, FeedProviderError> {
        let mut prices = vec![];

        for (pool_id, (from_ticker, from_symbol), (to_ticker, to_symbol)) in
            get_supported_denom_pairs(cosm_client)
                .await?
                .into_iter()
                .filter_map(|swap| {
                    let from_symbol = self.currencies.get(&swap.from).cloned()?;
                    let to_symbol = self.currencies.get(&swap.to.target).cloned()?;

                    Some((
                        swap.to.pool_id,
                        (swap.from, from_symbol),
                        (swap.to.target, to_symbol),
                    ))
                })
        {
            let resp = self
                .get_request_builder(&format!("pools/{pool_id}/prices"))
                .unwrap()
                .query(&[
                    ("base_asset_denom", from_symbol),
                    ("quote_asset_denom", to_symbol),
                ])
                .send()
                .await?;

            if resp.status() == StatusCode::OK {
                let AssetPrice {
                    spot_price:
                        Ratio {
                            numerator: base,
                            denominator: quote,
                        },
                } = resp.json().await?;

                prices.push(Price::new(from_ticker, base, to_ticker, quote));
            } else {
                error!(
                    from = %from_ticker,
                    to = %to_ticker,
                    "Couldn't resolve spot price! Server returned status code {}!",
                    resp.status().as_u16()
                );
            }
        }

        Ok(prices.into_boxed_slice())
    }
}
