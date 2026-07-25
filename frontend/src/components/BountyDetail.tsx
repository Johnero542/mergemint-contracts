import { useTxFlow } from "../hooks/useTxFlow";
import { TxResultBanner } from "./TxResultBanner";
import { Bounty, NetworkName } from "../lib/types";

interface BountyDetailProps {
  bounty: Bounty;
  network: NetworkName;
  onClaim: (bountyId: string) => Promise<{ hash: string }>;
}

export function BountyDetail({ bounty, network, onClaim }: BountyDetailProps) {
  const { pending, error, result, run } = useTxFlow(network);

  async function perform() {
    await run(() => onClaim(bounty.id));
  }

  return (
    <div className="bounty-detail">
      <h2>{bounty.rewardAmount.toString()} {bounty.rewardToken}</h2>
      <p>Creator: {bounty.creator}</p>
      <p>Status: {bounty.status}</p>

      <ul className="assignee-list">
        {bounty.assignees.map((assignee) => (
          <li key={assignee.address}>
            {assignee.address} — {(assignee.shareBp / 100).toFixed(2)}%
          </li>
        ))}
      </ul>

      <button onClick={perform} disabled={pending}>
        {pending ? "Submitting…" : "Claim"}
      </button>

      {error && <p className="error">{error}</p>}
      <TxResultBanner result={result} />
    </div>
  );
}
