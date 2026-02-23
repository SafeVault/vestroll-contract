document.addEventListener('DOMContentLoaded', () => {
    const payBtn = document.getElementById('payInvoiceBtn');
    const statusBadge = document.getElementById('statusBadge');
    const txDetails = document.getElementById('txDetails');
    const txHashEl = document.getElementById('txHash');

    payBtn.addEventListener('click', async () => {
        // Prevent double clicks
        if (payBtn.classList.contains('loading')) return;

        // Visual feedback
        payBtn.classList.add('loading');
        payBtn.textContent = 'Processing Transaction...';

        // MOCK: Simulate Soroban Contract Call & Indexer Delay
        // In reality, this would use @stellar/freighter-api to sign the tx,
        // send it to Soroban RPC, and await the indexer webhook.
        try {
            // Simulate the delay of the ledger closing
            await new Promise(resolve => setTimeout(resolve, 2500));

            // Mock transaction hash
            const mockHash = '0x' + Math.random().toString(16).substr(2, 40);

            // Update UI State to reflect "Paid" as required by criteria
            statusBadge.textContent = 'Paid';
            statusBadge.classList.remove('pending');
            statusBadge.classList.add('paid');

            // Show recorded hash
            txHashEl.textContent = mockHash;
            txDetails.classList.remove('hidden');

            // Cleanup button
            payBtn.textContent = 'Payment Confirmed';
            payBtn.style.background = 'var(--success)';
            payBtn.style.boxShadow = 'none';
            payBtn.disabled = true;

        } catch (error) {
            console.error('Payment failed', error);
            payBtn.textContent = 'Payment Failed';
            payBtn.classList.remove('loading');
        }
    });
});
