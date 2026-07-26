import React, { useState } from "react";
import { CharCounter } from "./CharCounter";
import { SYMBOL_MAX_LENGTH } from "../lib/validation";

export interface ContributorProfileFormValues {
  metadata: string;
}

interface ContributorProfileProps {
  initialMetadata?: string;
  onSave: (values: ContributorProfileFormValues) => void;
}

export function ContributorProfile({
  initialMetadata = "",
  onSave,
}: ContributorProfileProps) {
  const [metadata, setMetadata] = useState(initialMetadata);

  const isOverLimit = metadata.length > SYMBOL_MAX_LENGTH;

  function handleSave(event: React.FormEvent) {
    event.preventDefault();
    if (isOverLimit) return;
    onSave({ metadata });
  }

  return (
    <form onSubmit={handleSave} className="contributor-profile-form">
      <label htmlFor="contributor-metadata">Metadata</label>
      <input
        id="contributor-metadata"
        type="text"
        value={metadata}
        maxLength={SYMBOL_MAX_LENGTH}
        onChange={(event) => setMetadata(event.target.value)}
      />
      <div className="field-footer">
        <CharCounter length={metadata.length} max={SYMBOL_MAX_LENGTH} />
        <span className="helper-text">
          Stored on-chain as a Symbol, limited to {SYMBOL_MAX_LENGTH}{" "}
          characters.
        </span>
      </div>
      {isOverLimit && (
        <span className="field-error">
          Metadata exceeds the {SYMBOL_MAX_LENGTH}-character on-chain limit.
        </span>
      )}
      <button type="submit" disabled={isOverLimit}>
        Save
      </button>
    </form>
  );
}
