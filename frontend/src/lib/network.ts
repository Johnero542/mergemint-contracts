import { NetworkName } from "./types";

const EXPLORER_BASE: Record<NetworkName, string> = {
  testnet: "https://stellar.expert/explorer/testnet",
  mainnet: "https://stellar.expert/explorer/public",
};

export function explorerTxUrl(hash: string, network: NetworkName): string {
  return `${EXPLORER_BASE[network]}/tx/${hash}`;
}
