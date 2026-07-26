// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IBountyManager {
    /**
     * @dev Update contributor metrics for a specific bounty
     * @param contributor The contributor address
     * @param bountyId The bounty ID
     */
    function updateContributorMetrics(address contributor, uint256 bountyId) external;

    /**
     * @dev Get all contributors for a bounty
     * @param bountyId The bounty ID
     * @return Array of contributor addresses
     */
    function getBountyContributors(uint256 bountyId) external view returns (address[] memory);

    /**
     * @dev Get contributor count for a bounty
     * @param bountyId The bounty ID
     * @return Number of contributors
     */
    function getContributorCount(uint256 bountyId) external view returns (uint256);
}
