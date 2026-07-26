const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("BountyRefresh", function () {
    let bountyRefresh;
    let owner;
    let addr1, addr2, addr3;

    beforeEach(async function () {
        [owner, addr1, addr2, addr3] = await ethers.getSigners();

        const BountyRefresh = await ethers.getContractFactory("BountyRefresh");
        bountyRefresh = await BountyRefresh.deploy();
        await bountyRefresh.deployed();
    });

    describe("Batch Creation", function () {
        it("Should create a batch with valid contributors and bounty IDs", async function () {
            const contributors = [addr1.address, addr2.address];
            const bountyIds = [1, 2];

            const tx = await bountyRefresh.createBatch(contributors, bountyIds);
            const receipt = await tx.wait();

            expect(receipt.events.some((e) => e.event === "BatchCreated")).to.be
                .true;
        });

        it("Should reject batch with mismatched lengths", async function () {
            const contributors = [addr1.address, addr2.address];
            const bountyIds = [1];

            await expect(
                bountyRefresh.createBatch(contributors, bountyIds)
            ).to.be.revertedWith("Contributors and bountyIds length mismatch");
        });

        it("Should reject empty batch", async function () {
            const contributors = [];
            const bountyIds = [];

            await expect(
                bountyRefresh.createBatch(contributors, bountyIds)
            ).to.be.revertedWith("Empty batch");
        });

        it("Should reject batch exceeding max size", async function () {
            const contributors = Array(101).fill(addr1.address);
            const bountyIds = Array(101).fill(1);

            await expect(
                bountyRefresh.createBatch(contributors, bountyIds)
            ).to.be.revertedWith("Invalid batch size");
        });

        it("Should only allow owner to create batch", async function () {
            const contributors = [addr1.address];
            const bountyIds = [1];

            await expect(
                bountyRefresh
                    .connect(addr1)
                    .createBatch(contributors, bountyIds)
            ).to.be.revertedWith("Ownable: caller is not the owner");
        });
    });

    describe("Batch Processing", function () {
        beforeEach(async function () {
            const contributors = [addr1.address, addr2.address, addr3.address];
            const bountyIds = [1, 2, 3];
            await bountyRefresh.createBatch(contributors, bountyIds);
        });

        it("Should process batch in parallel", async function () {
            const tx = await bountyRefresh.processBatchParallel(0);
            const receipt = await tx.wait();

            expect(
                receipt.events.some((e) => e.event === "ParallelRefreshStarted")
            ).to.be.true;
        });

        it("Should reject processing non-existent batch", async function () {
            await expect(
                bountyRefresh.processBatchParallel(999)
            ).to.be.revertedWith("Batch does not exist");
        });

        it("Should reject double processing", async function () {
            await bountyRefresh.processBatchParallel(0);
            await expect(
                bountyRefresh.processBatchParallel(0)
            ).to.be.revertedWith("Batch already processing");
        });
    });

    describe("Batch Finalization", function () {
        beforeEach(async function () {
            const contributors = [addr1.address, addr2.address];
            const bountyIds = [1, 2];
            await bountyRefresh.createBatch(contributors, bountyIds);
            await bountyRefresh.processBatchParallel(0);
        });

        it("Should finalize batch", async function () {
            const tx = await bountyRefresh.finalizeBatch(0);
            const receipt = await tx.wait();

            expect(
                receipt.events.some((e) => e.event === "BatchProcessingCompleted")
            ).to.be.true;
        });

        it("Should reject finalizing non-processing batch", async function () {
            const contributors = [addr1.address];
            const bountyIds = [1];
            await bountyRefresh.createBatch(contributors, bountyIds);

            await expect(
                bountyRefresh.finalizeBatch(1)
            ).to.be.revertedWith("Batch not processing");
        });
    });

    describe("Batch Retrieval", function () {
        beforeEach(async function () {
            const contributors = [addr1.address, addr2.address];
            const bountyIds = [1, 2];
            await bountyRefresh.createBatch(contributors, bountyIds);
        });

        it("Should retrieve batch details", async function () {
            const batch = await bountyRefresh.getBatch(0);
            expect(batch.id).to.equal(0);
            expect(batch.contributors.length).to.equal(2);
            expect(batch.bountyIds.length).to.equal(2);
        });

        it("Should reject retrieving non-existent batch", async function () {
            await expect(bountyRefresh.getBatch(999)).to.be.revertedWith(
                "Batch does not exist"
            );
        });
    });

    describe("Pause/Unpause", function () {
        it("Should pause and unpause processing", async function () {
            await bountyRefresh.pause();

            const contributors = [addr1.address];
            const bountyIds = [1];
            await bountyRefresh.createBatch(contributors, bountyIds);

            await expect(
                bountyRefresh.processBatchParallel(0)
            ).to.be.revertedWith("Pausable: paused");

            await bountyRefresh.unpause();
            await expect(bountyRefresh.processBatchParallel(0)).not.to.be
                .reverted;
        });
    });
});
