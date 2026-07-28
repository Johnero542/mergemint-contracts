export interface Assignee {
  address: string;
  shareBp: number;
}

export interface Bounty {
  id: string;
  creator: string;
  rewardAmount: bigint;
  rewardToken: string;
  assignees: Assignee[];
  maxAssignees: number;
  status: string;
  minReputation: number;
  deadline: number | null;
}

export type NetworkName = "testnet" | "mainnet";

export interface SubmitResult {
  hash: string;
  network: NetworkName;
  ledger?: number;
}
