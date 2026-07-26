// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract MockBountyManager {
    mapping(address => uint256) public refreshCount;
    bool public shouldFail = false;

    function refreshContributor(address contributor) external {
        if (shouldFail) {
            revert("Mock refresh failed");
        }
        refreshCount[contributor]++;
    }

    function batchRefreshContributors(address[] calldata contributors) external {
        if (shouldFail) {
            revert("Mock batch refresh failed");
        }
        for (uint256 i = 0; i < contributors.length; i++) {
            refreshCount[contributors[i]]++;
        }
    }

    function getContributorBounty(address contributor)
        external
        view
        returns (
            uint256 totalBounty,
            uint256 claimedBounty,
            uint256 pendingBounty
        )
    {
        return (1000, 500, 500);
    }

    function setShouldFail(bool _shouldFail) external {
        shouldFail = _shouldFail;
    }

    function getRefreshCount(address contributor) external view returns (uint256) {
        return refreshCount[contributor];
    }
}
