// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract AOXStaking {
    mapping(address => uint256) public stakes;

    function stake() external payable {
        require(msg.value > 0, "zero stake");
        stakes[msg.sender] += msg.value;
    }

    function unstake(uint256 amount) external {
        require(stakes[msg.sender] >= amount, "insufficient stake");
        stakes[msg.sender] -= amount;
        payable(msg.sender).transfer(amount);
    }
}
