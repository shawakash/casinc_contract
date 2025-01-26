// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/Casinc.sol";
import "../src/AdminMultisig.sol";

contract CasincTest is Test {
    Casinc public casinc;
    AdminMultisig public admin;

    address[] public admins;
    address user = address(1);

    function setUp() public {
        admins.push(address(this));
        admins.push(address(2));

        admin = new AdminMultisig(admins, 1);
        casinc = new Casinc(address(admin));

        vm.deal(user, 10 ether);
    }

    function testDepositAndBet() public {
        vm.startPrank(user);
        casinc.deposit{value: 1 ether}();
        casinc.placeBet(0.5 ether);
        vm.stopPrank();

        assertEq(casinc.deposits(user), 0.5 ether);
        assertEq(casinc.winnings(user), 1 ether);
    }

    function testWithdrawalFlow() public {
        // Deposit and bet
        vm.startPrank(user);
        casinc.deposit{value: 1 ether}();
        casinc.placeBet(0.5 ether);

        // Request withdrawal
        casinc.requestWithdrawal(1 ether);
        vm.stopPrank();

        // Fast forward time
        vm.warp(block.timestamp + 1 days + 1);

        // Admin approval process
        bytes memory data = abi.encodeWithSignature(
            "approveWithdrawal(address)",
            user
        );
        bytes32 txHash = keccak256(data);

        // Admin confirms
        vm.prank(admins[0]);
        admin.confirmTransaction(txHash);

        // Admin executes
        vm.prank(admins[0]);
        admin.executeTransaction(address(casinc), data, txHash);

        // Execute withdrawal
        vm.startPrank(user);
        casinc.executeWithdrawal();
        vm.stopPrank();

        // error here
        assertEq(address(user).balance, 10 ether - 0.5 ether + 1 ether);
    }
}
