import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OrbitLen } from "../target/types/orbit_len";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import {
  mintTo,
  createMint,
  createAssociatedTokenAccount,
  getAccount,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  transferChecked,
  getOrCreateAssociatedTokenAccount,
  createAccount,
} from "@solana/spl-token";
import { delay, safeAirdrop } from "./utils";
import { log } from "console";

describe("orbit_len", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OrbitLen as Program<OrbitLen>;
  let admin, user;
  const conn = anchor.getProvider().connection;
  let ACTMint, bank;
  let liquidityVaultAuthority,
    liquidityVault,
    insuranceVaultAuthority,
    insuranceVault;

  let orbitlenAccount, userATA;

  before(async () => {
    admin = Keypair.generate();
    user = Keypair.generate();
    await safeAirdrop(admin.publicKey, conn);
    await safeAirdrop(user.publicKey, conn);
    await delay(2000);

    console.log(`admin: ${admin.publicKey}, user: ${user.publicKey}`);

    ACTMint = await createMint(
      conn,
      admin,
      admin.publicKey,
      undefined,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    console.log(`ACTMint: ${ACTMint}`);

    bank = Keypair.generate();

    console.log(`bank: ${bank.publicKey}`);

    [liquidityVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault_auth"), bank.publicKey.toBuffer()],
      program.programId
    );

    [liquidityVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault"), bank.publicKey.toBuffer()],
      program.programId
    );

    console.log(`liquidityVault: ${liquidityVault}`);

    [insuranceVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("insurance_vault_auth"), bank.publicKey.toBuffer()],
      program.programId
    );

    [insuranceVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("insurance_vault"), bank.publicKey.toBuffer()],
      program.programId
    );

    [orbitlenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("orbitlen_account"), user.publicKey.toBuffer()],
      program.programId
    );

    console.log(`orbitlenAccount: ${orbitlenAccount}`);

    userATA = await createAccount(
      conn,
      user,
      ACTMint,
      user.publicKey,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await mintTo(
      conn,
      user,
      ACTMint,
      userATA,
      admin,
      100 * LAMPORTS_PER_SOL,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
  });

  it("lending pool add bank", async () => {
    // params
    let bankConfig = {
      assetWeightInit: new anchor.BN(100),
      assetWeightMaint: new anchor.BN(90),
      liabilityWeightInit: new anchor.BN(100),
      liabilityWeightMaint: new anchor.BN(110),
      InterestRateConfig: {
        OptimalUtilizationRate: new anchor.BN(0.8),
        PlateauInterestRate: new anchor.BN(0.1),
        MaxInterestRate: new anchor.BN(0.5),
      },
    };
    // accounts

    await program.methods
      .initialVault(bank.publicKey)
      .accounts({
        admin: admin.publicKey,
        bankMint: ACTMint,
        liquidityVaultAuthority: liquidityVaultAuthority,
        liquidityVault: liquidityVault,
        insuranceVaultAuthority: insuranceVaultAuthority,
        insuranceVault: insuranceVault,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    await program.methods
      .lendingPoolAddBank(bankConfig)
      .accounts({
        admin: admin.publicKey,
        bankMint: ACTMint,
        bank: bank.publicKey,
        liquidityVaultAuthority: liquidityVaultAuthority,
        liquidityVault: liquidityVault,
        insuranceVaultAuthority: insuranceVaultAuthority,
        insuranceVault: insuranceVault,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([admin, bank])
      .rpc();

    const bankInfo = await program.account.bank.all();
    console.log(bankInfo);
  });

  it("lending account deposit", async () => {
    const depositAmount = new anchor.BN(5 * LAMPORTS_PER_SOL);
    let rm = [
      {
        pubkey: ACTMint,
        isWritable: false,
        isSigner: false,
      },
    ];

    await program.methods
      .initializeAccount()
      .accounts({
        orbitlenAccount: orbitlenAccount,
        authority: user.publicKey,
      })
      .signers([user])
      .rpc();

    console.log("initialize account");

    await program.methods
      .lendingAccountDeposit(depositAmount)
      .accounts({
        orbitlenAccount: orbitlenAccount,
        signer: user.publicKey,
        bank: bank.publicKey,
        signerTokenAccount: userATA,
        bankLiquidityVault: liquidityVault,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .remainingAccounts(rm)
      .signers([user])
      .rpc();

    console.log("deposit done");

    let userInfo = await getAccount(
      conn,
      userATA,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    console.log("user info:", userInfo);
  });

  it("lending account borrow", async () => {
    let borrower = Keypair.generate();
    await safeAirdrop(borrower.publicKey, conn);
    await delay(2000);

    const borrowAmount = new anchor.BN(3 * LAMPORTS_PER_SOL);
    let rm = [
      {
        pubkey: ACTMint,
        isWritable: false,
        isSigner: false,
      },
    ];

    let [borrowerOrbitlenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("orbitlen_account"), borrower.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializeAccount()
      .accounts({
        orbitlenAccount: borrowerOrbitlenAccount,
        authority: borrower.publicKey,
      })
      .signers([borrower])
      .rpc();

    console.log("initialize account");

    let borrowerATA = await createAccount(
      conn,
      borrower,
      ACTMint,
      borrower.publicKey,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    await program.methods
      .lendingAccountBorrow(borrowAmount)
      .accounts({
        orbitlenAccount: borrowerOrbitlenAccount,
        signer: borrower.publicKey,
        bank: bank.publicKey,
        destinationTokenAccount: borrowerATA,
        bankLiquidityVaultAuthority: liquidityVaultAuthority,
        bankLiquidityVault: liquidityVault,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .remainingAccounts(rm)
      .signers([borrower])
      .rpc();

    console.log("borrow done");

    let userInfo = await getAccount(
      conn,
      borrowerATA,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    console.log("user info:", userInfo);

    const orbitlenAccountAll = await program.account.orbitlenAccount.all();
    console.log("orbitlenAccountAll", JSON.stringify(orbitlenAccountAll, null, 2));
  });
});
