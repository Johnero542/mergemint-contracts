const hre = require("hardhat");
const { ethers } = require("hardhat");

/**
 * Batch refresh utility for managing contributor bounty refreshes
 */
class BatchRefreshManager {
    constructor(contractAddress) {
        this.contractAddress = contractAddress;
        this.contract = null;
        this.batchSize = 100;
        this.maxRetries = 3;
        this.retryDelay = 1000; // ms
    }

    /**
     * Initialize the contract instance
     */
    async initialize() {
        const BountyRefresh = await hre.ethers.getContractFactory("BountyRefresh");
        this.contract = BountyRefresh.attach(this.contractAddress);
    }

    /**
     * Split array into chunks
     */
    chunkArray(array, size) {
        const chunks = [];
        for (let i = 0; i < array.length; i += size) {
            chunks.push(array.slice(i, i + size));
        }
        return chunks;
    }

    /**
     * Create and process batch refresh
     */
    async processBatchRefresh(contributors, bountyIds, options = {}) {
        const { parallel = true, verbose = false } = options;

        if (contributors.length !== bountyIds.length) {
            throw new Error("Contributors and bountyIds must have same length");
        }

        if (contributors.length === 0) {
            throw new Error("Empty contributors array");
        }

        const chunks = this.chunkArray(contributors, this.batchSize);
        const results = [];

        for (let i = 0; i < chunks.length; i++) {
            const contributorChunk = chunks[i];
            const bountyIdChunk = bountyIds.slice(
                i * this.batchSize,
                (i + 1) * this.batchSize
            );

            if (verbose) {
                console.log(
                    `Processing batch ${i + 1}/${chunks.length} with ${contributorChunk.length} contributors`
                );
            }

            try {
                const batchResult = await this._processSingleBatch(
                    contributorChunk,
                    bountyIdChunk,
                    parallel,
                    verbose
                );
                results.push(batchResult);
            } catch (error) {
                console.error(`Error processing batch ${i + 1}:`, error.message);
                results.push({
                    batchIndex: i,
                    success: false,
                    error: error.message,
                });
            }
        }

        return {
            totalBatches: chunks.length,
            results,
            summary: this._summarizeResults(results),
        };
    }

    /**
     * Process a single batch
     */
    async _processSingleBatch(contributors, bountyIds, parallel, verbose) {
        let retries = 0;
        let lastError;

        while (retries < this.maxRetries) {
            try {
                // Create batch
                const createTx = await this.contract.createBatch(
                    contributors,
                    bountyIds
                );
                const createReceipt = await createTx.wait();

                if (verbose) {
                    console.log(`Batch created in tx: ${createTx.hash}`);
                }

                // Extract batch ID from events
                const batchId = this._extractBatchId(createReceipt);

                // Process batch
                const processTx = await this.contract.processBatchParallel(
                    batchId
                );
                const processReceipt = await processTx.wait();

                if (verbose) {
                    console.log(`Batch processed in tx: ${processTx.hash}`);
                }

                // Finalize batch
                const finalizeTx = await this.contract.finalizeBatch(batchId);
                await finalizeTx.wait();

                if (verbose) {
                    console.log(`Batch finalized in tx: ${finalizeTx.hash}`);
                }

                // Get batch details
                const batchDetails = await this.contract.getBatch(batchId);

                return {
                    batchId: batchId.toString(),
                    success: true,
                    successCount: batchDetails.successCount.toString(),
                    failureCount: batchDetails.failureCount.toString(),
                    transactionHashes: {
                        create: createTx.hash,
                        process: processTx.hash,
                        finalize: finalizeTx.hash,
                    },
                };
            } catch (error) {
                lastError = error;
                retries++;

                if (retries < this.maxRetries) {
                    if (verbose) {
                        console.log(
                            `Retry ${retries}/${this.maxRetries} after ${this.retryDelay}ms...`
                        );
                    }
                    await this._delay(this.retryDelay);
                }
            }
        }

        throw new Error(
            `Failed after ${this.maxRetries} retries: ${lastError.message}`
        );
    }

    /**
     * Extract batch ID from transaction receipt
     */
    _extractBatchId(receipt) {
        const batchCreatedEvent = receipt.events?.find(
            (e) => e.event === "BatchCreated"
        );
        if (!batchCreatedEvent) {
            throw new Error("BatchCreated event not found in receipt");
        }
        return batchCreatedEvent.args.batchId;
    }

    /**
     * Summarize batch processing results
     */
    _summarizeResults(results) {
        const successful = results.filter((r) => r.success).length;
        const failed = results.filter((r) => !r.success).length;
        const totalSuccess = results
            .filter((r) => r.success)
            .reduce((sum, r) => sum + BigInt(r.successCount), 0n);
        const totalFailure = results
            .filter((r) => r.success)
            .reduce((sum, r) => sum + BigInt(r.failureCount), 0n);

        return {
            successfulBatches: successful,
            failedBatches: failed,
            totalContributorsProcessed: totalSuccess.toString(),
            totalContributorsFailed: totalFailure.toString(),
        };
    }

    /**
     * Delay utility
     */
    _delay(ms) {
        return new Promise((resolve) => setTimeout(resolve, ms));
    }
}

/**
 * Main execution
 */
async function main() {
    const contractAddress = process.env.BOUNTY_REFRESH_ADDRESS;

    if (!contractAddress) {
        throw new Error("BOUNTY_REFRESH_ADDRESS environment variable not set");
    }

    const manager = new BatchRefreshManager(contractAddress);
    await manager.initialize();

    // Example usage
    const contributors = [
        "0x1234567890123456789012345678901234567890",
        "0x0987654321098765432109876543210987654321",
        // ... more contributors
    ];

    const bountyIds = [1, 2]; // corresponding bounty IDs

    const result = await manager.processBatchRefresh(contributors, bountyIds, {
        parallel: true,
        verbose: true,
    });

    console.log("\nBatch Refresh Summary:");
    console.log(JSON.stringify(result, null, 2));
}

if (require.main === module) {
    main().catch((error) => {
        console.error(error);
        process.exit(1);
    });
}

module.exports = BatchRefreshManager;
