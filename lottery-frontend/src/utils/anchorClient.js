import { Connection, PublicKey } from "@solana/web3.js";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import idl from "../idl/lottery_solana.json";

// ✅ Use the correct Program ID from your IDL
const programID = new PublicKey("ATNbt1MAreJosSPKpuDjmDCGSJ3WR4QkVmJQwid236kG");

const network = "https://api.devnet.solana.com";
const opts = {
  preflightCommitment: "processed",
};

export const getProgram = (wallet) => {
  if (!wallet) throw new Error("Wallet not connected");

  const connection = new Connection(network, opts.preflightCommitment);
  const provider = new AnchorProvider(connection, wallet, opts);

  return new Program(idl, programID, provider);
};

export { programID };