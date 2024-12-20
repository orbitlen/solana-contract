import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OrbitLen } from "../target/types/orbit_len";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
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
import {
  getKeypairFromEnvironment,
  makeKeypairs,
} from "@solana-developers/helpers";
import { it } from "mocha";
import * as dotenv from "dotenv";
import { log } from "console";
dotenv.config();
//**
// devnet:
// admin: J1fS6hqK9fLezuGfc2BUCTAoXpKDEL5UNpeoDenscizK, userA: 51X3SJJXPgQQtAWXq8juDVcFdauuo9tNs3Ux5dLpBunY, userB: FefN4V9GnLaNvgBSonxgkEsWy4Z97e8Y35jVw6SWYYzU
// RayMint: 7CtTWGmysW2RDNFLbdU13hfBMVkV9XFkd2xX5nHGe1V7, WIFMint: Dtq1VCyJgYiRfruo91EpkPL1HNuQUCGm7dkrxt3YkXBQ
// RayBank: 2KNir3jskbH6PYEYE2ib1vsVEF5TmRwc5a4XC3v4kWTv, WIFBank: HiUPShvuUpGhyDPSkbASswGffcDYNTqczUXowivA7xqU
// RayLiquidityVaultAuthority: 8mFxPGTn7voiw24HiCjvxGrM7adgvCben97cNPvwFQGA, RayLiquidityVault: BKocEzGfB2VCAvuCY6k4Et2R4ZddDKW1gVFk5or9rK4a
// WIFLiquidityVaultAuthority: 75TLjNQQEeds9M6Cz8QLNeG7GKBEoqsL37mtyxM8ZzX7, WIFLiquidityVault: F9J1ckFN2xXSf1xgVknomYafTGoLQtykBQYCwMRNYgN8
// adminOrbitlenAccount: 8LHWQpQ3cmwGR3hFVWrqRZNr7Df2hjABmp14p3y7Yv7z
// userAOrbitlenAccount: 5vMRUWAvhojMKdWAbrWhRDykfJ6tNy88A9goBcaCfQ82
// userBOrbitlenAccount: Ao7FGEwMK46ACyzAXF5vHu3ijiUizsGEzncHDjYqFsiL
// userARay: 4szNuBQkjUBmdsDCCyDEhHvk1Bqi7HWvF7ZokL69bTsM, userAWIF: BS5ZLt5GCiu5dfaQxna1xPSA7fzoN593JK86wgify3Zn
// userBRay: HYMreKT4fYk6Zjr6YutkdSHgFiK3Wdi94bTJYhnTcL8t, userBWIF: CM3FzZ5SXh5Q3nmaArYfXaVey7VuEwdr7rbFDGkqbbtq
//  */
describe("orbit_len", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OrbitLen as Program<OrbitLen>;
  const conn = anchor.getProvider().connection;
  const LAMPORTS_PER_TOKEN = 1000000;
  console.log("conn.rpcEndpoint", conn.rpcEndpoint);
  let isLocal = conn.rpcEndpoint.includes("127.0.0.1");

  let admin, userA, userB;
  let RayMint, WIFMint, RayBank, WIFBank;
  let RayLiquidityVaultAuthority,
    RayLiquidityVault,
    WIFLiquidityVaultAuthority,
    WIFLiquidityVault;
  let adminOrbitlenAccount, userAOrbitlenAccount, userBOrbitlenAccount;
  let userARay, userAWIF, userBRay, userBWIF;

  const RayMintOnDevnet = new PublicKey(
    "7JLuhte13cbFdzphGVkcvLDW3SiXPDrU78Qvu221svho"
  );
  const WIFMintOnDevnet = new PublicKey(
    "6LYZ446PHTThBJtN7R3bCv6dkBdS83Zrm2cqtnekRQnV"
  );

  const userARayOnDevnt = new PublicKey(
    "4szNuBQkjUBmdsDCCyDEhHvk1Bqi7HWvF7ZokL69bTsM"
  );
  const userAWIFOnDevnt = new PublicKey(
    "BS5ZLt5GCiu5dfaQxna1xPSA7fzoN593JK86wgify3Zn"
  );

  const userBRayOnDevnt = new PublicKey(
    "HYMreKT4fYk6Zjr6YutkdSHgFiK3Wdi94bTJYhnTcL8t"
  );
  const userBWIFOnDevnt = new PublicKey(
    "CM3FzZ5SXh5Q3nmaArYfXaVey7VuEwdr7rbFDGkqbbtq"
  );

  const RayFeedDataPk = new PublicKey(
    "2Vw5U3KRpVZJ7BnTeNhhMHuep4Ksxh1ohQBeKbKpsG7y"
  );
  const WIFFeedDataPk = new PublicKey(
    "2ffxPFJTGza5JSoheZYmTRmweRVyJi8Wn2ka4w5ksiAe"
  );

  before(async () => {
    console.log("program.programId", program.programId);

    // initialize mints
    console.log("isLocal:", isLocal);
    if (isLocal) {
      [admin, userA, userB] = makeKeypairs(3);
      await safeAirdrop(admin.publicKey, conn);
      await safeAirdrop(userA.publicKey, conn);
      await safeAirdrop(userB.publicKey, conn);
      await delay(1000);
    } else {
      [admin, userA, userB] = [
        getKeypairFromEnvironment("DEV_1"),
        getKeypairFromEnvironment("DEV_2"),
        getKeypairFromEnvironment("DEV_3"),
      ];
    }
    console.log(
      `admin: ${admin.publicKey}, userA: ${userA.publicKey}, userB: ${userB.publicKey}`
    );

    if (isLocal) {
      RayMint = await createMint(conn, admin, admin.publicKey, undefined, 6);
      WIFMint = await createMint(conn, admin, admin.publicKey, undefined, 6);
    } else {
      RayMint = RayMintOnDevnet;
      WIFMint = WIFMintOnDevnet;
    }
    console.log(`Ray Mint: ${RayMint}, WIF Mint: ${WIFMint}`);

    // fetch bank
    [RayBank] = PublicKey.findProgramAddressSync(
      [Buffer.from("bank"), RayMint.toBuffer()],
      program.programId
    );

    [WIFBank] = PublicKey.findProgramAddressSync(
      [Buffer.from("bank"), WIFMint.toBuffer()],
      program.programId
    );

    console.log(`RayBank: ${RayBank}, WIFBank: ${WIFBank}`);

    // fetch liquidity vault
    [RayLiquidityVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault_auth"), RayBank.toBuffer()],
      program.programId
    );

    [RayLiquidityVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault"), RayBank.toBuffer()],
      program.programId
    );

    console.log(
      `RayLiquidityVaultAuthority: ${RayLiquidityVaultAuthority}, RayLiquidityVault: ${RayLiquidityVault}`
    );

    [WIFLiquidityVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault_auth"), WIFBank.toBuffer()],
      program.programId
    );

    [WIFLiquidityVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault"), WIFBank.toBuffer()],
      program.programId
    );

    console.log(
      `WIFLiquidityVaultAuthority: ${WIFLiquidityVaultAuthority}, WIFLiquidityVault: ${WIFLiquidityVault}`
    );

    // fetch orbitlen account
    [adminOrbitlenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("orbitlen_account"), admin.publicKey.toBuffer()],
      program.programId
    );

    [userAOrbitlenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("orbitlen_account"), userA.publicKey.toBuffer()],
      program.programId
    );

    [userBOrbitlenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("orbitlen_account"), userB.publicKey.toBuffer()],
      program.programId
    );

    console.log(`adminOrbitlenAccount: ${adminOrbitlenAccount}`);
    console.log(`userAOrbitlenAccount: ${userAOrbitlenAccount}`);
    console.log(`userBOrbitlenAccount: ${userBOrbitlenAccount}`);
  });

  it("initialize ATAs and mint tokens", async () => {
    // initialize associated token accounts
    userARay = (
      await getOrCreateAssociatedTokenAccount(
        conn,
        userA,
        RayMint,
        userA.publicKey
      )
    ).address;

    userAWIF = (
      await getOrCreateAssociatedTokenAccount(
        conn,
        userA,
        WIFMint,
        userA.publicKey
      )
    ).address;

    userBRay = (
      await getOrCreateAssociatedTokenAccount(
        conn,
        userB,
        RayMint,
        userB.publicKey
      )
    ).address;

    userBWIF = (
      await getOrCreateAssociatedTokenAccount(
        conn,
        userB,
        WIFMint,
        userB.publicKey
      )
    ).address;

    // mint tokens
    await mintTo(
      conn,
      userA,
      RayMint,
      userARay,
      admin,
      1000 * LAMPORTS_PER_TOKEN
    );

    await mintTo(
      conn,
      userA,
      WIFMint,
      userAWIF,
      admin,
      1000 * LAMPORTS_PER_TOKEN
    );

    await mintTo(
      conn,
      userB,
      RayMint,
      userBRay,
      admin,
      1000 * LAMPORTS_PER_TOKEN
    );

    await mintTo(
      conn,
      userB,
      WIFMint,
      userBWIF,
      admin,
      1000 * LAMPORTS_PER_TOKEN
    );
  });

  it("lending pool add bank", async () => {
    // params
    let RayBankConfig = {
      interestRateConfig: {
        optimalUtilizationRate: new anchor.BN(80),
        plateauInterestRate: new anchor.BN(10),
        maxInterestRate: new anchor.BN(50),
      },
      feedDataKey: RayFeedDataPk,
    };
    let WIFBankConfig = {
      interestRateConfig: {
        optimalUtilizationRate: new anchor.BN(80),
        plateauInterestRate: new anchor.BN(10),
        maxInterestRate: new anchor.BN(50),
      },
      feedDataKey: WIFFeedDataPk,
    };
    // accounts
    // initialize Ray vault and bank
    await program.methods
      .initialVault(RayBank)
      .accounts({
        admin: admin.publicKey,
        bankMint: RayMint,
        liquidityVaultAuthority: RayLiquidityVaultAuthority,
        liquidityVault: RayLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    await program.methods
      .lendingPoolAddBank(RayBankConfig)
      .accounts({
        admin: admin.publicKey,
        bankMint: RayMint,
        bank: RayBank,
        liquidityVaultAuthority: RayLiquidityVaultAuthority,
        liquidityVault: RayLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    // initialize WIF vault and bank
    await program.methods
      .initialVault(WIFBank)
      .accounts({
        admin: admin.publicKey,
        bankMint: WIFMint,
        liquidityVaultAuthority: WIFLiquidityVaultAuthority,
        liquidityVault: WIFLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    await program.methods
      .lendingPoolAddBank(WIFBankConfig)
      .accounts({
        admin: admin.publicKey,
        bankMint: WIFMint,
        bank: WIFBank,
        liquidityVaultAuthority: WIFLiquidityVaultAuthority,
        liquidityVault: WIFLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    await delay(1000);
    const bankInfos = await program.account.bank.fetchMultiple([
      RayBank,
      WIFBank,
    ]);
    console.log("RayBank:", JSON.stringify(bankInfos[0], null, 2));
    console.log("WIFBank:", JSON.stringify(bankInfos[1], null, 2));
  });

  it("initialize orbitlen account", async () => {
    await program.methods
      .initializeAccount()
      .accounts({
        orbitlenAccount: userAOrbitlenAccount,
        authority: userA.publicKey,
      })
      .signers([userA])
      .rpc();

    await program.methods
      .initializeAccount()
      .accounts({
        orbitlenAccount: userBOrbitlenAccount,
        authority: userB.publicKey,
      })
      .signers([userB])
      .rpc();

    await delay(1000);

    let orbitlenAccountInfos =
      await program.account.orbitlenAccount.fetchMultiple([
        userAOrbitlenAccount,
        userBOrbitlenAccount,
      ]);
    console.log(
      "userAOrbitlenAccountInfo:",
      JSON.stringify(orbitlenAccountInfos[0], null, 2)
    );
    console.log(
      "userBOrbitlenAccountInfo:",
      JSON.stringify(orbitlenAccountInfos[1], null, 2)
    );
  });

  it("lending account deposit", async () => {
    const depositAmount = new anchor.BN(50 * LAMPORTS_PER_TOKEN);

    await program.methods
      .lendingAccountDeposit(depositAmount)
      .accounts({
        orbitlenAccount: userAOrbitlenAccount,
        signer: userA.publicKey,
        bank: RayBank,
        signerTokenAccount: isLocal ? userARay : userARayOnDevnt,
        bankLiquidityVault: RayLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: RayMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([userA])
      .rpc();

    await program.methods
      .lendingAccountDeposit(depositAmount)
      .accounts({
        orbitlenAccount: userAOrbitlenAccount,
        signer: userA.publicKey,
        bank: WIFBank,
        signerTokenAccount: isLocal ? userAWIF : userAWIFOnDevnt,
        bankLiquidityVault: WIFLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: WIFMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([userA])
      .rpc();

    console.log("=== deposit ===");

    let userARayInfo = await getAccount(conn, userARay);
    console.log("userARayInfo:", userARayInfo);

    let RayLiquidityVaultInfo = await getAccount(conn, RayLiquidityVault);
    console.log("RayLiquidityVaultInfo:", RayLiquidityVaultInfo);

    await delay(1000);

    let userAOrbitlenAccountInfo = await program.account.orbitlenAccount.fetch(
      userAOrbitlenAccount
    );
    console.log(
      "userAOrbitlenAccountInfo:",
      JSON.stringify(userAOrbitlenAccountInfo, null, 2)
    );

    let RayBankInfo = await program.account.bank.fetch(RayBank);
    console.log("RayBankInfo:", JSON.stringify(RayBankInfo, null, 2));
  });

  it("lending account borrow", async () => {
    const borrowAmount = new anchor.BN(3 * LAMPORTS_PER_TOKEN);
    const borrower = userB;
    const borrowerOrbitlenAccount = userBOrbitlenAccount;
    const borrowerWIF = userBWIF;

    await program.methods
      .lendingAccountBorrow(borrowAmount)
      .accounts({
        orbitlenAccount: borrowerOrbitlenAccount,
        signer: borrower.publicKey,
        bank: WIFBank,
        destinationTokenAccount: borrowerWIF,
        bankLiquidityVaultAuthority: WIFLiquidityVaultAuthority,
        bankLiquidityVault: WIFLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: WIFMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([borrower])
      .rpc();

    console.log("=== borrow ===");
    let borrowerOrbitlenAccountInfo =
      await program.account.orbitlenAccount.fetch(borrowerOrbitlenAccount);
    console.log(
      "borrowerOrbitlenAccountInfo:",
      JSON.stringify(borrowerOrbitlenAccountInfo, null, 2)
    );
    let WIFBankInfo = await program.account.bank.fetch(WIFBank);
    console.log("WIFBankInfo:", JSON.stringify(WIFBankInfo, null, 2));
  });

  it("lending account liquidate", async () => {
    // WIF as asset、Ray as liability
    let liquidator = userA;
    const liquidatorOrbitlenAccount = userAOrbitlenAccount;
    const liquidatorWIF = isLocal ? userAWIF : userAWIFOnDevnt;
    const liquidatorRay = isLocal ? userARay : userARayOnDevnt;

    let liquidatee = userB;
    const liquidateeOrbitlenAccount = userBOrbitlenAccount;
    const liquidateeWIF = isLocal ? userBWIF : userBWIFOnDevnt;
    const liquidateeRay = isLocal ? userBRay : userBRayOnDevnt;

    // liquidator deposit 500 WIF and 500 Ray, liquidatee borrow 5 Ray and deposit 500 WIF
    const depositAmount = new anchor.BN(500 * LAMPORTS_PER_TOKEN);
    const borrowAmount = new anchor.BN(5 * LAMPORTS_PER_TOKEN);

    await program.methods
      .lendingAccountDeposit(depositAmount)
      .accounts({
        orbitlenAccount: liquidatorOrbitlenAccount,
        signer: liquidator.publicKey,
        bank: WIFBank,
        signerTokenAccount: liquidatorWIF,
        bankLiquidityVault: WIFLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: WIFMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([liquidator])
      .rpc();

    console.log(`liquidator deposit 500 WIF`);

    await program.methods
      .lendingAccountDeposit(depositAmount)
      .accounts({
        orbitlenAccount: liquidatorOrbitlenAccount,
        signer: liquidator.publicKey,
        bank: RayBank,
        signerTokenAccount: liquidatorRay,
        bankLiquidityVault: RayLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: RayMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([liquidator])
      .rpc();

    console.log(`liquidator deposit 500 Ray`);

    await program.methods
      .lendingAccountDeposit(depositAmount)
      .accounts({
        orbitlenAccount: liquidateeOrbitlenAccount,
        signer: liquidatee.publicKey,
        bank: WIFBank,
        signerTokenAccount: liquidateeWIF,
        bankLiquidityVault: WIFLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: WIFMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([liquidatee])
      .rpc();

    console.log(`liquidatee deposit 1000 WIF`);

    await program.methods
      .lendingAccountBorrow(borrowAmount)
      .accounts({
        orbitlenAccount: liquidateeOrbitlenAccount,
        signer: liquidatee.publicKey,
        bank: RayBank,
        destinationTokenAccount: liquidateeRay,
        bankLiquidityVaultAuthority: RayLiquidityVaultAuthority,
        bankLiquidityVault: RayLiquidityVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: RayMint,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([liquidatee])
      .rpc();

    console.log(`liquidatee borrow 5 Ray`);

    // // WIF / USD 2.8
    // // Ray / USD 250
    // // if pay off 2 Ray then the liquidatee would send liquidator 179 WIF
    const liquidateAmount = new anchor.BN(2 * LAMPORTS_PER_TOKEN);

    await delay(5000);
    await program.methods
      .lendingAccountLiquidate(liquidateAmount)
      .accounts({
        assetBank: WIFBank,
        liabBank: RayBank,
        liquidatorOrbitlenAccount: liquidatorOrbitlenAccount,
        signer: liquidator.publicKey,
        liquidateeOrbitlenAccount: liquidateeOrbitlenAccount,
      })
      .remainingAccounts([
        {
          pubkey: WIFFeedDataPk,
          isWritable: false,
          isSigner: false,
        },
        {
          pubkey: RayFeedDataPk,
          isWritable: false,
          isSigner: false,
        },
      ])
      .signers([liquidator])
      .rpc();

    console.log("==== liquidator ===");
    let liqInfos = await program.account.orbitlenAccount.fetchMultiple([
      liquidatorOrbitlenAccount,
      liquidateeOrbitlenAccount,
    ]);
    console.log("liquidatorInfo:", JSON.stringify(liqInfos[0], null, 2));
    console.log("liquidateeInfo:", JSON.stringify(liqInfos[1], null, 2));
    let bankInfos = await program.account.bank.fetchMultiple([
      RayBank,
      WIFBank,
    ]);
    console.log("RayBankInfo:", JSON.stringify(bankInfos[0], null, 2));
    console.log("WIFBankInfo:", JSON.stringify(bankInfos[1], null, 2));
  });
});
