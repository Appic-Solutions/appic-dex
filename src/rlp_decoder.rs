use rlp::{Rlp, RlpStream, DecoderError};
use ethnum::U256;

#[derive(Debug)]
pub struct CrossChainQuote {
    pub success: bool,
    pub data: Option<CrossChainQuoteData>,
    pub encoded_data: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct CrossChainQuoteData {
    pub total_amount_in: U256,
    pub total_amount_out: U256,
    pub total_min_amount_out: Option<U256>,
    pub total_slippage: Option<String>,
    pub steps: Vec<CrossChainStep>,
}

#[derive(Debug)]
pub struct CrossChainStep {
    pub chain_id: String,
    pub amount_in: U256,
    pub amount_out: U256,
    pub min_amount_out: Option<U256>,
    pub slippage: Option<String>,
    pub gas_limit: Option<U256>,
    pub max_gas_fee: Option<U256>,
    pub gas_price_usd: Option<String>,
    pub canister_fee_usd: Option<String>,
    pub route: Vec<PoolHop>,
    pub qswap_data: Option<QSwapData>,
}

#[derive(Debug)]
pub struct PoolHop {
    pub sell_token: String,
    pub buy_token: String,
    pub fee: u32,
}

#[derive(Debug)]
pub struct QSwapData {
    pub commands: Vec<u32>,
    pub command_data: Vec<String>,
    pub deadline: U256,
}

#[derive(Debug)]
pub enum RlpDecodeError {
    InvalidRlpData,
    InvalidStructure,
    InvalidDataType,
    MissingField,
}

impl std::fmt::Display for RlpDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RlpDecodeError::InvalidRlpData => write!(f, "Invalid RLP data format"),
            RlpDecodeError::InvalidStructure => write!(f, "Invalid RLP structure"),
            RlpDecodeError::InvalidDataType => write!(f, "Invalid data type"),
            RlpDecodeError::MissingField => write!(f, "Missing required field"),
        }
    }
}

impl From<DecoderError> for RlpDecodeError {
    fn from(_: DecoderError) -> Self {
        RlpDecodeError::InvalidRlpData
    }
}

pub struct RlpDecoder;

impl RlpDecoder {
    pub fn decode_cross_chain_data(encoded_data: &[u8]) -> Result<CrossChainQuote, RlpDecodeError> {
        match Self::decode_quote_data(encoded_data) {
            Ok(data) => Ok(CrossChainQuote {
                success: true,
                data: Some(data),
                encoded_data: Some(hex::encode(encoded_data)),
                error: None,
            }),
            Err(e) => Ok(CrossChainQuote {
                success: false,
                data: None,
                encoded_data: Some(hex::encode(encoded_data)),
                error: Some(e.to_string()),
            }),
        }
    }

    fn decode_quote_data(encoded_data: &[u8]) -> Result<CrossChainQuoteData, RlpDecodeError> {
        let rlp = Rlp::new(encoded_data);
        
        let item_count = rlp.item_count()?;
        if item_count != 5 && item_count != 8 {
            return Err(RlpDecodeError::InvalidStructure);
        }

        let total_amount_in = Self::parse_u256(&rlp.val_at::<String>(0)?)?;
        let total_amount_out = Self::parse_u256(&rlp.val_at::<String>(1)?)?;
        let total_min_amount_out = Some(Self::parse_u256(&rlp.val_at::<String>(2)?)?);
        let total_slippage = Some(rlp.val_at::<String>(3)?);
        
        let steps_rlp = rlp.at(4)?;
        let mut steps = Vec::new();
        
        for i in 0..steps_rlp.item_count()? {
            let step_rlp = steps_rlp.at(i)?;
            if step_rlp.item_count()? > 0 {
                let step = Self::decode_step(step_rlp)?;
                steps.push(step);
            }
        }

        Ok(CrossChainQuoteData {
            total_amount_in,
            total_amount_out,
            total_min_amount_out,
            total_slippage,
            steps,
        })
    }

    fn parse_u256(s: &str) -> Result<U256, RlpDecodeError> {
        s.parse().map_err(|_| RlpDecodeError::InvalidDataType)
    }

