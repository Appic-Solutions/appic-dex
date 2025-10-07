use candid::CandidType;
use ethnum::U256;
use minicbor::{Decode, Encode};
use rlp::{DecoderError, Rlp};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::LazyLock;

#[derive(
    Encode,
    Decode,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Hash,
    Copy,
    CandidType,
    Deserialize,
)]
pub enum Blockchain {
    #[n(0)]
    ICP,
    #[n(1)]
    /// chain_id
    Evm(#[n(0)] u64),
}

impl Blockchain {
    pub fn is_evm(&self) -> bool {
        match self {
            Blockchain::ICP => false,
            Blockchain::Evm(_) => true,
        }
    }

    pub fn to_id(&self) -> u64 {
        match self {
            Blockchain::ICP => 0,
            Blockchain::Evm(id) => *id,
        }
    }
}

impl Display for Blockchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CrossChainQuote {
    #[cbor(n(0), with = "crate::cbor::u256")]
    pub total_amount_in: U256,
    #[cbor(n(1), with = "crate::cbor::u256")]
    pub total_amount_out: U256,
    #[cbor(n(2), with = "crate::cbor::u256::option")]
    pub total_min_amount_out: Option<U256>,
    #[n(3)]
    pub total_slippage: Option<String>,
    #[n(4)]
    pub steps: Vec<CrossChainStep>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct CrossChainStep {
    #[n(0)]
    pub chain_id: Blockchain,
    #[cbor(n(1), with = "crate::cbor::u256")]
    pub amount_in: U256,
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub amount_out: U256,
    #[cbor(n(3), with = "crate::cbor::u256::option")]
    pub min_amount_out: Option<U256>,
    #[n(4)]
    pub slippage: Option<String>,
    #[cbor(n(5), with = "crate::cbor::u256::option")]
    pub gas_limit: Option<U256>,
    #[cbor(n(6), with = "crate::cbor::u256::option")]
    pub max_gas_fee: Option<U256>,
    #[n(7)]
    pub gas_price_usd: Option<String>,
    #[n(8)]
    pub canister_fee_usd: Option<String>,
    #[n(9)]
    pub route: Vec<PoolHop>,
    #[n(10)]
    pub qswap_data: Option<QSwapData>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PoolHop {
    #[n(0)]
    pub sell_token: String,
    #[n(1)]
    pub buy_token: String,
    #[n(2)]
    pub fee: u32,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct QSwapData {
    #[n(0)]
    pub commands: Vec<u8>,
    #[n(1)]
    pub command_data: Vec<String>,
    #[cbor(n(2), with = "crate::cbor::u256")]
    pub deadline: U256,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, CandidType, Deserialize)]
pub enum RlpDecodeError {
    #[n(0)]
    InvalidRlpData,
    #[n(1)]
    InvalidStructure,
    #[n(2)]
    InvalidDataType,
    #[n(3)]
    MissingField,
    #[n(4)]
    InvalidChainId(#[n(0)] String),
    #[n(5)]
    InvalidAmount,
    #[n(6)]
    InvalidTokenAddress(#[n(0)] String),
    #[n(7)]
    DataTooLarge,
    #[n(8)]
    VersionMismatch,
}

impl std::fmt::Display for RlpDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RlpDecodeError::InvalidRlpData => write!(f, "Invalid RLP data format"),
            RlpDecodeError::InvalidStructure => write!(f, "Invalid RLP structure"),
            RlpDecodeError::InvalidDataType => write!(f, "Invalid data type"),
            RlpDecodeError::MissingField => write!(f, "Missing required field"),
            RlpDecodeError::InvalidChainId(chain) => write!(f, "Invalid chain ID: {chain}"),
            RlpDecodeError::InvalidAmount => write!(f, "Invalid amount value"),
            RlpDecodeError::InvalidTokenAddress(addr) => {
                write!(f, "Invalid token address: {addr}")
            }
            RlpDecodeError::DataTooLarge => write!(f, "Quote data exceeds maximum size limit"),
            RlpDecodeError::VersionMismatch => write!(f, "RLP format version not supported"),
        }
    }
}

impl From<DecoderError> for RlpDecodeError {
    fn from(_: DecoderError) -> Self {
        RlpDecodeError::InvalidRlpData
    }
}

static SUPPORTED_CHAINS: LazyLock<HashMap<&Blockchain, &str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(&Blockchain::Evm(1), "Ethereum");
    m.insert(&Blockchain::Evm(56), "BSC");
    m.insert(&Blockchain::Evm(8453), "Base");
    m.insert(&Blockchain::Evm(137), "Ethereum");
    m.insert(&Blockchain::ICP, "Internet Computer");
    m
});

const MAX_QUOTE_SIZE: usize = 1024 * 1024; // 1MB limit
const MAX_STEPS: usize = 10;
const MAX_HOPS_PER_STEP: usize = 5;
const NATIVE_TOKEN_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

