const TOKEN_DECIMALS = 7;

/**
 * Converts a raw on-chain integer amount (stroops-style, 7 decimal places)
 * into a human-readable decimal string.
 */
export function formatTokenAmount(raw: string): string {
  const negative = raw.startsWith("-");
  const digits = negative ? raw.slice(1) : raw;
  const padded = digits.padStart(TOKEN_DECIMALS + 1, "0");
  const whole = padded.slice(0, -TOKEN_DECIMALS).replace(/^0+(?=\d)/, "");
  const fraction = padded.slice(-TOKEN_DECIMALS).replace(/0+$/, "");
  const result = fraction ? `${whole}.${fraction}` : whole;
  return negative ? `-${result}` : result;
}

/**
 * Converts a human-readable decimal string into a raw on-chain integer
 * amount at 7 decimal places. Inverse of formatTokenAmount.
 */
export function toRawTokenAmount(value: string): string {
  const negative = value.startsWith("-");
  const trimmed = negative ? value.slice(1) : value;
  const [wholePart, fractionPart = ""] = trimmed.split(".");
  const fraction = fractionPart.slice(0, TOKEN_DECIMALS).padEnd(TOKEN_DECIMALS, "0");
  const raw = `${wholePart}${fraction}`.replace(/^0+(?=\d)/, "");
  return negative ? `-${raw}` : raw;
}

/**
 * Shortens a wallet/contract address for display, e.g. "GABCDE…4567".
 * Addresses of 12 characters or fewer are returned unchanged.
 */
export function shortenAddress(address: string): string {
  if (address.length <= 12) {
    return address;
  }
  return `${address.slice(0, 6)}…${address.slice(-4)}`;
}
