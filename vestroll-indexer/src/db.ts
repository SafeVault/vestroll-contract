import knex from 'knex';
import path from 'path';

const dbPath = path.resolve(__dirname, '../vestroll.sqlite');

export const db = knex({
    client: 'sqlite3',
    connection: {
        filename: dbPath,
    },
    useNullAsDefault: true,
});

export async function initDb() {
    const hasTable = await db.schema.hasTable('transactions');
    if (!hasTable) {
        await db.schema.createTable('transactions', (table) => {
            table.string('id').primary(); // Internal transaction_id
            table.string('stellar_tx_hash').unique();
            table.string('status').defaultTo('Pending');
            table.timestamp('updated_at').defaultTo(db.fn.now());
        });
        console.log('Database initialized: transactions table created.');
    }
}

export async function updateTransactionStatus(transactionId: string, txHash: string, status: string) {
    await db('transactions')
        .where({ id: transactionId })
        .update({
            stellar_tx_hash: txHash,
            status: status,
            updated_at: db.fn.now(),
        });
}
