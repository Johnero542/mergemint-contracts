export type BountyStatus =
  | "open"
  | "claimed"
  | "submitted"
  | "disputed"
  | "completed"
  | "cancelled"
  | "expired";

export interface Bounty {
  creator: string;
  assignee: string | null;
  verifiers: string[];
  approvals: string[];
  status: BountyStatus;
  deadline: number;
}

export interface BountyActionContext {
  bounty: Bounty;
  walletAddress: string | null;
  now?: number;
}

function isVerifier(bounty: Bounty, walletAddress: string | null): boolean {
  return walletAddress !== null && bounty.verifiers.includes(walletAddress);
}

export function canClaim({ bounty, walletAddress }: BountyActionContext): boolean {
  if (!walletAddress) return false;
  if (bounty.status !== "open") return false;
  return walletAddress !== bounty.creator;
}

export function canCancel({ bounty, walletAddress }: BountyActionContext): boolean {
  if (!walletAddress) return false;
  return walletAddress === bounty.creator && bounty.status === "open";
}

export function canExpire({ bounty, walletAddress, now = Date.now() }: BountyActionContext): boolean {
  if (!walletAddress) return false;
  if (bounty.status !== "claimed" && bounty.status !== "submitted") return false;
  return now > bounty.deadline;
}

export function canDispute({ bounty, walletAddress }: BountyActionContext): boolean {
  if (!walletAddress) return false;
  if (bounty.status !== "submitted") return false;
  return walletAddress === bounty.creator || walletAddress === bounty.assignee;
}

export function canResolve({ bounty, walletAddress }: BountyActionContext): boolean {
  if (!walletAddress) return false;
  if (bounty.status !== "disputed") return false;
  if (!isVerifier(bounty, walletAddress)) return false;
  return !bounty.approvals.includes(walletAddress);
}

export function canVerify({ bounty, walletAddress }: BountyActionContext): boolean {
  if (!walletAddress) return false;
  if (bounty.status !== "submitted") return false;
  if (!isVerifier(bounty, walletAddress)) return false;
  return !bounty.approvals.includes(walletAddress);
}

interface BountyDetailProps {
  bounty: Bounty;
  walletAddress: string | null;
  onClaim?: () => void;
  onCancel?: () => void;
  onExpire?: () => void;
  onDispute?: () => void;
  onResolve?: () => void;
  onVerify?: () => void;
}

export default function BountyDetail({
  bounty,
  walletAddress,
  onClaim,
  onCancel,
  onExpire,
  onDispute,
  onResolve,
  onVerify,
}: BountyDetailProps) {
  const ctx: BountyActionContext = { bounty, walletAddress };

  return (
    <div className="bounty-detail">
      {canClaim(ctx) && (
        <button type="button" onClick={onClaim}>
          Claim
        </button>
      )}
      {canCancel(ctx) && (
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
      )}
      {canExpire(ctx) && (
        <button type="button" onClick={onExpire}>
          Mark Expired
        </button>
      )}
      {canDispute(ctx) && (
        <button type="button" onClick={onDispute}>
          Dispute
        </button>
      )}
      {canResolve(ctx) && (
        <button type="button" onClick={onResolve}>
          Resolve
        </button>
      )}
      {canVerify(ctx) && (
        <button type="button" onClick={onVerify}>
          Verify
        </button>
      )}
    </div>
  );
}
