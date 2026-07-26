import React, { useMemo, useState } from "react";
import { CharCounter } from "./CharCounter";
import {
  SYMBOL_MAX_LENGTH,
  isValidContractAddress,
  isValidRewardAmount,
} from "../lib/validation";

export interface CreateBountyFormValues {
  title: string;
  description: string;
  rewardAmount: string;
  rewardToken: string;
}

interface CreateBountyProps {
  onSubmit: (values: CreateBountyFormValues) => void;
}

export function CreateBounty({ onSubmit }: CreateBountyProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [rewardAmount, setRewardAmount] = useState("");
  const [rewardToken, setRewardToken] = useState("");

  const rewardAmountError = useMemo(() => {
    if (rewardAmount === "") return null;
    return isValidRewardAmount(rewardAmount)
      ? null
      : "Enter a positive number with up to 7 decimal places.";
  }, [rewardAmount]);

  const rewardTokenError = useMemo(() => {
    if (rewardToken === "") return null;
    return isValidContractAddress(rewardToken)
      ? null
      : 'Enter a valid Soroban contract address (starts with "C", 56 characters).';
  }, [rewardToken]);

  const isFormValid =
    title.trim() !== "" &&
    description.trim() !== "" &&
    isValidRewardAmount(rewardAmount) &&
    isValidContractAddress(rewardToken);

  function handleSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!isFormValid) return;
    onSubmit({ title, description, rewardAmount, rewardToken });
  }

  return (
    <form onSubmit={handleSubmit} className="create-bounty-form">
      <label htmlFor="bounty-title">Title</label>
      <input
        id="bounty-title"
        type="text"
        value={title}
        maxLength={SYMBOL_MAX_LENGTH}
        onChange={(event) => setTitle(event.target.value)}
      />
      <div className="field-footer">
        <CharCounter length={title.length} max={SYMBOL_MAX_LENGTH} />
        <span className="helper-text">
          Stored on-chain as a Symbol, limited to {SYMBOL_MAX_LENGTH}{" "}
          characters.
        </span>
      </div>

      <label htmlFor="bounty-description">Description</label>
      <textarea
        id="bounty-description"
        value={description}
        maxLength={SYMBOL_MAX_LENGTH}
        onChange={(event) => setDescription(event.target.value)}
      />
      <div className="field-footer">
        <CharCounter length={description.length} max={SYMBOL_MAX_LENGTH} />
        <span className="helper-text">
          Stored on-chain as a Symbol, limited to {SYMBOL_MAX_LENGTH}{" "}
          characters.
        </span>
      </div>

      <label htmlFor="bounty-reward-amount">Reward Amount</label>
      <input
        id="bounty-reward-amount"
        type="text"
        inputMode="decimal"
        value={rewardAmount}
        onChange={(event) => setRewardAmount(event.target.value)}
      />
      {rewardAmountError && (
        <span className="field-error">{rewardAmountError}</span>
      )}

      <label htmlFor="bounty-reward-token">Reward Token Address</label>
      <input
        id="bounty-reward-token"
        type="text"
        value={rewardToken}
        onChange={(event) => setRewardToken(event.target.value)}
      />
      {rewardTokenError && (
        <span className="field-error">{rewardTokenError}</span>
      )}

      <button type="submit" disabled={!isFormValid}>
        Create Bounty
      </button>
    </form>
  );
}
