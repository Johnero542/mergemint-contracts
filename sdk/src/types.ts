export interface NetworkConfig {
  rpcUrl: string;
  networkPassphrase: string;
  contractId: string;
}

export interface Bounty {
  creator: string;
  rewardAmount: bigint;
  rewardToken: string;
  assignees: Array<{ address: string; shareBp: number }>;
  maxAssignees: number;
  status: string;
  minReputation: number;
  deadline: number | null;
}

export interface BountyMeta {
  title: string;
  description: string;
}

export interface Contributor {
  address: string;
  reputation: number;
  totalEarned: bigint;
  contributionCount: number;
  activeClaims: number;
  metadata: string | null;
}

export interface CreateBountyParams {
  creator: string;
  title: string;
  description: string;
  rewardAmount: bigint;
  rewardToken: string;
  maxAssignees?: number;
  minReputation?: number;
  deadline?: number | null;
}
