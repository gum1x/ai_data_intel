#!/usr/bin/env python3
"""
Blockchain Intelligence System
Advanced blockchain integration for secure data storage and intelligence gathering
"""

import asyncio
import json
import time
from datetime import datetime, timedelta
from typing import Dict, List, Any, Optional, Set
from dataclasses import dataclass
from enum import Enum
import logging
import hashlib
from eth_typing import HexStr
from eth_utils import to_checksum_address
import aiohttp

# Blockchain and Crypto
from web3 import Web3, AsyncWeb3
from web3.middleware import geth_poa_middleware
from eth_account import Account
from eth_account.messages import encode_defunct
import eth_keys
from eth_keys import keys
import bitcoin
from bitcoin.wallet import CBitcoinSecret, P2PKHBitcoinAddress

# IPFS for decentralized storage
import ipfshttpclient
from multiformats import multibase, multihash

# Smart contract interaction
from solcx import compile_source
from eth_abi import encode_abi, decode_abi

class BlockchainType(Enum):
    ETHEREUM = "ethereum"
    BITCOIN = "bitcoin"
    POLYGON = "polygon"
    BSC = "binance_smart_chain"
    ARBITRUM = "arbitrum"

class StorageType(Enum):
    IPFS = "ipfs"
    BLOCKCHAIN = "blockchain"
    HYBRID = "hybrid"

@dataclass
class BlockchainTransaction:
    """Blockchain transaction data"""
    tx_hash: str
    blockchain: BlockchainType
    from_address: str
    to_address: str
    value: float
    timestamp: datetime
    data: Optional[str]
    gas_used: Optional[int]
    status: bool
    block_number: int

@dataclass
class IntelligenceData:
    """Intelligence data for blockchain storage"""
    data_id: str
    content: Dict[str, Any]
    metadata: Dict[str, Any]
    timestamp: datetime
    source: str
    classification: str
    hash: str
    signatures: List[str]

class BlockchainStorageContract:
    """Smart contract for intelligence data storage"""
    
    CONTRACT_SOURCE = """
    // SPDX-License-Identifier: MIT
    pragma solidity ^0.8.0;

    contract IntelligenceStorage {
        struct IntelligenceRecord {
            bytes32 dataId;
            string ipfsHash;
            uint256 timestamp;
            string source;
            string classification;
            bytes32 contentHash;
            bool isEncrypted;
            address[] authorizedReaders;
        }
        
        mapping(bytes32 => IntelligenceRecord) public records;
        mapping(address => bool) public authorizedAgents;
        address public owner;
        
        event RecordStored(bytes32 indexed dataId, string ipfsHash, uint256 timestamp);
        event RecordAccessed(bytes32 indexed dataId, address indexed reader, uint256 timestamp);
        
        modifier onlyOwner() {
            require(msg.sender == owner, "Only owner can call this");
            _;
        }
        
        modifier onlyAuthorized() {
            require(authorizedAgents[msg.sender], "Not authorized");
            _;
        }
        
        constructor() {
            owner = msg.sender;
            authorizedAgents[msg.sender] = true;
        }
        
        function storeRecord(
            bytes32 _dataId,
            string memory _ipfsHash,
            string memory _source,
            string memory _classification,
            bytes32 _contentHash,
            bool _isEncrypted,
            address[] memory _authorizedReaders
        ) external onlyAuthorized {
            records[_dataId] = IntelligenceRecord(
                _dataId,
                _ipfsHash,
                block.timestamp,
                _source,
                _classification,
                _contentHash,
                _isEncrypted,
                _authorizedReaders
            );
            
            emit RecordStored(_dataId, _ipfsHash, block.timestamp);
        }
        
        function getRecord(bytes32 _dataId) external view returns (
            string memory ipfsHash,
            uint256 timestamp,
            string memory source,
            string memory classification,
            bytes32 contentHash,
            bool isEncrypted
        ) {
            IntelligenceRecord memory record = records[_dataId];
            require(record.timestamp > 0, "Record not found");
            
            bool isAuthorized = false;
            if (msg.sender == owner || authorizedAgents[msg.sender]) {
                isAuthorized = true;
            } else {
                for (uint i = 0; i < record.authorizedReaders.length; i++) {
                    if (record.authorizedReaders[i] == msg.sender) {
                        isAuthorized = true;
                        break;
                    }
                }
            }
            
            require(isAuthorized, "Not authorized to access this record");
            
            emit RecordAccessed(_dataId, msg.sender, block.timestamp);
            
            return (
                record.ipfsHash,
                record.timestamp,
                record.source,
                record.classification,
                record.contentHash,
                record.isEncrypted
            );
        }
        
        function addAuthorizedAgent(address _agent) external onlyOwner {
            authorizedAgents[_agent] = true;
        }
        
        function removeAuthorizedAgent(address _agent) external onlyOwner {
            require(_agent != owner, "Cannot remove owner");
            authorizedAgents[_agent] = false;
        }
    }
    """

