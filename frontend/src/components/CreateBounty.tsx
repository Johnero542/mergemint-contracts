import { useState } from "react";
import { useTxFlow } from "../hooks/useTxFlow";
import { TxResultBanner } from "./TxResultBanner";
import { NetworkName } from "../lib/types";

interface CreateBountyProps {
  network: NetworkName;
  onSubmit: (form: { title: string; description: string; rewardAmount: string }) => Promise<{ hash: string }>;
}

export function CreateBounty({ network, onSubmit }: CreateBountyProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [rewardAmount, setRewardAmount] = useState("");
  const [multisigOpen, setMultisigOpen] = useState(false);
  const [verifiers, setVerifiers] = useState<string[]>([""]);
  const [threshold, setThreshold] = useState(1);
  const [maxAssignees, setMaxAssignees] = useState(1);
  const { pending, error, result, run } = useTxFlow(network);

  async function perform() {
    await run(() => onSubmit({ title, description, rewardAmount }));
  }

  return (
    <form
      className="create-bounty"
      onSubmit={(e) => {
        e.preventDefault();
        perform();
      }}
    >
      <label>
        Title
        <input value={title} onChange={(e) => setTitle(e.target.value)} />
      </label>
      <label>
        Description
        <textarea value={description} onChange={(e) => setDescription(e.target.value)} />
      </label>
      <label>
        Reward amount
        <input value={rewardAmount} onChange={(e) => setRewardAmount(e.target.value)} />
      </label>

      {/* Stub UI only — wire up once contract issues #9/#10 (multi-assignee) ship. */}
      <label>
        Max assignees
        <input
          type="number"
          min={1}
          value={maxAssignees}
          onChange={(e) => setMaxAssignees(Number(e.target.value))}
        />
        <span className="create-bounty__hint">
          When more than one assignee is allowed, the reward is split across claimants by share
          (see the assignee list on the bounty detail page). Not yet enforced on-chain — pending
          contract issues #9/#10.
        </span>
      </label>

      {/* Stub UI only — wire up once contract issue #11 (multi-sig verifiers) ships. */}
      <details
        className="create-bounty__advanced"
        open={multisigOpen}
        onToggle={(e) => setMultisigOpen((e.target as HTMLDetailsElement).open)}
      >
        <summary>Advanced: multi-sig verifiers</summary>

        {verifiers.map((verifier, i) => (
          <div className="verifier-row" key={i}>
            <input
              placeholder="Verifier address"
              value={verifier}
              onChange={(e) => {
                const next = [...verifiers];
                next[i] = e.target.value;
                setVerifiers(next);
              }}
            />
            <button
              type="button"
              onClick={() => setVerifiers(verifiers.filter((_, idx) => idx !== i))}
              disabled={verifiers.length <= 1}
            >
              Remove
            </button>
          </div>
        ))}
        <button type="button" onClick={() => setVerifiers([...verifiers, ""])}>
          Add verifier
        </button>

        <label>
          Approval threshold
          <select value={threshold} onChange={(e) => setThreshold(Number(e.target.value))}>
            {verifiers.map((_, i) => (
              <option key={i} value={i + 1}>
                {i + 1} of {verifiers.length}
              </option>
            ))}
          </select>
        </label>

        <p className="create-bounty__advanced-note">
          Multi-sig verification is not yet enforced on-chain — this form does not submit these
          fields until contract issue #11 ships.
        </p>
      </details>

      <button type="submit" disabled={pending}>
        {pending ? "Submitting…" : "Create bounty"}
      </button>

      {error && <p className="error">{error}</p>}
      <TxResultBanner result={result} />
    </form>
  );
}
