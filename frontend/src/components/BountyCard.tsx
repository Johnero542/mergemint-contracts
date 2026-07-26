import React from 'react';
import { Link } from 'react-router-dom';
import { Bounty } from '../types';
import { formatReward } from '../lib/format';
import { StatusBadge } from './StatusBadge';

export function BountyCard({ bounty }: { bounty: Bounty }) {
  return (
    <Link to={`/bounties/${bounty.id}`} className="bounty-card">
      <h3>{bounty.title}</h3>
      <StatusBadge status={bounty.status} />
      <p>{formatReward(bounty.reward)}</p>
    </Link>
  );
}
