import { SorobanRpc } from "@stellar/stellar-sdk";

export interface NetworkPassphraseCheckConfig {
  rpcUrl: string;
  configuredPassphrase: string;
}

// Fails fast on startup if NETWORK_PASSPHRASE doesn't match the network the
// configured RPC endpoint actually serves, instead of failing silently on
// the first signed transaction.
export async function validateNetworkPassphrase(
  config: NetworkPassphraseCheckConfig,
): Promise<void> {
  const rpc = new SorobanRpc.Server(config.rpcUrl);
  const { passphrase: actualPassphrase } = await rpc.getNetwork();

  if (actualPassphrase !== config.configuredPassphrase) {
    throw new Error(
      `NETWORK_PASSPHRASE mismatch: configured "${config.configuredPassphrase}" ` +
        `but RPC endpoint ${config.rpcUrl} reports "${actualPassphrase}". ` +
        "Update NETWORK_PASSPHRASE to match the network the contract was deployed to.",
    );
  }
}
