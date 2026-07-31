import React from 'react';
import { BountyStatus } from '../types';

export function StatusBadge({ status }: { status: BountyStatus }) {
  return <span className={`status-badge status-badge--${status}`}>{status}</span>;
}
