use candid::Principal;
use ethnum::U256;

use crate::{
    address::Address,
    cross_chain::{
        parser::parse_recipient,
        rlp_decoder::{self, Blockchain, RlpDecodeError},
        types::CrosschainSwapOrder,
    },
    minter_client::minter_types::{MinterKey, SwapOrderCreationError},
    state::read_state,
    swap_id::SwapTxId,
    validation::swap_args::create_validated_swap_args_from_rlp_swap_step,
};

pub fn create_swap_order(
    encoded_swap_data: &str,
    mut amount_in: U256,
    mut token_in_icp_step: Principal,
    from_minter: Option<&MinterKey>,
    tx_id: &str,
    from_address: Option<Address>,
    from_principal: Option<Principal>,
    recipient: &str,
) -> Result<CrosschainSwapOrder, SwapOrderCreationError> {
    let crosschain_quote = rlp_decoder::RlpDecoder::decode_cross_chain_data(encoded_swap_data)
        .map_err(SwapOrderCreationError::InvalidRlpData)?;
    let swap_steps = crosschain_quote.steps.len();
    if swap_steps < 2 {
        return Err(SwapOrderCreationError::InvalidRlpData(
            RlpDecodeError::MissingField,
        ));
    }
    let from_chain = crosschain_quote.steps[0].chain_id;
    let to_chain = crosschain_quote.steps[swap_steps - 1].chain_id;
    let parsed_recipient = parse_recipient(recipient, to_chain).map_err(|e| {
        SwapOrderCreationError::InvalidRecipient(format!("Invalid recipient: {:?}", e))
    })?;
    if from_chain == to_chain {
        return Err(SwapOrderCreationError::InvalidOriginAndDestinationChain);
    }
    if from_chain == Blockchain::ICP {
        if swap_steps != 2 || !to_chain.is_evm() {
            return Err(SwapOrderCreationError::InvalidToChain);
        }
        let from = from_principal.ok_or(SwapOrderCreationError::InvalidFromAddress)?;
        let to_chain_id = match to_chain {
            Blockchain::Evm(chain_id) => chain_id,
            _ => return Err(SwapOrderCreationError::InvalidToChain),
        };

        let (to_minter, to_minter_info) = read_state(|s| s.get_minter_by_chain_id(to_chain_id))
            .ok_or(SwapOrderCreationError::InvalidToChain)?;

        amount_in = crosschain_quote.steps[0].amount_in;

        // in case the route is empty it means the token should already be USDC so we pick USDC
        token_in_icp_step = if crosschain_quote.steps[0].route.is_empty() {
            to_minter_info.twin_usdc_principal
        } else {
            Principal::from_text(&crosschain_quote.steps[0].route[0].sell_token)
                .map_err(|_| SwapOrderCreationError::InvalidTokenIn)?
        };

        let icp_swap_request = create_validated_swap_args_from_rlp_swap_step(
            crosschain_quote.steps[0].clone(),
            amount_in,
            token_in_icp_step,
        )
        .map_err(SwapOrderCreationError::InvalidIcpSwapStep)?;

        if icp_swap_request.token_out() != to_minter_info.twin_usdc_principal {
            return Err(SwapOrderCreationError::InvalidTokenIn);
        }
        Ok(CrosschainSwapOrder::IcpToEvm {
            tx_id: SwapTxId(tx_id.to_string()),
            from,
            recipient: parsed_recipient,
            icp_swap_request,
            evm_swap_step: crosschain_quote.steps[1].clone(),
            to_minter,
        })
    } else {
        if !from_chain.is_evm() {
            return Err(SwapOrderCreationError::InvalidOriginChain);
        }
        let from_minter = from_minter
            .ok_or(SwapOrderCreationError::InvalidMinter)?
            .clone();
        let from_address = from_address.ok_or(SwapOrderCreationError::InvalidFromAddress)?;
        if from_principal.is_some() {
            return Err(SwapOrderCreationError::InvalidOriginChain);
        }
        if swap_steps == 2 && to_chain == Blockchain::ICP {
            // Handle EVM to ICP swap configuration
            let icp_swap_request = create_validated_swap_args_from_rlp_swap_step(
                crosschain_quote.steps[1].clone(),
                amount_in,
                token_in_icp_step,
            )
            .map_err(SwapOrderCreationError::InvalidIcpSwapStep)?;
            Ok(CrosschainSwapOrder::EvmToIcp {
                tx_id: SwapTxId(tx_id.to_string()),
                from_address,
                recipient: parsed_recipient,
                icp_swap_request,
                from_minter,
            })
        } else if swap_steps == 3 && to_chain.is_evm() {
            // Handle EVM to EVM swap configuration
            let to_chain_id = match to_chain {
                Blockchain::Evm(chain_id) => chain_id,
                _ => return Err(SwapOrderCreationError::InvalidToChain),
            };
            let (to_minter, _) = read_state(|s| s.get_minter_by_chain_id(to_chain_id))
                .ok_or(SwapOrderCreationError::InvalidToChain)?;
            let icp_swap_request = create_validated_swap_args_from_rlp_swap_step(
                crosschain_quote.steps[1].clone(),
                amount_in,
                token_in_icp_step,
            )
            .map_err(SwapOrderCreationError::InvalidIcpSwapStep)?;
            Ok(CrosschainSwapOrder::EvmToEvm {
                tx_id: SwapTxId(tx_id.to_string()),
                from_address,
                recipient: parsed_recipient,
                icp_swap_request,
                evm_swap_step: crosschain_quote.steps[2].clone(),
                from_minter,
                to_minter,
            })
        } else {
            // Unsupported swap configuration
            Err(SwapOrderCreationError::InvalidOriginAndDestinationChain)
        }
    }
}
