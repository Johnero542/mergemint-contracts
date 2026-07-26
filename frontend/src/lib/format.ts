export function formatAddress(address: string): string {
  if (!address || address.length <= 10) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

export function formatReward(amount: string): string {
  return `${amount} XLM`;
}

// Known contract/backend error substrings mapped to user-friendly copy.
// Falls back to the raw message when nothing matches (see issue #57).
const ERROR_MAP: Array<[RegExp, string]> = [
  [/contributor already has an active claim/i, 'You already have an active claim on this bounty.'],
  [/bounty (is )?not found/i, 'This bounty no longer exists.'],
  [/bounty (is )?already claimed/i, 'This bounty has already been claimed by someone else.'],
  [/insufficient (funds|balance)/i, "You don't have enough balance to complete this action."],
  [/unauthorized|not (the )?creator/i, "You don't have permission to perform this action."],
  [/dispute window (has )?closed/i, 'The dispute window for this bounty has closed.'],
  [/internal server error/i, 'Something went wrong on our end. Please try again shortly.'],
  [/network ?error|failed to fetch/i, 'Unable to reach the server. Check your connection and try again.'],
];

export function mapErrorMessage(raw: string): string {
  if (!raw) return 'Something went wrong. Please try again.';
  for (const [pattern, friendly] of ERROR_MAP) {
    if (pattern.test(raw)) return friendly;
  }
  return raw;
}
