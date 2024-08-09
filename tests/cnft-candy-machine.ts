import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CnftCandyMachine } from "../target/types/cnft_candy_machine";
import {
  AccountMeta,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  clusterApiUrl,
  sendAndConfirmTransaction,
} from "@solana/web3.js"
import {
  ConcurrentMerkleTreeAccount,
  SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
  ValidDepthSizePair,
  createAllocTreeIx,
} from "@solana/spl-account-compression"
import { SPL_ACCOUNT_COMPRESSION_PROGRAM_ID as BUBBLEGUM_PROGRAM_ID, SPL_NOOP_PROGRAM_ID, findTreeConfigPda } from "@metaplex-foundation/mpl-bubblegum"
import {
  Metaplex,
  keypairIdentity,
  CreateNftOutput,
} from "@metaplex-foundation/js"
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { use } from "chai";

describe("cnft-candy-machine", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CnftCandyMachine as Program<CnftCandyMachine>;

  const wallet = provider.wallet as anchor.Wallet;

  const config = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("config"), wallet.publicKey.toBuffer()], program.programId);

  const mint_authority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("authority"), wallet.publicKey.toBuffer()], program.programId);

  const allowedOne = Keypair.generate();
  const allowedTwo = Keypair.generate();
  const allowedThree = Keypair.generate();

  const maxDepthSizePair: ValidDepthSizePair = {
    maxDepth: 14,
    maxBufferSize: 64,
  }
  const canopyDepth = maxDepthSizePair.maxDepth - 5;

  const emptyMerkleTree = anchor.web3.Keypair.generate();
  console.log(`Merke tree: ${emptyMerkleTree.publicKey.toBase58()}`);
  const umi = createUmi(provider.connection.rpcEndpoint);
  const treeConfig = findTreeConfigPda(
    umi,
    {
      merkleTree: emptyMerkleTree.publicKey.toBase58(),
    }
  )[0]

  const treeConfigPublicKey = new anchor.web3.PublicKey(treeConfig);
  console.log('treeConfigPublicKey', treeConfigPublicKey.toBase58());

  const confirm = async (signature: string): Promise<string> => {
    const block = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  it("Airdrop SOl to wallet", async () => {
    const tx = await provider.connection.requestAirdrop(allowedOne.publicKey, 1000000000).then(confirm);
    console.log("Airdrop done: ", tx);
  });

  it("Create Config Account and Initialize Merkle Tree", async () => {
    // Add your test here.
    const allocTreeIx = await createAllocTreeIx(
      provider.connection,
      emptyMerkleTree.publicKey,
      provider.publicKey,
      maxDepthSizePair,
      canopyDepth
    );

    const signature = await sendAndConfirmTransaction(provider.connection, new Transaction().add(allocTreeIx), [wallet.payer, emptyMerkleTree]);

    console.log("Allocated tree", signature);

    const tx = await program.methods.initialize(100, 14, 64)
    .accounts({
      authority: provider.wallet.publicKey,
      merkleTree: emptyMerkleTree.publicKey,
      treeConfig: treeConfigPublicKey,
    })
    .rpc();
    console.log("Config account created");
    console.log("Merkle tree initialized");
    console.log("Your transaction signature", tx);
  });

  it("Add user to allow list", async () => {
  const tx = await program.methods.addAllowList(88)
    .accounts({
      authority: provider.wallet.publicKey,
      user: allowedOne.publicKey,
    })
    .rpc();

    console.log("\nUser added to allow list: ", allowedOne.publicKey.toBase58());

    const allowList = await program.account.config.fetch(config[0]);
    console.log("\nAllow list:");
    allowList.allowList.forEach((user) => console.log("User: ", user.user.toBase58(),"\tAmount: ", user.amount));
  });

  it("Add user to allow list", async () => {
  const tx = await program.methods.addAllowList(10)
    .accounts({
      authority: provider.wallet.publicKey,
      user: allowedTwo.publicKey,
    })
    .rpc();

    console.log("\nUser added to allow list: ", allowedTwo.publicKey.toBase58());

    const allowList = await program.account.config.fetch(config[0]);
    console.log("\nAllow list:");
    allowList.allowList.forEach((user) => console.log("User: ", user.user.toBase58(),"\tAmount: ", user.amount));
  });

  it("Add user to allow list", async () => {
  const tx = await program.methods.addAllowList(50)
    .accounts({
      authority: provider.wallet.publicKey,
      user: allowedThree.publicKey,
    })
    .rpc();

    console.log("\nUser added to allow list: ", allowedThree.publicKey.toBase58());

    const allowList = await program.account.config.fetch(config[0]);
    console.log("\nAllow list:");
    allowList.allowList.forEach((user) => console.log("User: ", user.user.toBase58(),"\tAmount: ", user.amount));
  });

  it("Mint cNFT", async() => {
    console.log("\nMinting cNFT for user: ", allowedOne.publicKey.toBase58());
    console.log("User allowed amount: ", await program.account.config.fetch(config[0]).then((config) => config.allowList.find((user) => user.user.equals(allowedOne.publicKey))?.amount));

    const tx = await program.methods.mint("Test", "TST", "https://arweave.net/123")
    .accounts({
      user: allowedOne.publicKey,
      authority: provider.wallet.publicKey,
      treeConfig: treeConfigPublicKey,
      leafOwner: allowedOne.publicKey,
      merkleTree: emptyMerkleTree.publicKey,
    })
    .signers([allowedOne])
    .rpc();

    console.log(`\ncNFT minted for user: ${allowedOne.publicKey.toBase58()} with tx: ${tx}`);
    console.log("User allowed amount: ", await program.account.config.fetch(config[0]).then((config) => config.allowList.find((user) => user.user.equals(allowedOne.publicKey))?.amount));
  })
});
