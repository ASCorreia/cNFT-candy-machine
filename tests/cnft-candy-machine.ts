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

describe("cnft-candy-machine", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CnftCandyMachine as Program<CnftCandyMachine>;

  const wallet = provider.wallet as anchor.Wallet;

  const config = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("config"), wallet.publicKey.toBuffer()], program.programId);

  const mint_authority = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("authority"), wallet.publicKey.toBuffer()], program.programId);

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
  console.log('treeConfigPublicKey', treeConfigPublicKey.toBase58())

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
});
