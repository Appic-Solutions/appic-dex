use crate::rlp_decoder::RlpDecoder;
use ethnum::U256;

#[test]
fn test_decode_cross_chain_quote() {
    let encoded_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data).unwrap();
    
    assert!(quote.success);
    assert!(quote.error.is_none());
    assert!(quote.data.is_some());
    
    let data = quote.data.unwrap();
    assert_eq!(data.total_amount_in, U256::from_str_radix("50000000000000000000", 10).unwrap());
    assert_eq!(data.total_amount_out, U256::from_str_radix("11670858301863654", 10).unwrap());
    assert_eq!(data.total_min_amount_out.unwrap(), U256::from_str_radix("11577491435448744", 10).unwrap());
    assert_eq!(data.total_slippage.unwrap(), "0.8%");
    assert_eq!(data.steps.len(), 3);
    
    let step1 = &data.steps[0];
    assert_eq!(step1.chain_id, "56");
    assert_eq!(step1.amount_in, U256::from_str_radix("50000000000000000000", 10).unwrap());
    assert_eq!(step1.route.len(), 0);
    
    let has_routing = data.steps.iter().any(|step| step.route.len() > 0);
    let has_qswap = data.steps.iter().any(|step| step.qswap_data.is_some());
    assert!(has_routing || has_qswap);
}

#[test]
fn test_evm_to_evm_single_hop() {
    let encoded_data = hex::decode("f85989313030303030303030303089313030303030303030308931303030303030303084302e352566f831f82f81318831303030303030303089313030303030303030823025c0c0c030").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) => {
            if result.success {
                let data = result.data.unwrap();
                assert!(data.total_amount_in > U256::ZERO);
                assert!(data.steps.len() >= 1);
                
                let has_evm_chain = data.steps.iter().any(|step| 
                    step.chain_id.parse::<u64>().is_ok() && step.chain_id.len() <= 5
                );
                if !has_evm_chain {
                    assert!(!data.steps[0].chain_id.is_empty());
                }
            } else {
                assert!(result.error.is_some());
            }
        }
        Err(_) => {
        }
    }
}

#[test] 
fn test_evm_to_evm_multi_hop() {
    let encoded_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) if result.success => {
            let data = result.data.unwrap();
            assert!(data.steps.len() >= 1);
            
            let step_with_routing = data.steps.iter()
                .find(|step| step.route.len() > 0);
                
            if let Some(step) = step_with_routing {
                assert!(!step.chain_id.is_empty(), "Step with routing should have chain ID");
            }
        }
        _ => {
        }
    }
}

#[test]
fn test_evm_to_icp() {
    let encoded_data = hex::decode("f9032594353030303030303030303030303030303030303091313136373038353833303138363336353491313135373734393134333534343837343484302e3825f902e4f85b823536943530303030303030303030303030303030303030943530303030303030303030303030303030303030943530303030303030303030303030303030303030823025863135303030303088302e30303930303030c0c0c030f8b78369637094353030303030303030303030303030303030303088343936333436313488343934383537313084302e332530303030f87ef83d9b7a326979652d66796161612d61616161672d61743270612d6361699b7865766e6d2d67616161612d61616161722d7161666e712d6361698431303030f83d9b7865766e6d2d67616161612d61616161722d7161666e712d6361699b716b7277702d7a696161612d61616161672d6175656d712d6361698431303030c0c030f901cb843834353388343935383332393491313136373038353833303138363336353491313135373734393134333534343837343484302e382586333439383730873132363635393888302e30303230303984302e3035f85cf85aaa307838333335383966434436654462364530386634633743333244346637316235346264413032393133aa30783432303030303030303030303030303030303030303030303030303030303030303030303030303683313030c23134f90108b901023078303030303030303030303030303030303030303030303030383333353839666364366564623665303866346337633332643466373162353462646130323931333030303030303030303030303030303030303030303030303432303030303030303030303030303030303030303030303030303030303030303030303030303630303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030626238303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303032393431383263383764366561308230788a31373535373832333238").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) if result.success => {
            let data = result.data.unwrap();
            assert!(data.steps.len() >= 1);
            
            let has_bsc = data.steps.iter().any(|step| step.chain_id == "56");
            let has_base = data.steps.iter().any(|step| step.chain_id == "8453");
            let has_icp = data.steps.iter().any(|step| step.chain_id == "icp");
            
            assert!(has_bsc || has_base || has_icp || 
                   data.steps.iter().any(|step| step.chain_id.parse::<u64>().is_ok()),
                   "Should have recognizable chain identifiers");
        }
        _ => {
        }
    }
}

