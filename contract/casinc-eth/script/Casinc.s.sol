// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {Casinc} from "../src/Casinc.sol";
import {AdminMultisig} from "../src/AdminMultisig.sol";

contract DeployCasinc is Script {
    address[] public admins = [
        vm.envAddress("ADMIN1"),
        vm.envAddress("ADMIN2")
    ];
    uint256 public requiredConfirmations = 2;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // Deploy Admin Multisig first
        AdminMultisig admin = new AdminMultisig(admins, requiredConfirmations);

        // Deploy Casinc with admin address
        new Casinc(address(admin));

        vm.stopBroadcast();
    }
}
