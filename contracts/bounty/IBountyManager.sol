// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IBountyManager {
    /**
     * @dev Refresh a single contributor's bounty data
     * @param contributor Address of the contributor
     */
    function refreshContributor(address contributor) external;

    /**
     * @dev Batch refresh multiple contributors' bounty data
     * @param contributors Array of contributor addresses
     */
    function batchRefreshContributors(address[] calldata contributors) external;

    /**
     * @dev Get contributor bounty data
     * @param contributor Address of the contributor
     */
    function getContributorBounty(address contributor)
        external
        view
        returns (
            uint256 totalBounty,
            uint256 claimedBounty,
            uint256 pendingBounty
        );
}