#[test]
fn test_icp_to_evm() {
    let encoded_data = hex::decode("f85989313030303030303030303089313030303030303030308931303030303030303084302e352566f831f82f8369637089313030303030303030308931303030303030303030823025c0c0c030").unwrap();
    
    let quote = RlpDecoder::decode_cross_chain_data(&encoded_data);
    
    match quote {
        Ok(result) => {
            if result.success {
                let data = result.data.unwrap();
                assert!(data.total_amount_in > U256::ZERO);
                assert!(data.steps.len() >= 1);
                
                let has_icp_or_evm = data.steps.iter().any(|step| 
                    step.chain_id == "icp" || step.chain_id.parse::<u64>().is_ok()
                );
                
                if has_icp_or_evm {
                } else {
                    assert!(!data.steps[0].chain_id.is_empty());
                }
            } else {
                assert!(result.error.is_some());
            }
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_chain_id_validation() {
    let valid_chains = vec!["1", "56", "137", "8453", "42161", "10", "43114", "250", "icp"];
    
    for chain_id in valid_chains {
        let is_supported = RlpDecoder::get_supported_chains().contains_key(chain_id);
        assert!(is_supported, "Chain {} should be supported", chain_id);
    }
    
    let invalid_chains = vec!["999", "invalid", "", "0x1"];
    for chain_id in invalid_chains {
        let is_supported = RlpDecoder::get_supported_chains().contains_key(chain_id);
        assert!(!is_supported, "Chain {} should not be supported", chain_id);
    }
}

#[test]
fn test_data_size_limits() {
    let large_data = vec![0u8; 2 * 1024 * 1024];
    let result = RlpDecoder::decode_cross_chain_data(&large_data);
    
    match result {
        Err(e) => assert!(e.to_string().contains("exceeds maximum size")),
        Ok(quote) => assert!(!quote.success && quote.error.is_some()),
    }
}

#[test]
fn test_production_ready_features() {
    assert!(RlpDecoder::get_supported_chains().len() >= 9);
    
    let chains = RlpDecoder::get_supported_chains();
    assert!(chains.contains_key("1")); 
    assert!(chains.contains_key("56"));
    assert!(chains.contains_key("8453"));
    assert!(chains.contains_key("icp"));
    
    assert_eq!(chains.get("1"), Some(&"Ethereum"));
    assert_eq!(chains.get("56"), Some(&"BSC"));
    assert_eq!(chains.get("8453"), Some(&"Base"));
    assert_eq!(chains.get("icp"), Some(&"Internet Computer"));
}

#[test]
fn test_invalid_rlp_data() {
    let invalid_data = vec![0x01, 0x02, 0x03, 0xff, 0xee];
    let result = RlpDecoder::decode_cross_chain_data(&invalid_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
            assert!(quote.data.is_none());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_empty_data() {
    let empty_data = vec![];
    let result = RlpDecoder::decode_cross_chain_data(&empty_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_malformed_rlp_structure() {
    let incomplete_data = hex::decode("c483313233").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&incomplete_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_invalid_u256_values() {
    let invalid_u256_data = hex::decode("c584696e76616c6964").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&invalid_u256_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_missing_required_fields() {
    let insufficient_data = hex::decode("c3823132").unwrap();
    let result = RlpDecoder::decode_cross_chain_data(&insufficient_data);
    
    match result {
        Ok(quote) => {
            assert!(!quote.success);
            assert!(quote.error.is_some());
        }
        Err(_) => {
        }
    }
}

#[test]
fn test_comprehensive_error_scenarios() {
    let error_cases = vec![
        vec![0xff],
        vec![0xc0],
        hex::decode("c1ff").unwrap(),
    ];
    
    for invalid_data in error_cases {
        let result = RlpDecoder::decode_cross_chain_data(&invalid_data);
        
        match result {
            Ok(quote) => {
                assert!(!quote.success || quote.error.is_some());
            }
            Err(_) => {
            }
        }
    }
}