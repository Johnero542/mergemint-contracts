import React from 'react';
import { useWallet } from '../lib/WalletContext';
import { formatAddress } from '../lib/format';

export function WalletConnectButton() {
  const { address, connecting, error, connect, disconnect } = useWallet();

  if (address) {
    return (
      <button onClick={disconnect} title={address}>
        {formatAddress(address)}
      </button>
    );
  }

  return (
    <div>
      <button onClick={connect} disabled={connecting}>
        {connecting ? 'Connecting...' : 'Connect Wallet'}
      </button>
      {error && <span role="alert">{error}</span>}
    </div>
  );
}
