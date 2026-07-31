import { useState } from "react";

export function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }

  return (
    <button
      type="button"
      className="copy-button"
      title={value}
      onClick={handleCopy}
      aria-label="Copy to clipboard"
    >
      {copied ? "Copied!" : "⧉"}
    </button>
  );
}
