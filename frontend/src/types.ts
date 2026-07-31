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
  maxAssignees: number;
  tags: string[];
  milestones: Array<{
    description: string;
    reward: string;
    completed: boolean;
  }>;
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
