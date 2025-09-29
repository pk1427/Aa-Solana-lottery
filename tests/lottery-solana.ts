import * as anchor from "@project-serum/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.LotterySolana;

  // Demo participants
  const participants = [
    Keypair.generate(),
    Keypair.generate(),
    Keypair.generate(),
    Keypair.generate(),
  ];

  // Lottery PDA (replace with your PDA)
  const lotteryId = 1;
  const [lotteryPDA] = await PublicKey.findProgramAddress(
    [Buffer.from("lottery"), provider.wallet.publicKey.toBuffer(), Buffer.from([lotteryId])],
    program.programId
  );

  // Simulate buy_lottery
  console.log("Simulating participants buying lottery...");
  for (let p of participants) {
    await program.methods
      .buyLottery()
      .accounts({
        lotteryAccount: lotteryPDA,
        buyer: p.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([p])
      .rpc();

    console.log("Participant bought lottery:", p.publicKey.toBase58());
  }

  // ----------------------
  // Mock randomness (for testing)
  // ----------------------
  const randomIndex = Math.floor(Math.random() * participants.length);
  console.log("Random index for winner:", randomIndex);

  // Call pick_winner function with mocked randomness PDA
  // (You can update your program temporarily to accept a randomIndex for testing)
  await program.methods
    .pickWinner()
    .accounts({
      lotteryAccount: lotteryPDA,
      randomnessAccountData: provider.wallet.publicKey, // use any pubkey for test
      authority: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  const lotteryState = await program.account.lotteryAccount.fetch(lotteryPDA);
  console.log("Winner picked:", lotteryState.winner?.toBase58());
}

main().catch(console.error);
