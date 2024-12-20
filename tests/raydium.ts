import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OrbitLen } from "../target/types/orbit_len";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Transaction,
  sendAndConfirmTransaction,
  ComputeBudgetProgram,
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
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { delay, safeAirdrop } from "./utils";
import {
  getKeypairFromEnvironment,
  makeKeypairs,
} from "@solana-developers/helpers";
import { it } from "mocha";
import {
  MARKET_STATE_LAYOUT_V3,
  AMM_V4,
  OPEN_BOOK_PROGRAM,
  FEE_DESTINATION_ID,
  DEVNET_PROGRAM_ID,
  Raydium,
} from "@raydium-io/raydium-sdk-v2";
import { initSdk, txVersion } from "./raydium_config";
import * as dotenv from "dotenv";
import { log } from "console";
dotenv.config();

/**
     amm pool created! txId:  25yZwefx4mM4FDEveFqQmn6LS4MV3LXqwRxEeEuexjihvrLNUZ2QdR3UV3f9JypBpdRGJv5Q1RFBTdauPsbUJF5A , poolKeys: {
        programId: 'HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8',
        ammId: 'BD2KBLSZxQ6eHGDB9yjWQx6VXmon5z83WgQY7JH8uZGq',
        ammAuthority: 'DbQqP6ehDYmeYjcBaMRuA8tAJY1EjDUz9DpwSLjaQqfC',
        ammOpenOrders: 'FsAyJ1XHaCdP7FwRpcwqRGKSzYv6VSAfb593YRKP1boK',
        lpMint: '2N1FcsGEhQBcwJBzY5fhAwjHTS4e1FYdNyPNFfgKabp1',
        coinMint: '7JLuhte13cbFdzphGVkcvLDW3SiXPDrU78Qvu221svho',
        pcMint: 'DceACY73GHpkFWLDn3cfy9QBUgDWH4SXRcFJ4pGUPi3A',
        coinVault: 'G3XbsxY8v7xJ4dWwYTPM1p5yTBuXviMzUfgWz4ToF36S',
        pcVault: 'CsWy5GGeEDc3MTd79m7YZCe59UqFuFEmkh6eKeRhN2hh',
        withdrawQueue: '33u5UAC4vxPc9dGxKZeQ3zPLkGVLw1RRbW3dBi9N5E9e',
        eventQueue: 'ALKDo9po7bpgfS1PPRYaDExxMcchJcLv9LyR1G6B5WHM',

        ammTargetOrders: '52yQ6nTUvHaYYdNzPRhJCojb2QNJRJLYGKJ6eEFCoqnu',
        poolTempLp: '3gSzQz76gFZi8AP7JNCwTkeQXE3B3hpTsWNvDYsUMjpH',
        marketProgramId: 'EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj',
        marketId: 'EPUYeveeorDAX6qeG8T2uva7CpNCfDAsb8RpeTeLYp8Y',
        ammConfigId: '8QN9yfKqWDoKjvZmqFsgCzAqwZBQuzVVnC388dN5RCPo',
        feeDestinationId: '3XMrhbv989VxAMi3DErLV9eJht1pHppW5LbKxe9fkEFR'
        }
*/

/**
 * market info:  {
  marketId: 'Aryg18V2YU583cCdCWfFGcPGQwGt2xm9pFipkrvkWkGY',
  requestQueue: '8hujDiEiKbfdpo1oicVtzE95UonB2MJX3L6ZHs11EZB2',
  eventQueue: 'HHHaZkzu3sxH5Cu397Lg7XMTvFeyYjxFKTnvL4c2NQRD',
  bids: '6EVyZRRN8bJssAAWfWgJVmquWQK1j8XXQeB8Qzh1vQ5M',
  asks: 'HEsJhZJQD5NHFHVnHuq9Tn3WodgKBf5vxu2Q2n3LHWkB',
  baseVault: '2XLJ75K79rdN8NDBAVGa2fsi2TGDN4ceJnBfuxrdf3Lp',
  quoteVault: '4USZn46sz6issWSmbfEgeVbSNi6GCZg5cynjaLJWccjn',
  baseMint: '7JLuhte13cbFdzphGVkcvLDW3SiXPDrU78Qvu221svho',
  quoteMin: 'DceACY73GHpkFWLDn3cfy9QBUgDWH4SXRcFJ4pGUPi3A'
}
 */

