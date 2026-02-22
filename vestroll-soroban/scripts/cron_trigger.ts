import { Keypair, TransactionBuilder, Networks, Contract, xdr, Address, nativeToScVal, rpc } from '@stellar/stellar-sdk';
import fetch from 'node-fetch';

/**
 * MOCK DATABASE/BACKEND INTEGRATION
 * This script simulates a serverless function (Cron Job) that aggregates 
 * approved payroll payments and executes them via the VestRoll Organization Vault.
 */

const SERVER_URL = 'https://rpc-futurenet.stellar.org:443';
const NETWORK_PASSPHRASE = Networks.FUTURENET;

// Env Keys (Simulated)
const VAULT_CONTRACT_ID = process.env.VAULT_CONTRACT_ID || '';
const BATCH_PAYOUT_CONTRACT_ID = process.env.BATCH_PAYOUT_CONTRACT_ID || '';
const USDC_ASSET_ID = process.env.USDC_ASSET_ID || '';
const ADMIN_SECRET = process.env.ADMIN_SECRET || ''; // Vault Admin

interface PaymentRecord {
    employee_wallet: string;
    amount_usdc: number;
}

// 1. Fetch approved payments
async function fetchApprovedPayments(): Promise<PaymentRecord[]> {
    console.log("Fetching approved timesheets/salaries...");
    // Simulate DB delay
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Example Mock Data returning three employees
    return [
        { employee_wallet: "GBSOOMR72DRE7BNHBR3AQQE5A4YYD4X5U74P5P5Q55UFEFGF667A5L72", amount_usdc: 1500 },
        { employee_wallet: "GAIR2E24YI3Y6N22WJ66IHT4465B2BBRV7Z6K42U4N7O4EBN3B35JZZD", amount_usdc: 2200 },
        { employee_wallet: "GA5W2HT5U3Z4X4S622H7Z4W642O7Y7H3C2Z5Z6H6Y5A5O322W7U4K477", amount_usdc: 500 }
    ];
}

// 2. Trigger Smart Contract
async function processBatchPayout() {
    try {
        if (!VAULT_CONTRACT_ID || !ADMIN_SECRET) {
            console.warn("⚠️  Skipping execution: Missing Environment Variables");
            console.log("Set VAULT_CONTRACT_ID, BATCH_PAYOUT_CONTRACT_ID, USDC_ASSET_ID, and ADMIN_SECRET.");
            return;
        }

        const payments = await fetchApprovedPayments();
        if (payments.length === 0) {
            console.log("No approved payments found. Exiting.");
            return;
        }

        console.log(`Formatting ${payments.length} payments into Soroban Vector...`);

        // Format to expected xdr types for vestroll_common::Payment { recipient: Address, amount: i128 }
        const paymentScVals = payments.map(p => {
            return xdr.ScVal.scvMap([
                new xdr.ScMapEntry({
                    key: xdr.ScVal.scvSymbol('recipient'),
                    val: Address.fromString(p.employee_wallet).toScVal()
                }),
                new xdr.ScMapEntry({
                    key: xdr.ScVal.scvSymbol('amount'),
                    val: nativeToScVal(p.amount_usdc, { type: 'i128' })
                })
            ]);
        });

        const paymentsVec = xdr.ScVal.scvVec(paymentScVals);

        console.log("Building Transaction for 'batch_withdraw'...");
        const server = new rpc.Server(SERVER_URL);
        const adminKeypair = Keypair.fromSecret(ADMIN_SECRET);
        const account = await server.getAccount(adminKeypair.publicKey());

        const vaultContract = new Contract(VAULT_CONTRACT_ID);
        const tx = new TransactionBuilder(account, {
            fee: "100000",
            networkPassphrase: NETWORK_PASSPHRASE,
        })
            .addOperation(vaultContract.call("batch_withdraw",
                Address.fromString(BATCH_PAYOUT_CONTRACT_ID).toScVal(),
                Address.fromString(USDC_ASSET_ID).toScVal(),
                paymentsVec
            ))
            .setTimeout(30)
            .build();

        // 3. Output Simulation
        console.log("✅ Transaction Built Successfully");
        console.log("Admin executing batch payout to Organization Vault...");

        // For actual dispatch we would sign and submit here:
        // tx.sign(adminKeypair);
        // await server.sendTransaction(tx);

        console.log("🎉 Simulation Complete. Batch Payout Triggered.");

    } catch (e) {
        console.error("❌ Failed to process batch payout:", e);
    }
}

// Execute
if (require.main === module) {
    processBatchPayout();
}
