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

      <button type="submit" disabled={pending}>
        {pending ? "Submitting…" : "Create bounty"}
      </button>

      {error && <p className="error">{error}</p>}
      <TxResultBanner result={result} />
    </form>
  );
}