const globalInfo = {
  ammProgram: new PublicKey("HWy1jotHpo6UqeQxx49dpYYdQB8wj9Qk9MdxwjLvDHB8"),
  ammCreateFeeDestination: new PublicKey(
    "3XMrhbv989VxAMi3DErLV9eJht1pHppW5LbKxe9fkEFR"
  ),
  ammId: new PublicKey("BD2KBLSZxQ6eHGDB9yjWQx6VXmon5z83WgQY7JH8uZGq"),
  ammAuthority: new PublicKey("DbQqP6ehDYmeYjcBaMRuA8tAJY1EjDUz9DpwSLjaQqfC"),
  ammOpenOrders: new PublicKey("FsAyJ1XHaCdP7FwRpcwqRGKSzYv6VSAfb593YRKP1boK"),
  lpMint: new PublicKey("2N1FcsGEhQBcwJBzY5fhAwjHTS4e1FYdNyPNFfgKabp1"),
  coinMint: new PublicKey("7JLuhte13cbFdzphGVkcvLDW3SiXPDrU78Qvu221svho"),
  pcMint: new PublicKey("DceACY73GHpkFWLDn3cfy9QBUgDWH4SXRcFJ4pGUPi3A"),
  coinVault: new PublicKey("G3XbsxY8v7xJ4dWwYTPM1p5yTBuXviMzUfgWz4ToF36S"),
  pcVault: new PublicKey("CsWy5GGeEDc3MTd79m7YZCe59UqFuFEmkh6eKeRhN2hh"),
  withdrawQueue: new PublicKey("33u5UAC4vxPc9dGxKZeQ3zPLkGVLw1RRbW3dBi9N5E9e"),
  eventQueue: new PublicKey("ALKDo9po7bpgfS1PPRYaDExxMcchJcLv9LyR1G6B5WHM"),
  ammTargetOrders: new PublicKey(
    "52yQ6nTUvHaYYdNzPRhJCojb2QNJRJLYGKJ6eEFCoqnu"
  ),
  poolTempLp: new PublicKey("3gSzQz76gFZi8AP7JNCwTkeQXE3B3hpTsWNvDYsUMjpH"),
  marketProgram: new PublicKey("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj"),
  marketId: new PublicKey("EPUYeveeorDAX6qeG8T2uva7CpNCfDAsb8RpeTeLYp8Y"),
  ammConfigId: new PublicKey("8QN9yfKqWDoKjvZmqFsgCzAqwZBQuzVVnC388dN5RCPo"),
  feeDestinationId: new PublicKey(
    "3XMrhbv989VxAMi3DErLV9eJht1pHppW5LbKxe9fkEFR"
  ),
  RayFeedDataPk: new PublicKey("2Vw5U3KRpVZJ7BnTeNhhMHuep4Ksxh1ohQBeKbKpsG7y"),
};

