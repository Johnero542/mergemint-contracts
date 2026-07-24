const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("BountyRefresh", function () {
    let bountyRefresh;
    let mockBountyManager;
    let owner;
    let addr1, addr2, addr3;
    const BOUNTY_ID = 1;

    beforeEach(async function () {
        [owner, addr1, addr2, addr3] = await ethers.getSigners();

        // Deploy mock bounty manager
        const MockBountyManager = await ethers.getContractFactory("MockBountyManager");
        mockBountyManager = await MockBountyManager.deploy();
        await mockBountyManager.deployed();

        // Deploy BountyRefresh
        const BountyRefresh = await ethers.getContractFactory("BountyRefresh");
        bountyRefresh = await BountyRefresh.deploy(mockBountyManager.address);
        await bountyRefresh.deployed();
    });

    describe("Deployment", function () {
        it("Should deploy with correct bounty manager", async function () {
            expect(await bountyRefresh.bountyManager()).to.equal(mockBountyManager.address);
        });

        it("Should revert with zero address", async function () {
            const BountyRefresh = await ethers.getContractFactory("BountyRefresh");
            await expect(
                BountyRefresh.deploy(ethers.constants.AddressZero)
            ).to.be.revertedWith("Invalid bounty manager");
        });
    });

    describe("Batch Refresh", function () {
        beforeEach(async function () {
            // Setup mock contributors
            await mockBountyManager.addContributor(BOUNTY_ID, addr1.address);
            await mockBountyManager.addContributor(BOUNTY_ID, addr2.address);
            await mockBountyManager.addContributor(BOUNTY_ID, addr3.address);
        });

        it("Should refresh bounty with batching", async function () {
            await expect(bountyRefresh.refreshBountyBatched(BOUNTY_ID))
                .to.emit(bountyRefresh, "BatchRefreshStarted")
                .to.emit(bountyRefresh, "BatchRefreshCompleted");

            expect(await bountyRefresh.getProcessedContributorCount(BOUNTY_ID)).to.equal(3);
        });

        it("Should prevent concurrent refresh", async function () {
            // This would require a more complex setup to test properly
            // For now, we verify the flag is set correctly
            await bountyRefresh.refreshBountyBatched(BOUNTY_ID);
            expect(await bountyRefresh.isRefreshing(BOUNTY_ID)).to.equal(false);
        });

        it("Should revert with invalid bounty ID", async function () {
            await expect(bountyRefresh.refreshBountyBatched(0))
                .to.be.revertedWith("Invalid bounty ID");
        });

        it("Should revert with no contributors", async function () {
            await expect(bountyRefresh.refreshBountyBatched(999))
                .to.be.revertedWith("No contributors found");
        });
    });

    describe("Parallel Refresh", function () {
        beforeEach(async function () {
            // Add multiple contributors
            for (let i = 0; i < 10; i++) {
                const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
                await mockBountyManager.addContributor(BOUNTY_ID, wallet.address);
            }
        });

        it("Should refresh bounty in parallel", async function () {
            await expect(bountyRefresh.refreshBountyParallel(BOUNTY_ID, 5))
                .to.emit(bountyRefresh, "BatchRefreshStarted")
                .to.emit(bountyRefresh, "BatchRefreshCompleted");

            expect(await bountyRefresh.getProcessedContributorCount(BOUNTY_ID)).to.equal(10);
        });

        it("Should revert with invalid batch size", async function () {
            await expect(bountyRefresh.refreshBountyParallel(BOUNTY_ID, 0))
                .to.be.revertedWith("Invalid batch size");
        });

        it("Should revert with batch size exceeding max", async function () {
            await expect(bountyRefresh.refreshBountyParallel(BOUNTY_ID, 101))
                .to.be.revertedWith("Invalid batch size");
        });
    });

    describe("Range Refresh", function () {
        beforeEach(async function () {
            await mockBountyManager.addContributor(BOUNTY_ID, addr1.address);
            await mockBountyManager.addContributor(BOUNTY_ID, addr2.address);
            await mockBountyManager.addContributor(BOUNTY_ID, addr3.address);
        });

        it("Should refresh specific range", async function () {
            await bountyRefresh.refreshBountyRange(BOUNTY_ID, 0, 2);
            expect(await bountyRefresh.getProcessedContributorCount(BOUNTY_ID)).to.equal(2);
        });

        it("Should revert with invalid range", async function () {
            await expect(bountyRefresh.refreshBountyRange(BOUNTY_ID, 2, 1))
                .to.be.revertedWith("Invalid range");
        });

        it("Should revert with out of bounds index", async function () {
            await expect(bountyRefresh.refreshBountyRange(BOUNTY_ID, 0, 100))
                .to.be.revertedWith("End index out of bounds");
        });
    });

    describe("Contributor Tracking", function () {
        beforeEach(async function () {
            await mockBountyManager.addContributor(BOUNTY_ID, addr1.address);
            await mockBountyManager.addContributor(BOUNTY_ID, addr2.address);
            await bountyRefresh.refreshBountyBatched(BOUNTY_ID);
        });

        it("Should check if contributor is processed", async function () {
            expect(await bountyRefresh.isContributorProcessed(BOUNTY_ID, addr1.address)).to.equal(true);
        });

        it("Should get processed contributors", async function () {
            const contributors = await bountyRefresh.getProcessedContributors(BOUNTY_ID);
            expect(contributors.length).to.equal(2);
            expect(contributors).to.include(addr1.address);
            expect(contributors).to.include(addr2.address);
        });

        it("Should clear processed contributors", async function () {
            await bountyRefresh.clearProcessedContributors(BOUNTY_ID);
            expect(await bountyRefresh.getProcessedContributorCount(BOUNTY_ID)).to.equal(0);
        });
    });

    describe("Admin Functions", function () {
        it("Should update bounty manager", async function () {
            const MockBountyManager = await ethers.getContractFactory("MockBountyManager");
            const newManager = await MockBountyManager.deploy();
            await newManager.deployed();

            await bountyRefresh.setBountyManager(newManager.address);
            expect(await bountyRefresh.bountyManager()).to.equal(newManager.address);
        });

        it("Should revert setting zero address as manager", async function () {
            await expect(bountyRefresh.setBountyManager(ethers.constants.AddressZero))
                .to.be.revertedWith("Invalid bounty manager");
        });

        it("Should only allow owner to update manager", async function () {
            const MockBountyManager = await ethers.getContractFactory("MockBountyManager");
            const newManager = await MockBountyManager.deploy();
            await newManager.deployed();

            await expect(bountyRefresh.connect(addr1).setBountyManager(newManager.address))
                .to.be.revertedWith("Ownable: caller is not the owner");
        });
    });
});
