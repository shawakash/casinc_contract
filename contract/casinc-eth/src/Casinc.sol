// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

contract Casinc is ReentrancyGuard, Ownable {
    struct WithdrawalRequest {
        uint256 amount;
        uint256 unlockTime;
        bool approved;
    }

    mapping(address => uint256) public deposits;
    mapping(address => uint256) public winnings;
    mapping(address => WithdrawalRequest) public withdrawalRequests;

    uint256 public multiplier = 2;
    uint256 public withdrawalDelay = 1 days;
    address public adminContract;

    event Deposited(address indexed user, uint256 amount);
    event BetPlaced(address indexed user, uint256 betAmount, uint256 winnings);
    event WithdrawalRequested(address indexed user, uint256 amount);
    event WithdrawalExecuted(address indexed user, uint256 amount);

    modifier onlyAdmin() {
        require(msg.sender == adminContract, "Not authorized");
        _;
    }

    constructor(address _adminContract) Ownable(_adminContract) {
        adminContract = _adminContract;
    }

    function deposit() external payable {
        deposits[msg.sender] += msg.value;
        emit Deposited(msg.sender, msg.value);
    }

    function placeBet(uint256 betAmount) external nonReentrant {
        require(deposits[msg.sender] >= betAmount, "Insufficient funds");
        deposits[msg.sender] -= betAmount;

        uint256 winningAmount = betAmount * multiplier;
        winnings[msg.sender] += winningAmount;

        emit BetPlaced(msg.sender, betAmount, winningAmount);
    }

    function requestWithdrawal(uint256 amount) external {
        require(winnings[msg.sender] >= amount, "Insufficient winnings");
        winnings[msg.sender] -= amount;

        withdrawalRequests[msg.sender] = WithdrawalRequest({
            amount: amount,
            unlockTime: block.timestamp + withdrawalDelay,
            approved: false
        });

        emit WithdrawalRequested(msg.sender, amount);
    }

    function approveWithdrawal(address user) external onlyAdmin {
        WithdrawalRequest storage request = withdrawalRequests[user];
        require(block.timestamp >= request.unlockTime, "Withdrawal locked");
        request.approved = true;
    }

    function executeWithdrawal() external nonReentrant {
        WithdrawalRequest storage request = withdrawalRequests[msg.sender];
        require(request.approved, "Withdrawal not approved");
        require(request.amount > 0, "No withdrawal pending");

        uint256 amount = request.amount;
        delete withdrawalRequests[msg.sender];

        (bool success, ) = msg.sender.call{value: amount}("");
        require(success, "Transfer failed");

        emit WithdrawalExecuted(msg.sender, amount);
    }

    function setMultiplier(uint256 _multiplier) external onlyAdmin {
        multiplier = _multiplier;
    }

    function setWithdrawalDelay(uint256 _delay) external onlyAdmin {
        withdrawalDelay = _delay;
    }
}