describe("raydium", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.OrbitLen as Program<OrbitLen>;
  const conn = anchor.getProvider().connection;
  const LAMPORTS_PER_TOKEN = 1 * 10 ** 6;

  let raydium: Raydium;
  let admin, userA, userB;
  let BN = anchor.BN;

  let RayBank;
  let RayLiquidityVaultAuthority;
  let RayLiquidityVault;
  let userAOrbitlenAccount;
  let userARay, userAUSDC, userALP;

  before(async () => {
    raydium = await initSdk();
    [admin, userA, userB] = [
      getKeypairFromEnvironment("DEV_1"),
      getKeypairFromEnvironment("DEV_2"),
      getKeypairFromEnvironment("DEV_3"),
    ];

    [RayBank] = PublicKey.findProgramAddressSync(
      [Buffer.from("bank"), globalInfo.coinMint.toBuffer()],
      program.programId
    );

    [RayLiquidityVaultAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault_auth"), RayBank.toBuffer()],
      program.programId
    );

    [RayLiquidityVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("liquidity_vault"), RayBank.toBuffer()],
      program.programId
    );

    [userAOrbitlenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("orbitlen_account"), userA.publicKey.toBuffer()],
      program.programId
    );

    userARay = getAssociatedTokenAddressSync(
      globalInfo.coinMint,
      userA.publicKey
    );

    userAUSDC = getAssociatedTokenAddressSync(
      globalInfo.pcMint,
      userA.publicKey
    );

    userALP = getAssociatedTokenAddressSync(globalInfo.lpMint, userA.publicKey);
  });

  it("create market", async () => {
    let RAYMint = await createMint(conn, admin, admin.publicKey, undefined, 6);
    let USDCMint = await createMint(conn, admin, admin.publicKey, undefined, 9);

    // console.log("RAYMint", RAYMint.toString());
    // console.log("USDCMint", USDCMint.toString());

    // create market doesn't support token 2022
    const { execute, extInfo, transactions } = await raydium.marketV2.create({
      baseInfo: {
        mint: RAYMint,
        decimals: 6,
      },
      quoteInfo: {
        mint: USDCMint,
        decimals: 9,
      },
      lotSize: 1,
      tickSize: 0.01,
      dexProgramId: DEVNET_PROGRAM_ID.OPENBOOK_MARKET,
      txVersion,
    });

    console.log(
      `create market total ${transactions.length} txs, market info: `,
      Object.keys(extInfo.address).reduce(
        (acc, cur) => ({
          ...acc,
          [cur]:
            extInfo.address[cur as keyof typeof extInfo.address].toBase58(),
        }),
        {}
      )
    );

    const txIds = await execute({
      // set sequentially to true means tx will be sent when previous one confirmed
      sequentially: true,
    });

    console.log("note: create market does not support token 2022");
    console.log("create market txIds:", txIds);
  });

  it("create amm pool", async () => {
    // if you are confirmed your market info, don't have to get market info from rpc below
    const marketBufferInfo = await raydium.connection.getAccountInfo(
      globalInfo.marketId
    );
    const { baseMint, quoteMint } = MARKET_STATE_LAYOUT_V3.decode(
      marketBufferInfo!.data
    );
    console.log(
      `baseMint: ${baseMint.toBase58()}, quoteMint: ${quoteMint.toBase58()}`
    );

    // amm pool doesn't support token 2022
    const baseMintInfo = await raydium.token.getTokenInfo(baseMint);
    const quoteMintInfo = await raydium.token.getTokenInfo(quoteMint);
    const baseAmount = new BN(4 * 10 ** 9);
    const quoteAmount = new BN(4 * 10 ** 9);

    if (
      baseMintInfo.programId !== TOKEN_PROGRAM_ID.toBase58() ||
      quoteMintInfo.programId !== TOKEN_PROGRAM_ID.toBase58()
    ) {
      throw new Error(
        "amm pools with openbook market only support TOKEN_PROGRAM_ID mints, if you want to create pool with token-2022, please create cpmm pool instead"
      );
    }

    if (
      baseAmount
        .mul(quoteAmount)
        .lte(new BN(1).mul(new BN(10 ** baseMintInfo.decimals)).pow(new BN(2)))
    ) {
      throw new Error(
        "initial liquidity too low, try adding more baseAmount/quoteAmount"
      );
    }

    // await mintTo(
    //   conn,
    //   admin,
    //   RAYMint,
    //   adminRayATA,
    //   admin,
    //   baseAmount.toNumber()
    // );

    // await mintTo(
    //   conn,
    //   admin,
    //   USDCMint,
    //   adminUSDCATA,
    //   admin,
    //   quoteAmount.toNumber()
    // );

    const { execute, extInfo } = await raydium.liquidity.createPoolV4({
      programId: DEVNET_PROGRAM_ID.AmmV4,
      marketInfo: {
        marketId: globalInfo.marketId,
        programId: DEVNET_PROGRAM_ID.OPENBOOK_MARKET,
      },
      baseMintInfo: {
        mint: baseMint,
        decimals: baseMintInfo.decimals,
      },
      quoteMintInfo: {
        mint: quoteMint,
        decimals: quoteMintInfo.decimals,
      },
      baseAmount,
      quoteAmount,
      startTime: new BN(0),
      ownerInfo: {
        useSOLBalance: false,
      },
      associatedOnly: false,
      txVersion,
      feeDestinationId: DEVNET_PROGRAM_ID.FEE_DESTINATION_ID,
    });

    // don't want to wait confirm, set sendAndConfirm to false or don't pass any params to execute
    const { txId } = await execute({ sendAndConfirm: true });
    console.log(
      "amm pool created! txId: ",
      txId,
      ", poolKeys:",
      Object.keys(extInfo.address).reduce(
        (acc, cur) => ({
          ...acc,
          [cur]:
            extInfo.address[cur as keyof typeof extInfo.address].toBase58(),
        }),
        {}
      )
    );
  });

  it("deposit", async () => {
    const depositAmount = new BN(50 * LAMPORTS_PER_TOKEN);
    // await mintTo(
    //   conn,
    //   userA,
    //   globalInfo.coinMint,
    //   userARay,
    //   admin,
    //   depositAmount.toNumber()
    // );

    // await program.methods
    //   .lendingAccountDeposit(depositAmount)
    //   .accounts({
    //     orbitlenAccount: userAOrbitlenAccount,
    //     signer: userA.publicKey,
    //     bank: RayBank,
    //     signerTokenAccount: userARay,
    //     bankLiquidityVault: RayLiquidityVault,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //   })
    //   .remainingAccounts([
    //     {
    //       pubkey: globalInfo.coinMint,
    //       isWritable: false,
    //       isSigner: false,
    //     },
    //   ])
    //   .signers([userA])
    //   .rpc();

    // userALP = await getOrCreateAssociatedTokenAccount(
    //   conn,
    //   userA,
    //   globalInfo.lpMint,
    //   userA.publicKey
    // );

    // check balance in coin、pc、lp, ensure sufficient coin and pc
    let userAUSDCInfo = await getAccount(conn, userAUSDC);
    console.log("userAUSDCInfo", userAUSDCInfo);

    let userARayInfo = await getAccount(conn, userARay);
    console.log("userARayInfo", userARayInfo);

    let userALPInfo = await getAccount(conn, userALP);
    console.log("userALPInfo", userALPInfo);

    let tx = await program.methods
      .raydiumDeposit(
        new BN(depositAmount), // coinAmount
        new BN(depositAmount) // pcAmount
      )
      .accounts({
        ammProgram: globalInfo.ammProgram,
        amm: globalInfo.ammId,
        ammAuthority: globalInfo.ammAuthority,
        ammOpenOrders: globalInfo.ammOpenOrders,
        ammTargetOrders: globalInfo.ammTargetOrders,
        ammLpMint: globalInfo.lpMint,
        ammCoinVault: globalInfo.coinVault,
        ammPcVault: globalInfo.pcVault,
        market: globalInfo.marketId,
        marketEventQueue: globalInfo.eventQueue,
        userTokenCoin: userARay,
        userTokenPc: userAUSDC,
        userTokenLp: userALP.address,
        orbitlenAccount: userAOrbitlenAccount,
        coinMint: globalInfo.coinMint,
        coinBank: RayBank,
        coinBankLiquidityVault: RayLiquidityVault,
        coinBankLiquidityVaultAuthority: RayLiquidityVaultAuthority,
        userOwner: userA.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
      ])
      .signers([userA])
      .rpc();

    await delay(3000);
    console.log("tx", tx);

    let userAOrbitlenAccountInfo = await program.account.orbitlenAccount.fetch(
      userAOrbitlenAccount
    );

    console.log(
      "userAOrbitlenAccountInfo",
      JSON.stringify(userAOrbitlenAccountInfo, null, 2)
    );

    let RayBankInfo = await program.account.bank.fetch(RayBank);
    console.log("RayBankInfo", JSON.stringify(RayBankInfo, null, 2));
  });
});
