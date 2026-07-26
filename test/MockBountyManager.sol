// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

contract MockBountyManager {
    using EnumerableSet for EnumerableSet.AddressSet;

    mapping(uint256 => EnumerableSet.AddressSet) private bountyContributors;
    mapping(uint256 => mapping(address => uint256)) public contributorMetrics;

    event MetricsUpdated(address indexed contributor, uint256 indexed bountyId, uint256 newValue);

    function addContributor(uint256 bountyId, address contributor) external {
        bountyContributors[bountyId].add(contributor);
    }

    function removeContributor(uint256 bountyId, address contributor) external {
        bountyContributors[bountyId].remove(contributor);
    }

    function updateContributorMetrics(address contributor, uint256 bountyId) external {
        require(contributor != address(0), "Invalid contributor");
        require(bountyId > 0, "Invalid bounty ID");
        
        contributorMetrics[bountyId][contributor]++;
        emit MetricsUpdated(contributor, bountyId, contributorMetrics[bountyId][contributor]);
    }

    function getBountyContributors(uint256 bountyId) external view returns (address[] memory) {
        uint256 length = bountyContributors[bountyId].length();
        address[] memory contributors = new address[](length);
        
        for (uint256 i = 0; i < length; i++) {
            contributors[i] = bountyContributors[bountyId].at(i);
        }
        
        return contributors;
    }

    function getContributorCount(uint256 bountyId) external view returns (uint256) {
        return bountyContributors[bountyId].length();
    }

    function getContributorMetrics(uint256 bountyId, address contributor) external view returns (uint256) {
        return contributorMetrics[bountyId][contributor];
    }
}
