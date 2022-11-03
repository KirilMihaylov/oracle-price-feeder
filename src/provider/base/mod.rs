use serde::{Deserialize, Serialize};

pub use self::{errors::FeedProviderError, provider::*};

mod errors;
mod provider;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[must_use]
pub struct Coin {
    pub amount: u128,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[must_use]
pub struct Price {
    amount: Coin,
    amount_quote: Coin,
}

impl Price {
    pub fn new<S1, S2>(symbol1: S1, base: u128, symbol2: S2, quote: u128) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self::new_from_coins(
            Coin {
                amount: base,
                symbol: symbol1.into(),
            },
            Coin {
                amount: quote,
                symbol: symbol2.into(),
            },
        )
    }

    pub fn new_from_coins(amount: Coin, amount_quote: Coin) -> Self {
        Price {
            amount,
            amount_quote,
        }
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.amount.amount == 0 || self.amount_quote.amount == 0
    }
}