pub struct RlpDecoder;

impl RlpDecoder {
    pub fn decode_cross_chain_data(encoded_data: &str) -> Result<CrossChainQuote, RlpDecodeError> {
        if !encoded_data.starts_with("0x") {
            return Err(RlpDecodeError::InvalidRlpData);
        }

        Self::validate_input_size(&encoded_data[2..])?;
        Self::decode_cross_chain_data_internal(&encoded_data[2..])
    }

    fn decode_cross_chain_data_internal(
        encoded_data: &str,
    ) -> Result<CrossChainQuote, RlpDecodeError> {
        println!("encoded_data{encoded_data}");
        let data = hex::decode(encoded_data).map_err(|_| RlpDecodeError::InvalidRlpData)?;
        match Self::decode_quote_data(&data) {
            Ok(data) => {
                Self::validate_cross_chain_quote_data(&data)?;
                Ok(data)
            }
            Err(e) => Err(e),
        }
    }

    fn decode_quote_data(encoded_data: &[u8]) -> Result<CrossChainQuote, RlpDecodeError> {
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

        Ok(CrossChainQuote {
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

        let chain_id_string = step_rlp
            .val_at::<String>(0)
            .map_err(|_| RlpDecodeError::InvalidDataType)?;

        let chain_id = if chain_id_string == "icp" {
            Ok::<Blockchain, RlpDecodeError>(Blockchain::ICP)
        } else {
            let chain_id_number = chain_id_string
                .parse::<u64>()
                .map_err(|e| RlpDecodeError::InvalidChainId(e.to_string()))?;

            Ok(Blockchain::Evm(chain_id_number))
        }?;

        let amount_in = Self::parse_u256(
            &step_rlp
                .val_at::<String>(1)
                .map_err(|_| RlpDecodeError::InvalidDataType)?,
        )?;
        let amount_out = Self::parse_u256(
            &step_rlp
                .val_at::<String>(2)
                .map_err(|_| RlpDecodeError::InvalidDataType)?,
        )?;
        let min_amount_out = step_rlp
            .val_at::<String>(3)
            .map_err(|_| RlpDecodeError::InvalidDataType)
            .ok()
            .and_then(|s| Self::parse_u256(&s).ok());
        let slippage = step_rlp
            .val_at::<String>(4)
            .map_err(|_| RlpDecodeError::InvalidDataType)
            .ok();
        let gas_limit = step_rlp
            .val_at::<String>(5)
            .map_err(|_| RlpDecodeError::InvalidDataType)
            .ok()
            .and_then(|s| Self::parse_u256(&s).ok());
        let max_gas_fee = step_rlp
            .val_at::<String>(6)
            .map_err(|_| RlpDecodeError::InvalidDataType)
            .ok()
            .and_then(|s| Self::parse_u256(&s).ok());
        let gas_price_usd = step_rlp
            .val_at::<String>(7)
            .map_err(|_| RlpDecodeError::InvalidDataType)
            .ok();
        let canister_fee_usd = step_rlp
            .val_at::<String>(8)
            .map_err(|_| RlpDecodeError::InvalidDataType)
            .ok();

        let route_rlp = step_rlp.at(9).map_err(|_| RlpDecodeError::InvalidRlpData)?;
        let mut route = Vec::new();
        if let Ok(count) = route_rlp.item_count() {
            for i in 0..count {
                if let Ok(hop_rlp) = route_rlp.at(i)
                    && let Ok(hop) = Self::decode_pool_hop(hop_rlp)
                {
                    route.push(hop);
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

    fn decode_qswap_data(
        step_rlp: &Rlp,
        commands_idx: usize,
        data_idx: usize,
        deadline_idx: usize,
    ) -> Result<Option<QSwapData>, RlpDecodeError> {
        let commands_rlp = step_rlp
            .at(commands_idx)
            .map_err(|_| RlpDecodeError::InvalidRlpData)?;
        let mut commands = Vec::new();
        if let Ok(count) = commands_rlp.item_count() {
            for i in 0..count {
                if let Ok(command_str) = commands_rlp.val_at::<String>(i)
                    && let Ok(command) = command_str.parse::<u8>()
                {
                    commands.push(command);
                }
            }
        }

        let data_rlp = step_rlp
            .at(data_idx)
            .map_err(|_| RlpDecodeError::InvalidRlpData)?;
        let mut command_data = Vec::new();
        if let Ok(count) = data_rlp.item_count() {
            for i in 0..count {
                if let Ok(data) = data_rlp.val_at::<String>(i) {
                    command_data.push(data);
                }
            }
        }

        let deadline = if let Ok(deadline_str) = step_rlp.val_at::<String>(deadline_idx) {
            Self::parse_u256(&deadline_str).unwrap_or(U256::ZERO)
        } else {
            U256::ZERO
        };

        Ok(Some(QSwapData {
            commands,
            command_data,
            deadline,
        }))
    }

    fn decode_pool_hop(hop_rlp: Rlp) -> Result<PoolHop, RlpDecodeError> {
        if hop_rlp.item_count()? != 3 {
            return Err(RlpDecodeError::InvalidStructure);
        }

        let sell_token = String::from_utf8(hop_rlp.val_at::<Vec<u8>>(0)?)
            .map_err(|_| RlpDecodeError::InvalidDataType)?;
        let buy_token = String::from_utf8(hop_rlp.val_at::<Vec<u8>>(1)?)
            .map_err(|_| RlpDecodeError::InvalidDataType)?;
        let fee = hop_rlp
            .val_at::<String>(2)?
            .parse::<u32>()
            .map_err(|_| RlpDecodeError::InvalidDataType)?;

        Ok(PoolHop {
            sell_token,
            buy_token,
            fee,
        })
    }

    fn validate_input_size(data: &str) -> Result<(), RlpDecodeError> {
        if data.len() > MAX_QUOTE_SIZE {
            return Err(RlpDecodeError::DataTooLarge);
        }
        Ok(())
    }

    fn validate_chain_id(chain_id: &Blockchain) -> Result<(), RlpDecodeError> {
        if !SUPPORTED_CHAINS.contains_key(chain_id) {
            return Err(RlpDecodeError::InvalidChainId(chain_id.to_string()));
        }
        Ok(())
    }

    fn validate_token_address(address: &str) -> Result<(), RlpDecodeError> {
        if address.is_empty() {
            return Err(RlpDecodeError::InvalidTokenAddress(
                "empty address".to_string(),
            ));
        }

        // Native token address
        if address == NATIVE_TOKEN_ADDRESS {
            return Ok(());
        }

        // EVM token address (0x prefixed, 42 chars total)
        if address.starts_with("0x") && address.len() == 42 {
            return Ok(());
        }

        // ICP canister ID (ends with -cai, 5-63 chars)
        if address.len() >= 5 && address.len() <= 63 {
            return Ok(());
        }

        // Generic token identifier (allow alphanumeric, hyphens, underscores)
        if address.len() <= 100
            && address
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Ok(());
        }

        Err(RlpDecodeError::InvalidTokenAddress(format!(
            "invalid format: {address}"
        )))
    }

    fn validate_amount(amount: &U256) -> Result<(), RlpDecodeError> {
        if *amount == U256::ZERO {
            return Ok(());
        }

        if *amount > U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap() {
            return Err(RlpDecodeError::InvalidAmount);
        }

        Ok(())
    }

    fn validate_cross_chain_quote_data(data: &CrossChainQuote) -> Result<(), RlpDecodeError> {
        Self::validate_amount(&data.total_amount_in)?;
        Self::validate_amount(&data.total_amount_out)?;

        if let Some(min_amount) = &data.total_min_amount_out {
            Self::validate_amount(min_amount)?;
        }

        if data.steps.len() > MAX_STEPS {
            return Err(RlpDecodeError::InvalidStructure);
        }

        for step in &data.steps {
            Self::validate_cross_chain_step(step)?;
        }

        Ok(())
    }

    fn validate_cross_chain_step(step: &CrossChainStep) -> Result<(), RlpDecodeError> {
        Self::validate_chain_id(&step.chain_id)?;
        Self::validate_amount(&step.amount_in)?;
        Self::validate_amount(&step.amount_out)?;

        if let Some(min_amount) = &step.min_amount_out {
            Self::validate_amount(min_amount)?;
        }

        if let Some(gas_limit) = &step.gas_limit {
            Self::validate_amount(gas_limit)?;
        }

        if let Some(max_gas_fee) = &step.max_gas_fee {
            Self::validate_amount(max_gas_fee)?;
        }

        if step.route.len() > MAX_HOPS_PER_STEP {
            return Err(RlpDecodeError::InvalidStructure);
        }

        for hop in &step.route {
            Self::validate_token_address(&hop.sell_token)?;
            Self::validate_token_address(&hop.buy_token)?;
        }

        Ok(())
    }

    pub fn get_supported_chains() -> HashMap<&'static Blockchain, &'static str> {
        SUPPORTED_CHAINS.clone()
    }

    pub fn get_decoder_stats() -> DecoderStats {
        DecoderStats {
            supported_chains: SUPPORTED_CHAINS.len(),
            max_quote_size: MAX_QUOTE_SIZE,
            max_steps: MAX_STEPS,
            max_hops_per_step: MAX_HOPS_PER_STEP,
        }
    }
}

#[derive(Debug)]
pub struct DecoderStats {
    pub supported_chains: usize,
    pub max_quote_size: usize,
    pub max_steps: usize,
    pub max_hops_per_step: usize,
}
