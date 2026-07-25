import { MergeMintSDK, MAINNET } from "./index";

describe("MergeMintSDK constructor", () => {
  it("throws a clear error when given the unmodified MAINNET placeholder RPC URL", () => {
    expect(
      () =>
        new MergeMintSDK({
          ...MAINNET,
          contractId: "CONTRACT_ID",
        }),
    ).toThrow(/placeholder/);
  });
});
