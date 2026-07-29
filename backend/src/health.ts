// Exposes the backend's health status including indexer lag and connectivity checks.
// Reports indexer lag (lastIndexedLedger vs currentLedger), database reachability,
// and network passphrase for mismatch detection.
export interface HealthResponse {
  ok: boolean;
  contractId: string;
  lastIndexedLedger: number;
  currentLedger: number;
  dbReachable: boolean;
  networkPassphrase?: string;
}

export function buildHealthResponse(
  contractId: string,
  lastIndexedLedger: number,
  currentLedger: number,
  dbReachable: boolean,
  configuredPassphrase?: string,
): HealthResponse {
  return {
    ok: dbReachable && currentLedger > 0,
    contractId,
    lastIndexedLedger,
    currentLedger,
    dbReachable,
    networkPassphrase: configuredPassphrase,
  };
}

// Legacy compatibility function - returns minimal health response
export function buildHealthResponseLegacy(configuredPassphrase: string) {
  return {
    status: "ok",
    networkPassphrase: configuredPassphrase,
  };
}
