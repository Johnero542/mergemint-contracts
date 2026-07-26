export function shortenAddress(address: string, lead = 4, trail = 4): string {
  if (address.length <= lead + trail) return address;
  return `${address.slice(0, lead)}…${address.slice(-trail)}`;
}