class BlockchainIntelligenceSystem:
    """Main blockchain intelligence system"""
    
    def __init__(self):
        # Initialize blockchain connections
        self.web3_clients = {}
        self.ipfs_client = None
        self.contracts = {}
        self.encryption_keys = {}
        self.initialize_system()
    
    def initialize_system(self):
        """Initialize blockchain and IPFS connections"""
        try:
            # Initialize Ethereum connection
            self.web3_clients[BlockchainType.ETHEREUM] = AsyncWeb3(
                AsyncWeb3.AsyncHTTPProvider('https://mainnet.infura.io/v3/YOUR_INFURA_KEY')
            )
            
            # Initialize Polygon connection
            self.web3_clients[BlockchainType.POLYGON] = AsyncWeb3(
                AsyncWeb3.AsyncHTTPProvider('https://polygon-rpc.com')
            )
            
            # Initialize BSC connection
            self.web3_clients[BlockchainType.BSC] = AsyncWeb3(
                AsyncWeb3.AsyncHTTPProvider('https://bsc-dataseed.binance.org/')
            )
            
            # Initialize IPFS
            self.ipfs_client = ipfshttpclient.connect('/ip4/127.0.0.1/tcp/5001')
            
            # Deploy or load contracts
            self.deploy_contracts()
            
            logging.info("Blockchain intelligence system initialized")
            
        except Exception as e:
            logging.error(f"Blockchain system initialization error: {e}")
    
    def deploy_contracts(self):
        """Deploy or load smart contracts"""
        try:
            # Compile contract
            compiled_sol = compile_source(self.CONTRACT_SOURCE)
            contract_interface = compiled_sol['<stdin>:IntelligenceStorage']
            
            # Deploy to each chain
            for blockchain_type, web3 in self.web3_clients.items():
                try:
                    contract_address = self.get_existing_contract(blockchain_type)
                    if not contract_address:
                        contract_address = self.deploy_new_contract(
                            web3, contract_interface, blockchain_type
                        )
                    
                    self.contracts[blockchain_type] = web3.eth.contract(
                        address=contract_address,
                        abi=contract_interface['abi']
                    )
                    
                except Exception as e:
                    logging.error(f"Contract deployment error for {blockchain_type}: {e}")
            
        except Exception as e:
            logging.error(f"Contract deployment error: {e}")
    
    def get_existing_contract(self, blockchain_type: BlockchainType) -> Optional[str]:
        """Get existing contract address"""
        # This would load from configuration or database
        return None
    
    async def deploy_new_contract(self, web3: AsyncWeb3, 
                                contract_interface: Dict[str, Any],
                                blockchain_type: BlockchainType) -> str:
        """Deploy new contract"""
        try:
            account = self.get_deployment_account(blockchain_type)
            
            contract = web3.eth.contract(
                abi=contract_interface['abi'],
                bytecode=contract_interface['bin']
            )
            
            # Get nonce
            nonce = await web3.eth.get_transaction_count(account.address)
            
            # Estimate gas
            gas_estimate = await contract.constructor().estimate_gas()
            
            # Get gas price
            gas_price = await web3.eth.gas_price
            
            # Build transaction
            transaction = {
                'nonce': nonce,
                'gas': int(gas_estimate * 1.2),  # Add 20% buffer
                'gasPrice': gas_price,
                'from': account.address
            }
            
            # Deploy contract
            tx_hash = await contract.constructor().transact(transaction)
            
            # Wait for receipt
            receipt = await web3.eth.wait_for_transaction_receipt(tx_hash)
            
            return receipt.contractAddress
            
        except Exception as e:
            logging.error(f"Contract deployment error: {e}")
            return None
    
    def get_deployment_account(self, blockchain_type: BlockchainType) -> Account:
        """Get account for contract deployment"""
        # This would load from secure configuration
        private_key = "YOUR_PRIVATE_KEY"
        return Account.from_key(private_key)
    
    async def store_intelligence_data(self, data: IntelligenceData, 
                                    storage_type: StorageType = StorageType.HYBRID) -> Dict[str, Any]:
        """Store intelligence data"""
        try:
            if storage_type == StorageType.IPFS:
                return await self.store_on_ipfs(data)
            elif storage_type == StorageType.BLOCKCHAIN:
                return await self.store_on_blockchain(data)
            else:  # HYBRID
                return await self.store_hybrid(data)
                
        except Exception as e:
            logging.error(f"Data storage error: {e}")
            return {'error': str(e)}
    
    async def store_on_ipfs(self, data: IntelligenceData) -> Dict[str, Any]:
        """Store data on IPFS"""
        try:
            # Encrypt sensitive data
            encrypted_data = self.encrypt_sensitive_data(data)
            
            # Add to IPFS
            ipfs_result = self.ipfs_client.add_json(encrypted_data)
            ipfs_hash = ipfs_result['Hash']
            
            return {
                'storage_type': 'ipfs',
                'ipfs_hash': ipfs_hash,
                'timestamp': datetime.now().isoformat(),
                'data_id': data.data_id
            }
            
        except Exception as e:
            logging.error(f"IPFS storage error: {e}")
            return {'error': str(e)}
    
    async def store_on_blockchain(self, data: IntelligenceData) -> Dict[str, Any]:
        """Store data directly on blockchain"""
        try:
            # Choose blockchain based on data size and requirements
            blockchain_type = self.select_blockchain(data)
            contract = self.contracts[blockchain_type]
            
            # Prepare data
            data_bytes = Web3.keccak(text=json.dumps(data.content))
            
            # Store on blockchain
            tx_hash = await contract.functions.storeRecord(
                data_bytes,
                "",  # No IPFS hash
                data.source,
                data.classification,
                data.hash,
                True,  # Encrypted
                []  # No additional readers
            ).transact()
            
            # Wait for receipt
            receipt = await self.web3_clients[blockchain_type].eth.wait_for_transaction_receipt(tx_hash)
            
            return {
                'storage_type': 'blockchain',
                'blockchain': blockchain_type.value,
                'tx_hash': receipt.transactionHash.hex(),
                'block_number': receipt.blockNumber,
                'data_id': data.data_id
            }
            
        except Exception as e:
            logging.error(f"Blockchain storage error: {e}")
            return {'error': str(e)}
    
    async def store_hybrid(self, data: IntelligenceData) -> Dict[str, Any]:
        """Store data using hybrid approach (IPFS + Blockchain)"""
        try:
            # Store main data on IPFS
            ipfs_result = await self.store_on_ipfs(data)
            if 'error' in ipfs_result:
                raise Exception(ipfs_result['error'])
            
            # Store reference on blockchain
            blockchain_type = self.select_blockchain(data)
            contract = self.contracts[blockchain_type]
            
            # Prepare data
            data_bytes = Web3.keccak(text=data.data_id)
            
            # Store on blockchain
            tx_hash = await contract.functions.storeRecord(
                data_bytes,
                ipfs_result['ipfs_hash'],
                data.source,
                data.classification,
                data.hash,
                True,  # Encrypted
                []  # No additional readers
            ).transact()
            
            # Wait for receipt
            receipt = await self.web3_clients[blockchain_type].eth.wait_for_transaction_receipt(tx_hash)
            
            return {
                'storage_type': 'hybrid',
                'ipfs_hash': ipfs_result['ipfs_hash'],
                'blockchain': blockchain_type.value,
                'tx_hash': receipt.transactionHash.hex(),
                'block_number': receipt.blockNumber,
                'data_id': data.data_id
            }
            
        except Exception as e:
            logging.error(f"Hybrid storage error: {e}")
            return {'error': str(e)}
    
    def select_blockchain(self, data: IntelligenceData) -> BlockchainType:
        """Select best blockchain based on data requirements"""
        # This would implement sophisticated selection logic
        # For now, default to Polygon for cost-effectiveness
        return BlockchainType.POLYGON
    
    def encrypt_sensitive_data(self, data: IntelligenceData) -> Dict[str, Any]:
        """Encrypt sensitive data"""
        try:
            # This would implement actual encryption
            # For now, return mock encrypted data
            return {
                'encrypted_content': str(data.content),
                'metadata': data.metadata,
                'timestamp': data.timestamp.isoformat(),
                'source': data.source,
                'classification': data.classification
            }
        except Exception as e:
            logging.error(f"Encryption error: {e}")
            return data.content
    
    async def retrieve_intelligence_data(self, data_id: str) -> Optional[IntelligenceData]:
        """Retrieve intelligence data"""
        try:
            # Check all blockchains
            for blockchain_type, contract in self.contracts.items():
                try:
                    # Get record
                    record = await contract.functions.getRecord(
                        Web3.keccak(text=data_id)
                    ).call()
                    
                    if record and record[0]:  # Has IPFS hash
                        # Get from IPFS
                        ipfs_data = self.ipfs_client.cat(record[0])
                        decrypted_data = self.decrypt_data(ipfs_data)
                        
                        return IntelligenceData(
                            data_id=data_id,
                            content=decrypted_data.get('content', {}),
                            metadata=decrypted_data.get('metadata', {}),
                            timestamp=datetime.fromtimestamp(record[1]),
                            source=record[2],
                            classification=record[3],
                            hash=record[4].hex(),
                            signatures=[]
                        )
                    
                except Exception as e:
                    logging.error(f"Retrieval error from {blockchain_type}: {e}")
                    continue
            
            return None
            
        except Exception as e:
            logging.error(f"Data retrieval error: {e}")
            return None
    
    def decrypt_data(self, encrypted_data: bytes) -> Dict[str, Any]:
        """Decrypt data"""
        try:
            # This would implement actual decryption
            # For now, return mock decrypted data
            return json.loads(encrypted_data)
        except Exception as e:
            logging.error(f"Decryption error: {e}")
            return {}
    
    async def verify_data_integrity(self, data_id: str) -> Dict[str, bool]:
        """Verify data integrity across storage systems"""
        try:
            verification_results = {
                'ipfs_verified': False,
                'blockchain_verified': False,
                'cross_reference_verified': False
            }
            
            # Get data
            data = await self.retrieve_intelligence_data(data_id)
            if not data:
                return verification_results
            
            # Verify IPFS integrity
            if data.content:
                verification_results['ipfs_verified'] = True
            
            # Verify blockchain integrity
            for blockchain_type, contract in self.contracts.items():
                try:
                    record = await contract.functions.getRecord(
                        Web3.keccak(text=data_id)
                    ).call()
                    
                    if record and record[4].hex() == data.hash:
                        verification_results['blockchain_verified'] = True
                        break
                        
                except Exception as e:
                    continue
            
            # Verify cross-reference integrity
            if verification_results['ipfs_verified'] and verification_results['blockchain_verified']:
                verification_results['cross_reference_verified'] = True
            
            return verification_results
            
        except Exception as e:
            logging.error(f"Verification error: {e}")
            return {'error': str(e)}
    
    async def analyze_blockchain_transactions(self, address: str, 
                                           blockchain_type: BlockchainType) -> List[BlockchainTransaction]:
        """Analyze blockchain transactions for an address"""
        transactions = []
        
        try:
            web3 = self.web3_clients[blockchain_type]
            
            # Get latest block
            latest_block = await web3.eth.block_number
            
            # Analyze last 10000 blocks
            start_block = max(0, latest_block - 10000)
            
            # Get transactions
            for block_number in range(start_block, latest_block + 1):
                block = await web3.eth.get_block(block_number, full_transactions=True)
                
                for tx in block.transactions:
                    if tx['to'] and (tx['to'].lower() == address.lower() or 
                                   tx['from'].lower() == address.lower()):
                        
                        # Get transaction receipt
                        receipt = await web3.eth.get_transaction_receipt(tx['hash'])
                        
                        transaction = BlockchainTransaction(
                            tx_hash=tx['hash'].hex(),
                            blockchain=blockchain_type,
                            from_address=tx['from'],
                            to_address=tx['to'],
                            value=web3.from_wei(tx['value'], 'ether'),
                            timestamp=datetime.fromtimestamp(block.timestamp),
                            data=tx.get('input'),
                            gas_used=receipt['gasUsed'],
                            status=receipt['status'],
                            block_number=block_number
                        )
                        
                        transactions.append(transaction)
            
            return transactions
            
        except Exception as e:
            logging.error(f"Transaction analysis error: {e}")
            return []
    
    async def monitor_blockchain_activity(self, addresses: List[str], 
                                        callback: Callable[[BlockchainTransaction], None]):
        """Monitor blockchain activity for addresses"""
        try:
            # Monitor each blockchain
            for blockchain_type, web3 in self.web3_clients.items():
                asyncio.create_task(
                    self.monitor_blockchain(blockchain_type, web3, addresses, callback)
                )
            
        except Exception as e:
            logging.error(f"Monitoring error: {e}")
    
    async def monitor_blockchain(self, blockchain_type: BlockchainType,
                               web3: AsyncWeb3, addresses: List[str],
                               callback: Callable[[BlockchainTransaction], None]):
        """Monitor specific blockchain"""
        try:
            while True:
                # Get latest block
                block = await web3.eth.get_block('latest', full_transactions=True)
                
                # Check transactions
                for tx in block.transactions:
                    if tx['to'] and (tx['to'].lower() in [addr.lower() for addr in addresses] or
                                   tx['from'].lower() in [addr.lower() for addr in addresses]):
                        
                        # Get receipt
                        receipt = await web3.eth.get_transaction_receipt(tx['hash'])
                        
                        transaction = BlockchainTransaction(
                            tx_hash=tx['hash'].hex(),
                            blockchain=blockchain_type,
                            from_address=tx['from'],
                            to_address=tx['to'],
                            value=web3.from_wei(tx['value'], 'ether'),
                            timestamp=datetime.now(),  # Block timestamp not available in new blocks
                            data=tx.get('input'),
                            gas_used=receipt['gasUsed'],
                            status=receipt['status'],
                            block_number=block.number
                        )
                        
                        # Call callback
                        callback(transaction)
                
                await asyncio.sleep(1)  # Wait for next block
                
        except Exception as e:
            logging.error(f"Blockchain monitoring error: {e}")
            await asyncio.sleep(60)  # Wait before retrying

# Main execution
async def main():
    """Main execution function"""
    system = BlockchainIntelligenceSystem()
    
    try:
        # Test data storage
        test_data = IntelligenceData(
            data_id=str(time.time()),
            content={'test': 'data'},
            metadata={'source': 'test'},
            timestamp=datetime.now(),
            source='test',
            classification='test',
            hash=hashlib.sha256(b'test').hexdigest(),
            signatures=[]
        )
        
        result = await system.store_intelligence_data(test_data)
        print(f"Storage result: {result}")
        
        # Keep running
        while True:
            await asyncio.sleep(3600)
            
    except KeyboardInterrupt:
        logging.info("System shutdown requested")
    except Exception as e:
        logging.critical(f"System error: {e}")

if __name__ == "__main__":
    print("=== BLOCKCHAIN INTELLIGENCE SYSTEM ===")
    print("Secure data storage and verification")
    print("Initializing...")
    
    asyncio.run(main())
