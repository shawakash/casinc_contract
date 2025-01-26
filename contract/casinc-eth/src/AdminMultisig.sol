// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract AdminMultisig {
    address[] public admins;
    uint256 public requiredConfirmations;

    mapping(bytes32 => mapping(address => bool)) public confirmations;

    event Confirmation(address indexed admin, bytes32 indexed txHash);
    event Execution(bytes32 indexed txHash);

    modifier onlyAdmin() {
        bool isAdmin = false;
        for (uint i = 0; i < admins.length; i++) {
            if (admins[i] == msg.sender) {
                isAdmin = true;
                break;
            }
        }
        require(isAdmin, "Not admin");
        _;
    }

    constructor(address[] memory _admins, uint256 _requiredConfirmations) {
        admins = _admins;
        requiredConfirmations = _requiredConfirmations;
    }

    function confirmTransaction(bytes32 txHash) external onlyAdmin {
        confirmations[txHash][msg.sender] = true;
        emit Confirmation(msg.sender, txHash);
    }

    function executeTransaction(
        address target,
        bytes memory data,
        bytes32 txHash
    ) external onlyAdmin {
        uint256 count = 0;
        for (uint i = 0; i < admins.length; i++) {
            if (confirmations[txHash][admins[i]]) {
                count++;
            }
        }

        require(count >= requiredConfirmations, "Insufficient confirmations");

        (bool success, ) = target.call(data);
        require(success, "Execution failed");

        emit Execution(txHash);
    }
}