    fn decode_step(step_rlp: Rlp) -> Result<CrossChainStep, RlpDecodeError> {
        let item_count = step_rlp.item_count()?;
        if item_count < 13 {
            return Err(RlpDecodeError::InvalidStructure);
        }

        let chain_id = step_rlp.val_at::<String>(0).map_err(|_| RlpDecodeError::InvalidDataType)?;
        let amount_in = Self::parse_u256(&step_rlp.val_at::<String>(1).map_err(|_| RlpDecodeError::InvalidDataType)?)?;
        let amount_out = Self::parse_u256(&step_rlp.val_at::<String>(2).map_err(|_| RlpDecodeError::InvalidDataType)?)?;
        let min_amount_out = step_rlp.val_at::<String>(3).map_err(|_| RlpDecodeError::InvalidDataType).ok().and_then(|s| Self::parse_u256(&s).ok());
        let slippage = step_rlp.val_at::<String>(4).map_err(|_| RlpDecodeError::InvalidDataType).ok();
        let gas_limit = step_rlp.val_at::<String>(5).map_err(|_| RlpDecodeError::InvalidDataType).ok().and_then(|s| Self::parse_u256(&s).ok());
        let max_gas_fee = step_rlp.val_at::<String>(6).map_err(|_| RlpDecodeError::InvalidDataType).ok().and_then(|s| Self::parse_u256(&s).ok());
        let gas_price_usd = step_rlp.val_at::<String>(7).map_err(|_| RlpDecodeError::InvalidDataType).ok();
        let canister_fee_usd = step_rlp.val_at::<String>(8).map_err(|_| RlpDecodeError::InvalidDataType).ok();

        let route_rlp = step_rlp.at(9).map_err(|_| RlpDecodeError::InvalidRlpData)?;
        let mut route = Vec::new();
        if let Ok(count) = route_rlp.item_count() {
            for i in 0..count {
                if let Ok(hop_rlp) = route_rlp.at(i) {
                    if let Ok(hop) = Self::decode_pool_hop(hop_rlp) {
                        route.push(hop);
                    }
                }
            }
        }

        let qswap_data = Self::decode_qswap_data(&step_rlp, 10, 11, 12)?;

        Ok(CrossChainStep {
            chain_id,
            amount_in,
            amount_out,
            min_amount_out,
            slippage,
            gas_limit,
            max_gas_fee,
            gas_price_usd,
            canister_fee_usd,
            route,
            qswap_data,
        })
    }

    fn decode_qswap_data(step_rlp: &Rlp, commands_idx: usize, data_idx: usize, deadline_idx: usize) -> Result<Option<QSwapData>, RlpDecodeError> {
        let commands_rlp = step_rlp.at(commands_idx).map_err(|_| RlpDecodeError::InvalidRlpData)?;
        let mut commands = Vec::new();
        if let Ok(count) = commands_rlp.item_count() {
            for i in 0..count {
                if let Ok(command_str) = commands_rlp.val_at::<String>(i) {
                    if let Ok(command) = command_str.parse::<u32>() {
                        commands.push(command);
                    }
                }
            }
        }

        let data_rlp = step_rlp.at(data_idx).map_err(|_| RlpDecodeError::InvalidRlpData)?;
        let mut command_data = Vec::new();
        if let Ok(count) = data_rlp.item_count() {
            for i in 0..count {
                if let Ok(data) = data_rlp.val_at::<Vec<u8>>(i) {
                    command_data.push(hex::encode(data));
                }
            }
        }

        let deadline = if let Ok(deadline_str) = step_rlp.val_at::<String>(deadline_idx) {
            Self::parse_u256(&deadline_str).unwrap_or(U256::ZERO)
        } else {
            U256::ZERO
        };

        if commands.is_empty() && command_data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(QSwapData {
                commands,
                command_data,
                deadline,
            }))
        }
    }

    fn decode_pool_hop(hop_rlp: Rlp) -> Result<PoolHop, RlpDecodeError> {
        if hop_rlp.item_count()? != 3 {
            return Err(RlpDecodeError::InvalidStructure);
        }

        let sell_token = String::from_utf8(hop_rlp.val_at::<Vec<u8>>(0)?).map_err(|_| RlpDecodeError::InvalidDataType)?;
        let buy_token = String::from_utf8(hop_rlp.val_at::<Vec<u8>>(1)?).map_err(|_| RlpDecodeError::InvalidDataType)?;
        let fee = hop_rlp.val_at::<String>(2)?.parse::<u32>().map_err(|_| RlpDecodeError::InvalidDataType)?;

        Ok(PoolHop {
            sell_token,
            buy_token,
            fee,
        })
    }

}