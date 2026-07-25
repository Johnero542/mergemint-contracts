import { useTxFlow } from "../hooks/useTxFlow";
import { TxResultBanner } from "./TxResultBanner";
import { CopyButton } from "./CopyButton";
import { Bounty, NetworkName } from "../lib/types";
import { shortenAddress } from "../lib/format";

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
      <p>
        Creator: <span title={bounty.creator}>{shortenAddress(bounty.creator)}</span>
        <CopyButton value={bounty.creator} />
      </p>
      <p>Status: {bounty.status}</p>

      <ul className="assignee-list">
        {bounty.assignees.map((assignee) => (
          <li key={assignee.address}>
            <span title={assignee.address}>{shortenAddress(assignee.address)}</span>
            <CopyButton value={assignee.address} />
            {" — "}
            {(assignee.shareBp / 100).toFixed(2)}%
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
