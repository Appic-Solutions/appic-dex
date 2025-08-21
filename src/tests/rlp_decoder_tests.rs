use crate::rlp_decoder::{RlpDecoder, CrossChainQuote, CrossChainStep, Hop};

#[test]
fn test_decode_simple_cross_chain_quote() {
    let encoded_data = hex::decode("f9011689313030303030303030303084302e3325f90102f83a823536893130303030303030308931303030303030303089313030303030303030823025863135303030303088302e30303930303030c0c0c030f89c8369637089313030303030303030303084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030e7843834353330303084302e3325863135303030303088302e30303034353284302e3035c0c0c030").unwrap();
    
    let result = RlpDecoder::decode_cross_chain_data(&encoded_data);
    assert!(result.is_ok());
    
    let quote = result.unwrap();
    assert_eq!(quote.total_amount_in, "100000000");
    assert_eq!(quote.total_amount_out, "0");
    assert_eq!(quote.total_slippage, "0.3%");
    assert_eq!(quote.steps.len(), 3);
    
    assert_eq!(quote.steps[0].chain_id, "56");
    assert_eq!(quote.steps[0].amount_in, "100000000");
    assert_eq!(quote.steps[0].gas_limit, "150000");
    
    assert_eq!(quote.steps[1].chain_id, "icp");
    assert_eq!(quote.steps[1].hops.len(), 2);
    
    assert_eq!(quote.steps[2].chain_id, "8453");
    assert_eq!(quote.steps[2].canister_fee_usd, "0.05");
}


#[test]
fn test_encode_decode_roundtrip() {
    let original_quote = CrossChainQuote {
        total_amount_in: "1000000".to_string(),
        total_amount_out: "950000".to_string(),
        total_min_amount_out: "940000".to_string(),
        total_slippage: "1.0%".to_string(),
        steps: vec![
            CrossChainStep {
                chain_id: "1".to_string(),
                amount_in: "1000000".to_string(),
                amount_out: "980000".to_string(),
                min_amount_out: "970000".to_string(),
                slippage: "0.5%".to_string(),
                gas_limit: "200000".to_string(),
                max_gas_fee: "50000000000".to_string(),
                gas_price_usd: "10.50".to_string(),
                canister_fee_usd: "0.01".to_string(),
                hops: vec![
                    Hop {
                        sell_token: b"token1".to_vec(),
                        buy_token: b"token2".to_vec(),
                        fee: "3000".to_string(),
                    }
                ],
                qswap_commands: vec!["1".to_string(), "2".to_string()],
                qswap_command_data: vec![b"data1".to_vec(), b"data2".to_vec()],
                qswap_deadline: "1700000000".to_string(),
            }
        ],
    };
    
    let encoded = RlpDecoder::encode_cross_chain_data(&original_quote);
    let decoded = RlpDecoder::decode_cross_chain_data(&encoded).unwrap();
    
    assert_eq!(original_quote, decoded);
}

#[test]
fn test_empty_steps() {
    let quote = CrossChainQuote {
        total_amount_in: "1000000".to_string(),
        total_amount_out: "950000".to_string(),
        total_min_amount_out: "940000".to_string(),
        total_slippage: "1.0%".to_string(),
        steps: vec![],
    };
    
    let encoded = RlpDecoder::encode_cross_chain_data(&quote);
    let decoded = RlpDecoder::decode_cross_chain_data(&encoded).unwrap();
    
    assert_eq!(quote, decoded);
    assert_eq!(decoded.steps.len(), 0);
}

#[test]
fn test_step_with_no_hops() {
    let quote = CrossChainQuote {
        total_amount_in: "1000000".to_string(),
        total_amount_out: "950000".to_string(),
        total_min_amount_out: "940000".to_string(),
        total_slippage: "1.0%".to_string(),
        steps: vec![
            CrossChainStep {
                chain_id: "1".to_string(),
                amount_in: "1000000".to_string(),
                amount_out: "1000000".to_string(),
                min_amount_out: "1000000".to_string(),
                slippage: "0%".to_string(),
                gas_limit: "21000".to_string(),
                max_gas_fee: "20000000000".to_string(),
                gas_price_usd: "1.00".to_string(),
                canister_fee_usd: "0".to_string(),
                hops: vec![],
                qswap_commands: vec![],
                qswap_command_data: vec![],
                qswap_deadline: "0".to_string(),
            }
        ],
    };
    
    let encoded = RlpDecoder::encode_cross_chain_data(&quote);
    let decoded = RlpDecoder::decode_cross_chain_data(&encoded).unwrap();
    
    assert_eq!(quote, decoded);
    assert_eq!(decoded.steps[0].hops.len(), 0);
}

#[test]
fn test_invalid_rlp_data() {
    let invalid_data = vec![0x01, 0x02, 0x03];
    let result = RlpDecoder::decode_cross_chain_data(&invalid_data);
    assert!(result.is_err());
}

#[test]
fn test_missing_fields() {
    let incomplete_data = hex::decode("c483313233").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&incomplete_data);
    assert!(result.is_err());
}

#[test]
fn test_multiple_hops() {
    let quote = CrossChainQuote {
        total_amount_in: "1000000".to_string(),
        total_amount_out: "900000".to_string(),
        total_min_amount_out: "890000".to_string(),
        total_slippage: "1.1%".to_string(),
        steps: vec![
            CrossChainStep {
                chain_id: "1".to_string(),
                amount_in: "1000000".to_string(),
                amount_out: "950000".to_string(),
                min_amount_out: "940000".to_string(),
                slippage: "0.5%".to_string(),
                gas_limit: "300000".to_string(),
                max_gas_fee: "100000000000".to_string(),
                gas_price_usd: "15.75".to_string(),
                canister_fee_usd: "0.02".to_string(),
                hops: vec![
                    Hop {
                        sell_token: b"USDC".to_vec(),
                        buy_token: b"WETH".to_vec(),
                        fee: "500".to_string(),
                    },
                    Hop {
                        sell_token: b"WETH".to_vec(),
                        buy_token: b"DAI".to_vec(),
                        fee: "3000".to_string(),
                    },
                    Hop {
                        sell_token: b"DAI".to_vec(),
                        buy_token: b"USDC".to_vec(),
                        fee: "500".to_string(),
                    }
                ],
                qswap_commands: vec!["1".to_string(), "2".to_string(), "3".to_string()],
                qswap_command_data: vec![
                    b"command1data".to_vec(),
                    b"command2data".to_vec(),
                    b"command3data".to_vec()
                ],
                qswap_deadline: "1700000000".to_string(),
            }
        ],
    };
    
    let encoded = RlpDecoder::encode_cross_chain_data(&quote);
    let decoded = RlpDecoder::decode_cross_chain_data(&encoded).unwrap();
    
    assert_eq!(quote, decoded);
    assert_eq!(decoded.steps[0].hops.len(), 3);
    assert_eq!(decoded.steps[0].qswap_commands.len(), 3);
    assert_eq!(decoded.steps[0].qswap_command_data.len(), 3);
}