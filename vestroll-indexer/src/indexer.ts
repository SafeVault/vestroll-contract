import { Horizon } from 'stellar-sdk';
import { db, initDb, updateTransactionStatus } from './db';

const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const VAULT_CONTRACT_ID = 'your_vault_contract_id_here'; // This should come from config/env

const server = new Horizon.Server(HORIZON_URL);

async function startIndexer() {
    await initDb();
    console.log('Indexer started, listening for transactions...');

    server.transactions()
        .forAccount(VAULT_CONTRACT_ID)
        .cursor('now')
        .stream({
            onmessage: async (tx: Horizon.ServerApi.TransactionRecord) => {
                console.log(`Processing transaction: ${tx.hash}`);

                const memo = tx.memo;
                const memoType = tx.memo_type;

                if (memo && (memoType === 'text' || memoType === 'id')) {
                    const transactionId = memo;
                    console.log(`Mapping Stellar hash ${tx.hash} to internal ID ${transactionId}`);

                    try {
                        await updateTransactionStatus(transactionId, tx.hash, 'Paid');
                        console.log(`Transaction ${transactionId} updated to Paid.`);
                    } catch (error) {
                        console.error(`Failed to update transaction ${transactionId}:`, error);
                    }
                } else {
                    console.log(`Transaction ${tx.hash} has no relevant memo, skipping.`);
                }
            },
            onerror: (error) => {
                console.error('Stream error:', error);
            },
        });
}

if (VAULT_CONTRACT_ID === 'your_vault_contract_id_here') {
    console.warn('VAULT_CONTRACT_ID not set. Please update indexer.ts with the correct contract ID.');
}

startIndexer().catch(console.error);
