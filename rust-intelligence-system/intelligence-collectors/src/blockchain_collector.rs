use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use reqwest::Client;
use intelligence_core::{
    IntelligenceData, DataSource, DataClassification, IntelligenceId, Result as IntelligenceResult
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub ethereum_rpc_url: String,
    pub bitcoin_rpc_url: String,
    pub polygon_rpc_url: String,
    pub bsc_rpc_url: String,
    pub avalanche_rpc_url: String,
    pub max_concurrent_requests: usize,
    pub rate_limit_per_second: u32,
    pub batch_size: usize,
    pub collection_timeout_seconds: u64,
    pub api_keys: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainTransaction {
    pub hash: String,
    pub block_number: u64,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub gas_used: u64,
    pub gas_price: String,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub input_data: Option<String>,
    pub contract_address: Option<String>,
    pub token_transfers: Vec<TokenTransfer>,
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub token_address: String,
    pub token_symbol: String,
    pub token_name: String,
    pub from_address: String,
    pub to_address: String,
    pub value: String,
    pub decimals: u8,
    pub transfer_type: String, // "ERC20", "ERC721", "ERC1155"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub balance: String,
    pub token_balances: Vec<TokenBalance>,
    pub transaction_count: u64,
    pub first_transaction: Option<DateTime<Utc>>,
    pub last_transaction: Option<DateTime<Utc>>,
    pub is_contract: bool,
    pub contract_type: Option<String>,
    pub risk_score: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token_address: String,
    pub token_symbol: String,
    pub balance: String,
    pub usd_value: Option<f64>,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub transactions_collected: u64,
    pub wallets_analyzed: u64,
    pub tokens_tracked: u64,
    pub errors_count: u64,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub throughput_per_second: f64,
}

pub struct BlockchainCollector {
    config: BlockchainConfig,
    http_client: Client,
    stats: Arc<RwLock<CollectionStats>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    transaction_buffer: Arc<RwLock<Vec<BlockchainTransaction>>>,
    wallet_cache: Arc<RwLock<HashMap<String, WalletInfo>>>,
    is_running: Arc<RwLock<bool>>,
    processors: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

struct RateLimiter {
    last_request: std::time::Instant,
    requests_per_second: u32,
}

impl BlockchainCollector {
    pub fn new(config: BlockchainConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            stats: Arc::new(RwLock::new(CollectionStats {
                transactions_collected: 0,
                wallets_analyzed: 0,
                tokens_tracked: 0,
                errors_count: 0,
                start_time: Utc::now(),
                last_update: Utc::now(),
                throughput_per_second: 0.0,
            })),
            rate_limiter: Arc::new(RwLock::new(RateLimiter {
                last_request: std::time::Instant::now(),
                requests_per_second: 10,
            })),
            transaction_buffer: Arc::new(RwLock::new(Vec::new())),
            wallet_cache: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            processors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_collection(&self, target_addresses: Vec<String>) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);
        
        info!("Starting blockchain data collection for {} addresses", target_addresses.len());
        
        let mut tasks = Vec::new();
        
        // Start transaction monitoring
        for address in target_addresses.clone() {
            let collector = self.clone_for_task();
            let task = tokio::spawn(async move {
                collector.monitor_address_transactions(&address).await
            });
            tasks.push(task);
        }
        
        // Start wallet analysis
        let collector = self.clone_for_task();
        let wallet_task = tokio::spawn(async move {
            collector.analyze_wallets().await
        });
        tasks.push(wallet_task);
        
        // Start batch processing
        let collector = self.clone_for_task();
        let batch_task = tokio::spawn(async move {
            collector.process_batches().await
        });
        tasks.push(batch_task);
        
        {
            let mut processors = self.processors.write().await;
            processors.extend(tasks);
        }
        
        info!("Blockchain collector started with {} concurrent tasks", tasks.len());
        Ok(())
    }

    pub async fn stop_collection(&self) -> IntelligenceResult<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        let mut processors = self.processors.write().await;
        for processor in processors.drain(..) {
            processor.abort();
        }
        
        info!("Blockchain collection stopped");
        Ok(())
    }

    async fn monitor_address_transactions(&self, address: &str) -> IntelligenceResult<()> {
        let mut last_block = 0u64;
        
        while *self.is_running.read().await {
            match self.fetch_transactions_for_address(address, last_block).await {
                Ok(transactions) => {
                    if !transactions.is_empty() {
                        {
                            let mut buffer = self.transaction_buffer.write().await;
                            buffer.extend(transactions.clone());
                        }
                        
                        {
                            let mut stats = self.stats.write().await;
                            stats.transactions_collected += transactions.len() as u64;
                            stats.last_update = Utc::now();
                        }
                        
                        if let Some(last_tx) = transactions.last() {
                            last_block = last_tx.block_number;
                        }
                        
                        debug!("Collected {} transactions for address {}", transactions.len(), address);
                    }
                    
                    self.enforce_rate_limit().await;
                }
                Err(e) => {
                    error!("Failed to fetch transactions for address {}: {}", address, e);
                    {
                        let mut stats = self.stats.write().await;
                        stats.errors_count += 1;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
        Ok(())
    }

    async fn fetch_transactions_for_address(&self, address: &str, from_block: u64) -> IntelligenceResult<Vec<BlockchainTransaction>> {
        // Fetch from Ethereum first
        let mut all_transactions = Vec::new();
        
        // Ethereum transactions
        if let Ok(eth_txs) = self.fetch_ethereum_transactions(address, from_block).await {
            all_transactions.extend(eth_txs);
        }
        
        // Bitcoin transactions (if address is Bitcoin)
        if address.starts_with("1") || address.starts_with("3") || address.starts_with("bc1") {
            if let Ok(btc_txs) = self.fetch_bitcoin_transactions(address).await {
                all_transactions.extend(btc_txs);
            }
        }
        
        Ok(all_transactions)
    }

    async fn fetch_ethereum_transactions(&self, address: &str, from_block: u64) -> IntelligenceResult<Vec<BlockchainTransaction>> {
        let url = &self.config.ethereum_rpc_url;
        
        // Get latest block number
        let latest_block = self.get_latest_block_number(url).await?;
        let to_block = (from_block + 1000).min(latest_block);
        
        if from_block >= latest_block {
            return Ok(Vec::new());
        }
        
        // Fetch transactions using Etherscan API or direct RPC
        let mut transactions = Vec::new();
        
        // Use Etherscan API if available
        if let Some(api_key) = self.config.api_keys.get("etherscan") {
            let etherscan_txs = self.fetch_etherscan_transactions(address, from_block, to_block, api_key).await?;
            transactions.extend(etherscan_txs);
        } else {
            // Fallback to direct RPC calls
            let rpc_txs = self.fetch_rpc_transactions(url, address, from_block, to_block).await?;
            transactions.extend(rpc_txs);
        }
        
        Ok(transactions)
    }

    async fn fetch_etherscan_transactions(&self, address: &str, from_block: u64, to_block: u64, api_key: &str) -> IntelligenceResult<Vec<BlockchainTransaction>> {
        let url = format!(
            "https://api.etherscan.io/api?module=account&action=txlist&address={}&startblock={}&endblock={}&sort=asc&apikey={}",
            address, from_block, to_block, api_key
        );
        
        let response = self.http_client.get(&url).send().await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            
            if let Some(result) = data.get("result").and_then(|r| r.as_array()) {
                let mut transactions = Vec::new();
                
                for tx_data in result {
                    if let Ok(transaction) = self.parse_ethereum_transaction(tx_data) {
                        transactions.push(transaction);
                    }
                }
                
                Ok(transactions)
            } else {
                Ok(Vec::new())
            }
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "etherscan".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn fetch_rpc_transactions(&self, rpc_url: &str, address: &str, from_block: u64, to_block: u64) -> IntelligenceResult<Vec<BlockchainTransaction>> {
        let mut transactions = Vec::new();
        
        for block_num in from_block..=to_block {
            let block_txs = self.get_block_transactions(rpc_url, block_num, address).await?;
            transactions.extend(block_txs);
            
            self.enforce_rate_limit().await;
        }
        
        Ok(transactions)
    }

    async fn get_latest_block_number(&self, rpc_url: &str) -> IntelligenceResult<u64> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });
        
        let response = self.http_client
            .post(rpc_url)
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            if let Some(result) = data.get("result") {
                let hex_str = result.as_str().unwrap_or("0x0");
                let block_num = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)?;
                Ok(block_num)
            } else {
                Err(intelligence_core::IntelligenceError::ExternalService {
                    service: "ethereum_rpc".to_string(),
                    message: "No result in response".to_string(),
                })
            }
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "ethereum_rpc".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn get_block_transactions(&self, rpc_url: &str, block_number: u64, target_address: &str) -> IntelligenceResult<Vec<BlockchainTransaction>> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [format!("0x{:x}", block_number), true],
            "id": 1
        });
        
        let response = self.http_client
            .post(rpc_url)
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            let mut transactions = Vec::new();
            
            if let Some(block) = data.get("result") {
                if let Some(txs) = block.get("transactions").and_then(|t| t.as_array()) {
                    for tx_data in txs {
                        if let Some(from) = tx_data.get("from").and_then(|f| f.as_str()) {
                            if let Some(to) = tx_data.get("to").and_then(|t| t.as_str()) {
                                if from.to_lowercase() == target_address.to_lowercase() || 
                                   to.to_lowercase() == target_address.to_lowercase() {
                                    if let Ok(transaction) = self.parse_ethereum_transaction(tx_data) {
                                        transactions.push(transaction);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(transactions)
        } else {
            Err(intelligence_core::IntelligenceError::ExternalService {
                service: "ethereum_rpc".to_string(),
                message: format!("HTTP error: {}", response.status()),
            })
        }
    }

    async fn parse_ethereum_transaction(&self, tx_data: &serde_json::Value) -> IntelligenceResult<BlockchainTransaction> {
        let hash = tx_data.get("hash").and_then(|h| h.as_str()).unwrap_or("").to_string();
        let block_number = tx_data.get("blockNumber")
            .and_then(|b| b.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(0);
        let from = tx_data.get("from").and_then(|f| f.as_str()).unwrap_or("").to_string();
        let to = tx_data.get("to").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let value = tx_data.get("value").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let gas_used = tx_data.get("gasUsed")
            .and_then(|g| g.as_str())
            .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
            .unwrap_or(0);
        let gas_price = tx_data.get("gasPrice").and_then(|g| g.as_str()).unwrap_or("0").to_string();
        let status = tx_data.get("status").and_then(|s| s.as_str()).unwrap_or("0").to_string();
        let input_data = tx_data.get("input").and_then(|i| i.as_str()).map(|s| s.to_string());
        let contract_address = tx_data.get("contractAddress").and_then(|c| c.as_str()).map(|s| s.to_string());
        
        // Parse timestamp
        let timestamp = if let Some(timestamp_hex) = tx_data.get("timeStamp").and_then(|t| t.as_str()) {
            if let Ok(timestamp_num) = u64::from_str_radix(timestamp_hex.trim_start_matches("0x"), 16) {
                DateTime::from_timestamp(timestamp_num as i64, 0).unwrap_or_else(Utc::now)
            } else {
                Utc::now()
            }
        } else {
            Utc::now()
        };
        
        // Parse token transfers if available
        let token_transfers = if let Some(transfers) = tx_data.get("tokenTransfers").and_then(|t| t.as_array()) {
            let mut parsed_transfers = Vec::new();
            for transfer_data in transfers {
                if let Ok(transfer) = self.parse_token_transfer(transfer_data) {
                    parsed_transfers.push(transfer);
                }
            }
            parsed_transfers
        } else {
            Vec::new()
        };
        
        Ok(BlockchainTransaction {
            hash,
            block_number,
            from_address: from,
            to_address: to,
            value,
            gas_used,
            gas_price,
            timestamp,
            status,
            input_data,
            contract_address,
            token_transfers,
            network: "ethereum".to_string(),
        })
    }

    async fn parse_token_transfer(&self, transfer_data: &serde_json::Value) -> IntelligenceResult<TokenTransfer> {
        let token_address = transfer_data.get("contractAddress").and_then(|a| a.as_str()).unwrap_or("").to_string();
        let token_symbol = transfer_data.get("tokenSymbol").and_then(|s| s.as_str()).unwrap_or("").to_string();
        let token_name = transfer_data.get("tokenName").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let from_address = transfer_data.get("from").and_then(|f| f.as_str()).unwrap_or("").to_string();
        let to_address = transfer_data.get("to").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let value = transfer_data.get("value").and_then(|v| v.as_str()).unwrap_or("0").to_string();
        let decimals = transfer_data.get("tokenDecimal").and_then(|d| d.as_str()).and_then(|s| s.parse().ok()).unwrap_or(18);
        let transfer_type = transfer_data.get("transferType").and_then(|t| t.as_str()).unwrap_or("ERC20").to_string();
        
        Ok(TokenTransfer {
            token_address,
            token_symbol,
            token_name,
            from_address,
            to_address,
            value,
            decimals,
            transfer_type,
        })
    }

    async fn fetch_bitcoin_transactions(&self, address: &str) -> IntelligenceResult<Vec<BlockchainTransaction>> {
        // Use BlockCypher API or similar for Bitcoin transactions
        if let Some(api_key) = self.config.api_keys.get("blockcypher") {
            let url = format!("https://api.blockcypher.com/v1/btc/main/addrs/{}/txs?token={}", address, api_key);
            
            let response = self.http_client.get(&url).send().await?;
            
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let mut transactions = Vec::new();
                
                if let Some(txs) = data.get("txs").and_then(|t| t.as_array()) {
                    for tx_data in txs {
                        if let Ok(transaction) = self.parse_bitcoin_transaction(tx_data) {
                            transactions.push(transaction);
                        }
                    }
                }
                
                Ok(transactions)
            } else {
                Err(intelligence_core::IntelligenceError::ExternalService {
                    service: "blockcypher".to_string(),
                    message: format!("HTTP error: {}", response.status()),
                })
            }
        } else {
            // Fallback to mock data for Bitcoin
            Ok(self.generate_mock_bitcoin_transactions(address).await)
        }
    }

    async fn parse_bitcoin_transaction(&self, tx_data: &serde_json::Value) -> IntelligenceResult<BlockchainTransaction> {
        let hash = tx_data.get("hash").and_then(|h| h.as_str()).unwrap_or("").to_string();
        let block_height = tx_data.get("block_height").and_then(|b| b.as_u64()).unwrap_or(0);
        let timestamp = if let Some(time) = tx_data.get("confirmed").and_then(|t| t.as_str()) {
            DateTime::parse_from_rfc3339(time).unwrap_or_else(|_| Utc::now()).with_timezone(&Utc)
        } else {
            Utc::now()
        };
        
        // Parse inputs and outputs to determine from/to addresses
        let mut from_addresses = Vec::new();
        let mut to_addresses = Vec::new();
        let mut total_value = 0u64;
        
        if let Some(inputs) = tx_data.get("inputs").and_then(|i| i.as_array()) {
            for input in inputs {
                if let Some(addresses) = input.get("addresses").and_then(|a| a.as_array()) {
                    for addr in addresses {
                        if let Some(addr_str) = addr.as_str() {
                            from_addresses.push(addr_str.to_string());
                        }
                    }
                }
            }
        }
        
        if let Some(outputs) = tx_data.get("outputs").and_then(|o| o.as_array()) {
            for output in outputs {
                if let Some(addresses) = output.get("addresses").and_then(|a| a.as_array()) {
                    for addr in addresses {
                        if let Some(addr_str) = addr.as_str() {
                            to_addresses.push(addr_str.to_string());
                        }
                    }
                }
                if let Some(value) = output.get("value").and_then(|v| v.as_u64()) {
                    total_value += value;
                }
            }
        }
        
        Ok(BlockchainTransaction {
            hash,
            block_number: block_height,
            from_address: from_addresses.join(","),
            to_address: to_addresses.join(","),
            value: total_value.to_string(),
            gas_used: 0, // Bitcoin doesn't use gas
            gas_price: "0".to_string(),
            timestamp,
            status: "confirmed".to_string(),
            input_data: None,
            contract_address: None,
            token_transfers: Vec::new(),
            network: "bitcoin".to_string(),
        })
    }

    async fn generate_mock_bitcoin_transactions(&self, address: &str) -> Vec<BlockchainTransaction> {
        let mut transactions = Vec::new();
        
        for i in 0..5 {
            let transaction = BlockchainTransaction {
                hash: format!("btc_tx_{}_{}", address, i),
                block_number: 800000 + i,
                from_address: format!("btc_from_{}", i),
                to_address: address.to_string(),
                value: format!("{}", 1000000 + i * 100000), // satoshis
                gas_used: 0,
                gas_price: "0".to_string(),
                timestamp: Utc::now() - chrono::Duration::hours(i as i64),
                status: "confirmed".to_string(),
                input_data: None,
                contract_address: None,
                token_transfers: Vec::new(),
                network: "bitcoin".to_string(),
            };
            transactions.push(transaction);
        }
        
        transactions
    }

    async fn analyze_wallets(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let addresses = {
                let buffer = self.transaction_buffer.read().await;
                let mut addresses = std::collections::HashSet::new();
                for tx in buffer.iter() {
                    addresses.insert(tx.from_address.clone());
                    addresses.insert(tx.to_address.clone());
                }
                addresses.into_iter().collect::<Vec<_>>()
            };
            
            for address in addresses {
                if self.wallet_cache.read().await.contains_key(&address) {
                    continue;
                }
                
                match self.analyze_wallet(&address).await {
                    Ok(wallet_info) => {
                        self.wallet_cache.write().await.insert(address.clone(), wallet_info);
                        {
                            let mut stats = self.stats.write().await;
                            stats.wallets_analyzed += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to analyze wallet {}: {}", address, e);
                    }
                }
                
                self.enforce_rate_limit().await;
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
        Ok(())
    }

    async fn analyze_wallet(&self, address: &str) -> IntelligenceResult<WalletInfo> {
        // Get wallet balance
        let balance = self.get_wallet_balance(address).await?;
        
        // Get token balances
        let token_balances = self.get_token_balances(address).await?;
        
        // Get transaction count
        let transaction_count = self.get_transaction_count(address).await?;
        
        // Calculate risk score
        let risk_score = self.calculate_wallet_risk_score(address, &token_balances).await?;
        
        // Determine if it's a contract
        let is_contract = self.is_contract_address(address).await?;
        
        // Get tags
        let tags = self.get_wallet_tags(address).await?;
        
        Ok(WalletInfo {
            address: address.to_string(),
            balance,
            token_balances,
            transaction_count,
            first_transaction: None, // Would need to fetch from transaction history
            last_transaction: None,  // Would need to fetch from transaction history
            is_contract,
            contract_type: if is_contract { Some("unknown".to_string()) } else { None },
            risk_score,
            tags,
        })
    }

    async fn get_wallet_balance(&self, address: &str) -> IntelligenceResult<String> {
        // Use Etherscan API or direct RPC
        if let Some(api_key) = self.config.api_keys.get("etherscan") {
            let url = format!(
                "https://api.etherscan.io/api?module=account&action=balance&address={}&tag=latest&apikey={}",
                address, api_key
            );
            
            let response = self.http_client.get(&url).send().await?;
            
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                if let Some(result) = data.get("result").and_then(|r| r.as_str()) {
                    Ok(result.to_string())
                } else {
                    Ok("0".to_string())
                }
            } else {
                Ok("0".to_string())
            }
        } else {
            // Fallback to mock balance
            Ok("1000000000000000000".to_string()) // 1 ETH in wei
        }
    }

    async fn get_token_balances(&self, address: &str) -> IntelligenceResult<Vec<TokenBalance>> {
        // Use Etherscan API for token balances
        if let Some(api_key) = self.config.api_keys.get("etherscan") {
            let url = format!(
                "https://api.etherscan.io/api?module=account&action=tokentx&address={}&startblock=0&endblock=999999999&sort=asc&apikey={}",
                address, api_key
            );
            
            let response = self.http_client.get(&url).send().await?;
            
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                let mut token_balances = Vec::new();
                
                if let Some(result) = data.get("result").and_then(|r| r.as_array()) {
                    let mut token_map: HashMap<String, TokenBalance> = HashMap::new();
                    
                    for tx in result {
                        if let Some(token_address) = tx.get("contractAddress").and_then(|a| a.as_str()) {
                            let token_symbol = tx.get("tokenSymbol").and_then(|s| s.as_str()).unwrap_or("UNKNOWN");
                            let token_name = tx.get("tokenName").and_then(|n| n.as_str()).unwrap_or("Unknown Token");
                            let decimals = tx.get("tokenDecimal").and_then(|d| d.as_str()).and_then(|s| s.parse().ok()).unwrap_or(18);
                            
                            // This is a simplified approach - in reality you'd need to call the token contract
                            // to get the actual balance
                            token_map.insert(token_address.to_string(), TokenBalance {
                                token_address: token_address.to_string(),
                                token_symbol: token_symbol.to_string(),
                                balance: "0".to_string(), // Would need to call contract
                                usd_value: None,
                                decimals,
                            });
                        }
                    }
                    
                    token_balances.extend(token_map.into_values());
                }
                
                Ok(token_balances)
            } else {
                Ok(Vec::new())
            }
        } else {
            // Fallback to mock token balances
            Ok(vec![
                TokenBalance {
                    token_address: "0xA0b86a33E6441b8c4C8C0C4C0C4C0C4C0C4C0C4C".to_string(),
                    token_symbol: "USDC".to_string(),
                    balance: "1000000000".to_string(), // 1000 USDC
                    usd_value: Some(1000.0),
                    decimals: 6,
                }
            ])
        }
    }

    async fn get_transaction_count(&self, address: &str) -> IntelligenceResult<u64> {
        // Use Etherscan API
        if let Some(api_key) = self.config.api_keys.get("etherscan") {
            let url = format!(
                "https://api.etherscan.io/api?module=proxy&action=eth_getTransactionCount&address={}&tag=latest&apikey={}",
                address, api_key
            );
            
            let response = self.http_client.get(&url).send().await?;
            
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                if let Some(result) = data.get("result").and_then(|r| r.as_str()) {
                    let count = u64::from_str_radix(result.trim_start_matches("0x"), 16).unwrap_or(0);
                    Ok(count)
                } else {
                    Ok(0)
                }
            } else {
                Ok(0)
            }
        } else {
            // Fallback to mock count
            Ok(42)
        }
    }

    async fn calculate_wallet_risk_score(&self, address: &str, token_balances: &[TokenBalance]) -> IntelligenceResult<f64> {
        let mut risk_score = 0.0;
        
        // Check if address is in known risk databases
        if self.is_high_risk_address(address).await? {
            risk_score += 0.5;
        }
        
        // Check token diversity
        if token_balances.len() > 10 {
            risk_score += 0.2; // High token diversity might indicate mixing
        }
        
        // Check for suspicious tokens
        for token in token_balances {
            if self.is_suspicious_token(&token.token_address).await? {
                risk_score += 0.3;
            }
        }
        
        Ok(risk_score.min(1.0))
    }

    async fn is_high_risk_address(&self, address: &str) -> IntelligenceResult<bool> {
        // In a real implementation, this would check against known risk databases
        // For now, return false
        Ok(false)
    }

    async fn is_suspicious_token(&self, token_address: &str) -> IntelligenceResult<bool> {
        // In a real implementation, this would check against known suspicious tokens
        // For now, return false
        Ok(false)
    }

    async fn is_contract_address(&self, address: &str) -> IntelligenceResult<bool> {
        // Use Etherscan API to check if address is a contract
        if let Some(api_key) = self.config.api_keys.get("etherscan") {
            let url = format!(
                "https://api.etherscan.io/api?module=proxy&action=eth_getCode&address={}&tag=latest&apikey={}",
                address, api_key
            );
            
            let response = self.http_client.get(&url).send().await?;
            
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await?;
                if let Some(result) = data.get("result").and_then(|r| r.as_str()) {
                    // If result is not "0x", it's a contract
                    Ok(result != "0x")
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        } else {
            // Fallback - assume it's not a contract
            Ok(false)
        }
    }

    async fn get_wallet_tags(&self, address: &str) -> IntelligenceResult<Vec<String>> {
        // In a real implementation, this would check against known address tags
        // For now, return empty vector
        Ok(Vec::new())
    }

    async fn process_batches(&self) -> IntelligenceResult<()> {
        while *self.is_running.read().await {
            let batch = {
                let mut buffer = self.transaction_buffer.write().await;
                if buffer.len() >= self.config.batch_size {
                    let batch = buffer.drain(0..self.config.batch_size).collect::<Vec<_>>();
                    batch
                } else {
                    Vec::new()
                }
            };
            
            if !batch.is_empty() {
                let intelligence_data = self.convert_transactions_to_intelligence_data(batch).await?;
                info!("Processed batch of {} transactions into {} intelligence data points",
                      batch.len(), intelligence_data.len());
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        Ok(())
    }

    async fn convert_transactions_to_intelligence_data(
        &self,
        transactions: Vec<BlockchainTransaction>,
    ) -> IntelligenceResult<Vec<IntelligenceData>> {
        let mut intelligence_data = Vec::new();
        
        for transaction in transactions {
            let content = serde_json::json!({
                "hash": transaction.hash,
                "block_number": transaction.block_number,
                "from_address": transaction.from_address,
                "to_address": transaction.to_address,
                "value": transaction.value,
                "gas_used": transaction.gas_used,
                "gas_price": transaction.gas_price,
                "timestamp": transaction.timestamp,
                "status": transaction.status,
                "network": transaction.network,
                "token_transfers": transaction.token_transfers,
                "contract_address": transaction.contract_address,
                "input_data": transaction.input_data
            });
            
            let data = IntelligenceData {
                id: IntelligenceId::new(),
                content: content.to_string(),
                source: DataSource::Blockchain,
                classification: DataClassification::Internal,
                metadata: HashMap::new(),
                timestamp: transaction.timestamp,
                confidence: 1.0,
                quality_score: 0.9,
            };
            
            intelligence_data.push(data);
        }
        
        Ok(intelligence_data)
    }

    async fn enforce_rate_limit(&self) {
        let mut limiter = self.rate_limiter.write().await;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(limiter.last_request);
        let min_interval = std::time::Duration::from_millis(1000 / limiter.requests_per_second as u64);
        
        if elapsed < min_interval {
            tokio::time::sleep(min_interval - elapsed).await;
        }
        
        limiter.last_request = std::time::Instant::now();
    }

    pub async fn get_stats(&self) -> CollectionStats {
        self.stats.read().await.clone()
    }

    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            stats: Arc::clone(&self.stats),
            rate_limiter: Arc::clone(&self.rate_limiter),
            transaction_buffer: Arc::clone(&self.transaction_buffer),
            wallet_cache: Arc::clone(&self.wallet_cache),
            is_running: Arc::clone(&self.is_running),
            processors: Arc::clone(&self.processors),
        }
    }
}
