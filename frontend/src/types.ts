export type BountyStatus = 'open' | 'claimed' | 'disputed' | 'completed' | 'cancelled';

export interface Bounty {
  id: string;
  title: string;
  description: string;
  reward: string;
  status: BountyStatus;
  creator: string;
  assignee?: string;
  createdAt: string;
}

export interface BountyPage {
  bounties: Bounty[];
  nextCursor: string | null;
}

export interface Contributor {
  address: string;
  reputation: number;
  completedBounties: number;
}
