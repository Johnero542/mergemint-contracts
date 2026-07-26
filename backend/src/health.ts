// Exposes the backend's configured network passphrase via /api/health so
// the frontend can detect and warn on a NETWORK_PASSPHRASE mismatch.
export function buildHealthResponse(configuredPassphrase: string) {
  return {
    status: "ok",
    networkPassphrase: configuredPassphrase,
  };
}
