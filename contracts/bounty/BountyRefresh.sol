// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

interface IBountyManager {
    function refreshContributor(address contributor) external;
    function batchRefreshContributors(address[] calldata contributors) external;
}

contract BountyRefresh is Ownable, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;

    IBountyManager public bountyManager;
    uint256 public constant MAX_BATCH_SIZE = 100;
    uint256 public constant BATCH_DELAY = 1 seconds;

    EnumerableSet.AddressSet private pendingContributors;
    mapping(address => uint256) public lastRefreshTime;
    mapping(address => bool) public isProcessing;

    event BatchRefreshStarted(uint256 indexed batchId, uint256 contributorCount);
    event BatchRefreshCompleted(uint256 indexed batchId, uint256 successCount, uint256 failureCount);
    event ContributorRefreshFailed(address indexed contributor, string reason);
    event BountyManagerUpdated(address indexed newManager);

    error InvalidBountyManager();
    error BatchSizeExceeded();
    error NoContributorsToRefresh();
    error ContributorAlreadyProcessing();
    error InvalidContributorList();

    constructor(address _bountyManager) {
        if (_bountyManager == address(0)) revert InvalidBountyManager();
        bountyManager = _bountyManager;
    }

    /**
     * @dev Refresh a single bounty with batched contributor updates
     * @param contributors Array of contributor addresses to refresh
     */
    function refreshBounty(address[] calldata contributors) external nonReentrant onlyOwner {
        if (contributors.length == 0) revert NoContributorsToRefresh();
        if (contributors.length > MAX_BATCH_SIZE) revert BatchSizeExceeded();

        _validateContributorList(contributors);
        _batchRefreshContributors(contributors);
    }

    /**
     * @dev Refresh bounty with parallel batch processing
     * @param contributors Array of contributor addresses
     * @param batchSize Size of each batch for processing
     */
    function refreshBountyParallel(
        address[] calldata contributors,
        uint256 batchSize
    ) external nonReentrant onlyOwner {
        if (contributors.length == 0) revert NoContributorsToRefresh();
        if (batchSize == 0 || batchSize > MAX_BATCH_SIZE) revert BatchSizeExceeded();

        _validateContributorList(contributors);
        _parallelBatchRefresh(contributors, batchSize);
    }

    /**
     * @dev Queue contributors for batch refresh
     * @param contributors Array of contributor addresses to queue
     */
    function queueContributorsForRefresh(address[] calldata contributors) external onlyOwner {
        if (contributors.length == 0) revert NoContributorsToRefresh();
        _validateContributorList(contributors);

        for (uint256 i = 0; i < contributors.length; i++) {
            pendingContributors.add(contributors[i]);
        }
    }

    /**
     * @dev Process queued contributors in batches
     * @param batchSize Number of contributors to process in this call
     */
    function processPendingBatch(uint256 batchSize) external nonReentrant onlyOwner {
        if (batchSize == 0 || batchSize > MAX_BATCH_SIZE) revert BatchSizeExceeded();
        if (pendingContributors.length() == 0) revert NoContributorsToRefresh();

        uint256 length = pendingContributors.length();
        uint256 processCount = batchSize < length ? batchSize : length;
        address[] memory batch = new address[](processCount);

        for (uint256 i = 0; i < processCount; i++) {
            batch[i] = pendingContributors.at(i);
        }

        _batchRefreshContributors(batch);

        for (uint256 i = 0; i < processCount; i++) {
            pendingContributors.remove(batch[i]);
        }
    }

    /**
     * @dev Get pending contributors count
     */
    function getPendingContributorsCount() external view returns (uint256) {
        return pendingContributors.length();
    }

    /**
     * @dev Get pending contributors
     */
    function getPendingContributors(uint256 offset, uint256 limit)
        external
        view
        returns (address[] memory)
    {
        uint256 length = pendingContributors.length();
        if (offset >= length) return new address[](0);

        uint256 resultLength = (offset + limit > length) ? (length - offset) : limit;
        address[] memory result = new address[](resultLength);

        for (uint256 i = 0; i < resultLength; i++) {
            result[i] = pendingContributors.at(offset + i);
        }

        return result;
    }

    /**
     * @dev Update bounty manager address
     * @param _newManager New bounty manager address
     */
    function setBountyManager(address _newManager) external onlyOwner {
        if (_newManager == address(0)) revert InvalidBountyManager();
        bountyManager = _newManager;
        emit BountyManagerUpdated(_newManager);
    }

    /**
     * @dev Internal function to validate contributor list
     */
    function _validateContributorList(address[] calldata contributors) internal pure {
        for (uint256 i = 0; i < contributors.length; i++) {
            if (contributors[i] == address(0)) revert InvalidContributorList();
            for (uint256 j = i + 1; j < contributors.length; j++) {
                if (contributors[i] == contributors[j]) revert InvalidContributorList();
            }
        }
    }

    /**
     * @dev Internal function to batch refresh contributors
     */
    function _batchRefreshContributors(address[] memory contributors) internal {
        uint256 batchId = uint256(keccak256(abi.encodePacked(block.timestamp, msg.sender)));
        uint256 successCount = 0;
        uint256 failureCount = 0;

        emit BatchRefreshStarted(batchId, contributors.length);

        try bountyManager.batchRefreshContributors(contributors) {
            successCount = contributors.length;
            for (uint256 i = 0; i < contributors.length; i++) {
                lastRefreshTime[contributors[i]] = block.timestamp;
            }
        } catch {
            for (uint256 i = 0; i < contributors.length; i++) {
                try bountyManager.refreshContributor(contributors[i]) {
                    successCount++;
                    lastRefreshTime[contributors[i]] = block.timestamp;
                } catch Error(string memory reason) {
                    failureCount++;
                    emit ContributorRefreshFailed(contributors[i], reason);
                } catch {
                    failureCount++;
                    emit ContributorRefreshFailed(contributors[i], "Unknown error");
                }
            }
        }

        emit BatchRefreshCompleted(batchId, successCount, failureCount);
    }

    /**
     * @dev Internal function to parallelize batch refresh
     */
    function _parallelBatchRefresh(address[] calldata contributors, uint256 batchSize) internal {
        uint256 numBatches = (contributors.length + batchSize - 1) / batchSize;

        for (uint256 batchIndex = 0; batchIndex < numBatches; batchIndex++) {
            uint256 start = batchIndex * batchSize;
            uint256 end = start + batchSize;
            if (end > contributors.length) {
                end = contributors.length;
            }

            address[] memory batch = new address[](end - start);
            for (uint256 i = start; i < end; i++) {
                batch[i - start] = contributors[i];
            }

            _batchRefreshContributors(batch);
        }
    }
}
