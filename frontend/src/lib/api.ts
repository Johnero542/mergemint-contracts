import { Bounty, BountyPage, BountyStatus, Contributor } from '../types';

const API_BASE = import.meta.env.VITE_API_BASE_URL ?? '/api';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, init);
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(body || `Request failed with status ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export interface ListBountiesParams {
  status?: BountyStatus;
  cursor?: string;
  limit?: number;
}

function toQuery(params: Record<string, string | number | undefined>): string {
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined) search.set(key, String(value));
  }
  const qs = search.toString();
  return qs ? `?${qs}` : '';
}

export const api = {
  getBounties(params: ListBountiesParams = {}): Promise<BountyPage> {
    return request(`/bounties${toQuery(params)}`);
  },
  getBountiesByCreator(creator: string, params: ListBountiesParams = {}): Promise<BountyPage> {
    return request(`/bounties${toQuery({ ...params, creator })}`);
  },
  getBountiesByAssignee(assignee: string, params: ListBountiesParams = {}): Promise<BountyPage> {
    return request(`/bounties${toQuery({ ...params, assignee })}`);
  },
  getBounty(id: string): Promise<Bounty> {
    return request(`/bounties/${id}`);
  },
  createBounty(payload: Partial<Bounty>): Promise<Bounty> {
    return request('/bounties', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
  },
  claimBounty(id: string): Promise<Bounty> {
    return request(`/bounties/${id}/claim`, { method: 'POST' });
  },
  getContributor(address: string): Promise<Contributor> {
    return request(`/contributors/${address}`);
  },
};
