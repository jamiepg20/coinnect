//! Use this module to interact with Poloniex through a Generic API.
//! This a more convenient and safe way to deal with the exchange since methods return a Result<>
//! but this generic API does not provide all the functionnality that Poloniex offers.

use exchange::ExchangeApi;
use poloniex::api::PoloniexApi;

use error::*;
use pair::Pair;
use types::*;
use poloniex::utils;
use helpers;

impl ExchangeApi for PoloniexApi {
    fn ticker(&mut self, pair: Pair) -> Result<Ticker> {
        let pair_name = match utils::get_pair_string(&pair) {
            Some(name) => name,
            None => return Err(ErrorKind::PairUnsupported.into()),
        };
        let raw_response = self.return_ticker()?;

        let result = utils::parse_result(&raw_response)?;

        let price =
            result[*pair_name]["last"]
                .as_str()
                .ok_or_else(|| ErrorKind::MissingField(format!("{}.last", pair_name)))?
                .parse::<f64>()
                .chain_err(|| ErrorKind::InvalidFieldFormat(format!("{}.last", pair_name)))?;
        let ask =
            result[*pair_name]["lowestAsk"]
                .as_str()
                .ok_or_else(|| ErrorKind::MissingField(format!("{}.lowestAsk", pair_name)))?
                .parse::<f64>()
                .chain_err(|| ErrorKind::InvalidFieldFormat(format!("{}.lowestAsk", pair_name)))?;
        let bid =
            result[*pair_name]["highestBid"]
                .as_str()
                .ok_or_else(|| ErrorKind::MissingField(format!("{}.hightestBid", pair_name)))?
                .parse::<f64>()
                .chain_err(|| ErrorKind::InvalidFieldFormat(format!("{}.highestBid", pair_name)))?;
        let vol =
            result[*pair_name]["quoteVolume"]
                .as_str()
                .ok_or_else(|| ErrorKind::MissingField(format!("{}.quoteVolume", pair_name)))?
                .parse::<f64>()
                .chain_err(|| ErrorKind::InvalidFieldFormat(format!("{}.quoteVolume", pair_name)))?;

        Ok(Ticker {
               timestamp: helpers::get_unix_timestamp_ms(),
               pair: pair,
               last_trade_price: price,
               lowest_ask: ask,
               highest_bid: bid,
               volume: Some(vol),
           })
    }

    fn orderbook(&mut self, pair: Pair) -> Result<Orderbook> {
        let pair_name = match utils::get_pair_string(&pair) {
            Some(name) => name,
            None => return Err(ErrorKind::PairUnsupported.into()),
        };
        let raw_response = self.return_order_book(pair_name, "1000")?; // 1000 entries max

        let result = utils::parse_result(&raw_response)?;

        let mut ask_offers = Vec::new();
        let mut bid_offers = Vec::new();

        let ask_array =
            result["asks"]
                .as_array()
                .ok_or_else(|| ErrorKind::InvalidFieldFormat(format!("{}", result["asks"])))?;
        let bid_array =
            result["bids"]
                .as_array()
                .ok_or_else(|| ErrorKind::InvalidFieldFormat(format!("{}", result["asks"])))?;

        for ask in ask_array {
            let price = ask[0]
                .as_str()
                .ok_or_else(|| ErrorKind::InvalidFieldFormat(format!("{}", ask[0])))?
                .parse::<f64>()
                .chain_err(|| ErrorKind::InvalidFieldFormat(format!("{}", ask[0])))?;

            let volume = ask[1]
                .as_f64()
                .ok_or_else(|| ErrorKind::InvalidFieldFormat(format!("{}", ask[1])))?;
            ask_offers.push((price, volume));
        }

        for bid in bid_array {
            let price = bid[0]
                .as_str()
                .ok_or_else(|| ErrorKind::InvalidFieldFormat(format!("{}", bid[0])))?
                .parse::<f64>()
                .chain_err(|| ErrorKind::InvalidFieldFormat(format!("{}", bid[0])))?;
            let volume = bid[1]
                .as_f64()
                .ok_or_else(|| ErrorKind::InvalidFieldFormat(format!("{}", bid[1])))?;
            bid_offers.push((price, volume));
        }

        Ok(Orderbook {
               timestamp: helpers::get_unix_timestamp_ms(),
               pair: pair,
               asks: ask_offers,
               bids: bid_offers,
           })
    }

    fn add_order(&mut self,
                 order_type: OrderType,
                 pair: Pair,
                 quantity: Volume,
                 price: Option<Price>)
                 -> Result<OrderInfo> {
        let pair_name = match utils::get_pair_string(&pair) {
            Some(name) => name,
            None => return Err(ErrorKind::PairUnsupported.into()),
        };

        // The trick is to use minimal (0.0) and "maximum" (999..) price to simulate market order
        let raw_response = match order_type {
            // Unwrap safe here with the check above.
            OrderType::BuyLimit => {
                if price.is_none() {
                    return Err(ErrorKind::MissingPrice.into());
                }

                self.buy(pair_name,
                         &price.unwrap().to_string(),
                         &quantity.to_string())
            }
            OrderType::BuyMarket => {
                self.buy(pair_name, "9999999999999999999", &quantity.to_string())
            }
            OrderType::SellLimit => {
                if price.is_none() {
                    return Err(ErrorKind::MissingPrice.into());
                }

                self.sell(pair_name,
                          &price.unwrap().to_string(),
                          &quantity.to_string())
            }
            OrderType::SellMarket => self.sell(pair_name, "0.0", &quantity.to_string()),
        }?;

        let result = utils::parse_result(&raw_response)?;

        Ok(OrderInfo {
               timestamp: helpers::get_unix_timestamp_ms(),
               identifier: vec![result["orderNumber"]
                                    .as_f64()
                                    .ok_or_else(|| {
                                                    ErrorKind::MissingField("orderNumber"
                                                                                .to_string())
                                                })?
                                    .to_string()],
           })
    }

    fn balances(&mut self) -> Result<Balances> {
        unimplemented!();
    }
}
