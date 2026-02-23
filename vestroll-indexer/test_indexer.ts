import { updateTransactionStatus, initDb, db } from './src/db';

async function testIndexer() {
    await initDb();

    // Insert a dummy pending invoice transaction
    const mockTxId = 'INV-12345';
    const mockStellarHash = '0x123abc456def';

    // Insert pending
    await db('transactions').insert({
        id: mockTxId,
        status: 'Pending',
    }).onConflict('id').ignore();

    console.log(`Inserted pending transaction: ${mockTxId}`);

    // Update to Paid
    await updateTransactionStatus(mockTxId, mockStellarHash, 'Paid');

    console.log('Updated db with Paid status.');

    // Fetch and explicitly check
    const row = await db('transactions').where({ id: mockTxId }).first();
    console.log('Resulting record:', row);

    // Cleanup mock row
    await db('transactions').where({ id: mockTxId }).del();
    process.exit(0);
}

testIndexer().catch(console.error);
