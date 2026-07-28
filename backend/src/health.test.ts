import { buildHealthResponse, buildHealthResponseLegacy } from './health';

describe('Health Endpoint', () => {
  it('should return enriched health response with all fields', () => {
    const response = buildHealthResponse(
      'CABC123456789',
      1000,
      1010,
      true,
      'Test Passphrase',
    );

    expect(response).toEqual({
      ok: true,
      contractId: 'CABC123456789',
      lastIndexedLedger: 1000,
      currentLedger: 1010,
      dbReachable: true,
      networkPassphrase: 'Test Passphrase',
    });
  });

  it('should report ok=true when db is reachable and currentLedger > 0', () => {
    const response = buildHealthResponse(
      'CONTRACT_ID',
      500,
      600,
      true,
    );

    expect(response.ok).toBe(true);
  });

  it('should report ok=false when db is not reachable', () => {
    const response = buildHealthResponse(
      'CONTRACT_ID',
      500,
      600,
      false,
    );

    expect(response.ok).toBe(false);
  });

  it('should report ok=false when currentLedger is 0', () => {
    const response = buildHealthResponse(
      'CONTRACT_ID',
      0,
      0,
      true,
    );

    expect(response.ok).toBe(false);
  });

  it('should correctly report indexer lag', () => {
    const response = buildHealthResponse(
      'CONTRACT_ID',
      950,
      1000,
      true,
    );

    expect(response.lastIndexedLedger).toBe(950);
    expect(response.currentLedger).toBe(1000);
    // Lag would be 50 ledgers
    const lag = response.currentLedger - response.lastIndexedLedger;
    expect(lag).toBe(50);
  });

  it('should support optional networkPassphrase parameter', () => {
    const responseWithPassphrase = buildHealthResponse(
      'CONTRACT_ID',
      100,
      200,
      true,
      'Custom Passphrase',
    );

    expect(responseWithPassphrase.networkPassphrase).toBe('Custom Passphrase');

    const responseWithoutPassphrase = buildHealthResponse(
      'CONTRACT_ID',
      100,
      200,
      true,
    );

    expect(responseWithoutPassphrase.networkPassphrase).toBeUndefined();
  });

  it('should provide legacy function for backward compatibility', () => {
    const response = buildHealthResponseLegacy('Test Passphrase');

    expect(response).toEqual({
      status: 'ok',
      networkPassphrase: 'Test Passphrase',
    });
  });
});
