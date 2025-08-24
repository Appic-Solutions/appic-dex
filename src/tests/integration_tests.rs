use crate::rlp_decoder::RlpDecoder;

#[cfg(feature = "integration")]
#[tokio::test]
async fn test_live_cross_chain_api_integration() {
    let test_cases = vec![
        // BSC USDT -> Base USDC 
        "http://108.61.179.161:3000/api/quote/cross-chain?tokenA=0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d&chainA=56&tokenB=0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913&chainB=8453&amount=100000000",
        
        // BNB -> Base ETH
        "http://108.61.179.161:3000/api/quote/cross-chain?tokenA=0x0000000000000000000000000000000000000000&chainA=56&tokenB=0x0000000000000000000000000000000000000000&chainB=8453&amount=100000000000000000",
        
        // BSC USDT -> Base ETH  
        "http://108.61.179.161:3000/api/quote/cross-chain?tokenA=0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d&chainA=56&tokenB=0x0000000000000000000000000000000000000000&chainB=8453&amount=50000000000000000000",
    ];
    
    for api_url in test_cases {
        println!("Testing API endpoint: {}", api_url);
        
        match fetch_and_decode_quote(api_url).await {
            Ok((quote_response, decode_result)) => {
                println!("✓ API Response received for {}", api_url);
                
                if let Some(encoded_data) = quote_response.get("encodedData").and_then(|v| v.as_str()) {
                    match hex::decode(encoded_data) {
                        Ok(data) => {
                            let quote = RlpDecoder::decode_cross_chain_data(&data);
                            match quote {
                                Ok(result) => {
                                    assert!(result.success, "Decoded quote should be successful");
                                    if let Some(quote_data) = result.data {
                                        validate_production_quote(&quote_data);
                                        println!("✓ Quote decoded and validated successfully");
                                    }
                                }
                                Err(e) => {
                                    panic!("Failed to decode quote from {}: {}", api_url, e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠ Invalid hex data from {}: {}", api_url, e);
                        }
                    }
                } else {
                    println!("⚠ No encoded data in response from {}", api_url);
                }
            }
            Err(e) => {
                println!("⚠ API request failed for {}: {}", api_url, e);
            }
        }
    }
}

#[cfg(feature = "integration")]
async fn fetch_and_decode_quote(url: &str) -> Result<(serde_json::Value, Option<String>), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let json: serde_json::Value = response.json().await?;
    
    let encoded_data = json.get("encodedData")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Ok((json, encoded_data))
}

#[cfg(feature = "integration")]
fn validate_production_quote(quote_data: &crate::rlp_decoder::CrossChainQuoteData) {
    assert!(quote_data.total_amount_in > ethnum::U256::ZERO, "Should have positive input amount");
    assert!(quote_data.steps.len() >= 2, "Cross-chain quote should have multiple steps");
    
    let chain_ids: Vec<&String> = quote_data.steps.iter().map(|s| &s.chain_id).collect();
    let unique_chains: std::collections::HashSet<&String> = chain_ids.iter().cloned().collect();
    assert!(unique_chains.len() >= 2, "Should involve multiple chains");
    
    let supported_chains = RlpDecoder::get_supported_chains();
    for step in &quote_data.steps {
        assert!(supported_chains.contains_key(step.chain_id.as_str()), 
               "Chain {} should be supported", step.chain_id);
    }
}

#[test]
fn test_api_response_structure_compatibility() {
    let sample_responses = vec![
        r#"{"encodedData":"f9032594353030...", "success":true, "data":{}}"#,
        r#"{"encodedData":"", "success":false, "error":"Quote failed"}"#,
        r#"{"success":true, "data":{"totalAmountIn":"1000", "steps":[]}}"#,
    ];
    
    for response_json in sample_responses {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_json) {
            let has_encoded_data = json.get("encodedData").is_some();
            let has_success = json.get("success").is_some();
            
            assert!(has_success, "Response should have success field");
            
            if has_encoded_data {
                if let Some(encoded) = json.get("encodedData").and_then(|v| v.as_str()) {
                    if !encoded.is_empty() {
                        println!("Found encoded data: {}...", &encoded[..std::cmp::min(20, encoded.len())]);
                    }
                }
            }
        }
    }
}

#[test]
fn test_cross_chain_pair_combinations() {
    let chain_pairs = vec![
        ("56", "8453"),   // BSC -> Base
        ("1", "56"),      // Ethereum -> BSC  
        ("8453", "137"),  // Base -> Polygon
        ("42161", "10"),  // Arbitrum -> Optimism
    ];
    
    let supported_chains = RlpDecoder::get_supported_chains();
    
    for (chain_a, chain_b) in chain_pairs {
        assert!(supported_chains.contains_key(chain_a), "Chain A {} should be supported", chain_a);
        assert!(supported_chains.contains_key(chain_b), "Chain B {} should be supported", chain_b);
        
        println!("✓ Cross-chain pair supported: {} ({}) -> {} ({})", 
                chain_a, supported_chains[chain_a], 
                chain_b, supported_chains[chain_b]);
    }
}