// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

interface IBountyManager {
    function updateContributorMetrics(address contributor, uint256 bountyId) external;
    function getBountyContributors(uint256 bountyId) external view returns (address[] memory);
}

contract BountyRefresh is Ownable, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;
    using EnumerableSet for EnumerableSet.UintSet;

    IBountyManager public bountyManager;
    
    uint256 public constant MAX_BATCH_SIZE = 100;
    uint256 public constant MAX_PARALLEL_TASKS = 50;
    
    mapping(uint256 => EnumerableSet.AddressSet) private bountyContributors;
    mapping(uint256 => bool) public isRefreshing;
    mapping(uint256 => uint256) public lastRefreshTime;
    
    event BatchRefreshStarted(uint256 indexed bountyId, uint256 totalContributors);
    event BatchRefreshCompleted(uint256 indexed bountyId, uint256 processedCount);
    event BatchRefreshFailed(uint256 indexed bountyId, string reason);
    event ContributorRefreshed(uint256 indexed bountyId, address indexed contributor);

    constructor(address _bountyManager) {
        require(_bountyManager != address(0), "Invalid bounty manager");
        bountyManager = IBountyManager(_bountyManager);
    }

    /**
     * @dev Refresh all contributors for a bounty using batching
     * @param bountyId The ID of the bounty to refresh
     */
    function refreshBountyBatched(uint256 bountyId) external nonReentrant {
        require(!isRefreshing[bountyId], "Refresh already in progress");
        require(bountyId > 0, "Invalid bounty ID");
        
        isRefreshing[bountyId] = true;
        
        try {
            address[] memory contributors = bountyManager.getBountyContributors(bountyId);
            require(contributors.length > 0, "No contributors found");
            
            emit BatchRefreshStarted(bountyId, contributors.length);
            
            uint256 processedCount = _processBatch(bountyId, contributors, 0, MAX_BATCH_SIZE);
            
            lastRefreshTime[bountyId] = block.timestamp;
            emit BatchRefreshCompleted(bountyId, processedCount);
        } catch Error(string memory reason) {
            emit BatchRefreshFailed(bountyId, reason);
            revert(reason);
        } finally {
            isRefreshing[bountyId] = false;
        }
    }

    /**
     * @dev Refresh contributors in parallel batches
     * @param bountyId The ID of the bounty to refresh
     * @param batchSize Size of each batch (max MAX_BATCH_SIZE)
     */
    function refreshBountyParallel(uint256 bountyId, uint256 batchSize) external nonReentrant {
        require(!isRefreshing[bountyId], "Refresh already in progress");
        require(bountyId > 0, "Invalid bounty ID");
        require(batchSize > 0 && batchSize <= MAX_BATCH_SIZE, "Invalid batch size");
        
        isRefreshing[bountyId] = true;
        
        try {
            address[] memory contributors = bountyManager.getBountyContributors(bountyId);
            require(contributors.length > 0, "No contributors found");
            
            emit BatchRefreshStarted(bountyId, contributors.length);
            
            uint256 totalProcessed = 0;
            uint256 numBatches = (contributors.length + batchSize - 1) / batchSize;
            uint256 parallelBatches = numBatches > MAX_PARALLEL_TASKS ? MAX_PARALLEL_TASKS : numBatches;
            
            for (uint256 i = 0; i < parallelBatches; i++) {
                uint256 startIdx = i * batchSize;
                uint256 endIdx = startIdx + batchSize;
                if (endIdx > contributors.length) {
                    endIdx = contributors.length;
                }
                
                uint256 batchProcessed = _processBatch(bountyId, contributors, startIdx, endIdx);
                totalProcessed += batchProcessed;
            }
            
            lastRefreshTime[bountyId] = block.timestamp;
            emit BatchRefreshCompleted(bountyId, totalProcessed);
        } catch Error(string memory reason) {
            emit BatchRefreshFailed(bountyId, reason);
            revert(reason);
        } finally {
            isRefreshing[bountyId] = false;
        }
    }

    /**
     * @dev Refresh a specific range of contributors
     * @param bountyId The ID of the bounty
     * @param startIndex Start index in contributors array
     * @param endIndex End index in contributors array (exclusive)
     */
    function refreshBountyRange(
        uint256 bountyId,
        uint256 startIndex,
        uint256 endIndex
    ) external nonReentrant {
        require(bountyId > 0, "Invalid bounty ID");
        require(startIndex < endIndex, "Invalid range");
        require(endIndex - startIndex <= MAX_BATCH_SIZE, "Range too large");
        
        try {
            address[] memory contributors = bountyManager.getBountyContributors(bountyId);
            require(endIndex <= contributors.length, "End index out of bounds");
            
            uint256 processedCount = _processBatch(bountyId, contributors, startIndex, endIndex);
            emit BatchRefreshCompleted(bountyId, processedCount);
        } catch Error(string memory reason) {
            emit BatchRefreshFailed(bountyId, reason);
            revert(reason);
        }
    }

    /**
     * @dev Internal function to process a batch of contributors
     * @param bountyId The bounty ID
     * @param contributors Array of contributor addresses
     * @param startIdx Start index (inclusive)
     * @param endIdx End index (exclusive)
     * @return processedCount Number of successfully processed contributors
     */
    function _processBatch(
        uint256 bountyId,
        address[] memory contributors,
        uint256 startIdx,
        uint256 endIdx
    ) internal returns (uint256 processedCount) {
        require(endIdx <= contributors.length, "Index out of bounds");
        require(startIdx <= endIdx, "Invalid range");
        
        processedCount = 0;
        
        for (uint256 i = startIdx; i < endIdx; i++) {
            address contributor = contributors[i];
            
            if (contributor == address(0)) {
                continue;
            }
            
            try bountyManager.updateContributorMetrics(contributor, bountyId) {
                bountyContributors[bountyId].add(contributor);
                emit ContributorRefreshed(bountyId, contributor);
                processedCount++;
            } catch {
                // Continue processing other contributors on individual failure
                continue;
            }
        }
    }

    /**
     * @dev Get the number of processed contributors for a bounty
     * @param bountyId The bounty ID
     * @return Number of processed contributors
     */
    function getProcessedContributorCount(uint256 bountyId) external view returns (uint256) {
        return bountyContributors[bountyId].length();
    }

    /**
     * @dev Get processed contributors for a bounty
     * @param bountyId The bounty ID
     * @return Array of processed contributor addresses
     */
    function getProcessedContributors(uint256 bountyId) external view returns (address[] memory) {
        uint256 length = bountyContributors[bountyId].length();
        address[] memory contributors = new address[](length);
        
        for (uint256 i = 0; i < length; i++) {
            contributors[i] = bountyContributors[bountyId].at(i);
        }
        
        return contributors;
    }

    /**
     * @dev Check if a contributor has been processed for a bounty
     * @param bountyId The bounty ID
     * @param contributor The contributor address
     * @return True if contributor has been processed
     */
    function isContributorProcessed(uint256 bountyId, address contributor) external view returns (bool) {
        return bountyContributors[bountyId].contains(contributor);
    }

    /**
     * @dev Clear processed contributors for a bounty
     * @param bountyId The bounty ID
     */
    function clearProcessedContributors(uint256 bountyId) external onlyOwner {
        require(!isRefreshing[bountyId], "Cannot clear while refresh in progress");
        
        uint256 length = bountyContributors[bountyId].length();
        for (uint256 i = 0; i < length; i++) {
            bountyContributors[bountyId].remove(bountyContributors[bountyId].at(0));
        }
    }

    /**
     * @dev Update bounty manager address
     * @param _bountyManager New bounty manager address
     */
    function setBountyManager(address _bountyManager) external onlyOwner {
        require(_bountyManager != address(0), "Invalid bounty manager");
        bountyManager = IBountyManager(_bountyManager);
    }
}
