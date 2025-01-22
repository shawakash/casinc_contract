import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import { Casinc } from "../target/types/casinc";
import { assert, expect } from "chai";
import bs58 from "bs58";
import * as dotenv from "dotenv";

dotenv.config({ path: __dirname + "/../.env" });

describe("casinc", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const programId = new PublicKey(
    "FtZ7YQmqr9px1rtr3EzcYrZ6SYXUgvTQMMESDAwyT1mG"
  );
  const program = new anchor.Program<Casinc>(
    require("../target/idl/casinc.json"),
    programId,
    provider
  ) as Program<Casinc>;

  // Load keypairs with proper validation
  const admin1 = Keypair.fromSecretKey(bs58.decode(process.env.ADMIN1_SK));
  const admin2 = Keypair.fromSecretKey(bs58.decode(process.env.ADMIN2_SK));
  const user = Keypair.fromSecretKey(bs58.decode(process.env.USER_SK));

  let gameParamsPDA: PublicKey;
  let userStatePDA: PublicKey;
  let withdrawalRequestPDA: PublicKey;

  // Add this helper to advance time
  const advanceTime = async (seconds: number) => {
    await provider.connection.requestAirdrop(user.publicKey, 1e9);
    await program.methods
      .advanceClock(new anchor.BN(seconds))
      .accounts({
        clock: SYSVAR_CLOCK_PUBKEY,
      })
      .rpc();
  };

  it("Initialize Game Parameters", async () => {
    [gameParamsPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("game_params")],
      program.programId
    );

    await program.methods
      .initialize(
        new anchor.BN(2),
        new anchor.BN(60),
        [admin1.publicKey, admin2.publicKey],
        2
      )
      .accounts({
        gameParams: gameParamsPDA,
        admin: admin1.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([admin1])
      .rpc();

    console.log("Game parameters initialized");
  });

  it("Initialize User State", async () => {
    [userStatePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_state"), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializeUser()
      .accounts({
        userState: userStatePDA,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    // Verify initialization
    const userState = await program.account.userState.fetch(userStatePDA);
    assert.equal(userState.user.toString(), user.publicKey.toString());
    console.log("User state initialized");
  });

  it("Deposit Funds", async () => {
    await program.methods
      .deposit(new anchor.BN(1_000_000))
      .accounts({
        userState: userStatePDA,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const userState = await program.account.userState.fetch(userStatePDA);
    console.log(`Deposit successful. New balance: ${userState.deposit}`);
    assert.equal(userState.deposit, new anchor.BN(1_000_000));
  });

  it("Place Bet", async () => {
    await program.methods
      .placeBet(new anchor.BN(500_000))
      .accounts({
        userState: userStatePDA,
        gameParams: gameParamsPDA,
        user: user.publicKey,
      })
      .signers([user])
      .rpc(); // Added user as signer

    const userState = await program.account.userState.fetch(userStatePDA);
    console.log(`Bet placed. Winnings: ${userState.winnings}`);
    assert.equal(userState.winnings, new anchor.BN(1_000_000));
  });

  it("Request Withdrawal", async () => {
    await advanceTime(60);

    [withdrawalRequestPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("withdrawal_request"), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .requestWithdrawal(new anchor.BN(1_000_000))
      .accounts({
        userState: userStatePDA,
        withdrawalRequest: withdrawalRequestPDA,
        user: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const wr = await program.account.withdrawalRequest.fetch(
      withdrawalRequestPDA
    );
    console.log("Withdrawal requested:", wr.amount);
    assert.equal(wr.amount, new anchor.BN(1_000_000));
  });

  it("Approve Withdrawal (Multisig)", async () => {
    await program.methods
      .approveWithdrawal()
      .accounts({
        withdrawalRequest: withdrawalRequestPDA,
        gameParams: gameParamsPDA,
      })
      .remainingAccounts([
        { pubkey: admin1.publicKey, isSigner: true, isWritable: false },
        { pubkey: admin2.publicKey, isSigner: true, isWritable: false },
      ])
      .signers([admin1, admin2])
      .rpc();

    const wr = await program.account.withdrawalRequest.fetch(
      withdrawalRequestPDA
    );
    console.log("Withdrawal approved:", wr.approved);
    assert.isTrue(wr.approved);
  });

  it("Execute Withdrawal", async () => {
    const initialBalance = await provider.connection.getBalance(user.publicKey);

    await program.methods
      .executeWithdrawal()
      .accounts({
        withdrawalRequest: withdrawalRequestPDA,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();

    const finalBalance = await provider.connection.getBalance(user.publicKey);
    console.log(
      `Withdrawal executed. Balance change: ${finalBalance - initialBalance}`
    );
    assert.isAbove(finalBalance, initialBalance);
  });
});
