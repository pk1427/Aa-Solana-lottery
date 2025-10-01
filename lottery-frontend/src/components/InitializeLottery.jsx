import React, { useState } from "react";
import { useAnchorWallet } from "@solana/wallet-adapter-react";
import { getProgram } from "../utils/anchorClient";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";

const InitializeLottery = () => {
  const wallet = useAnchorWallet();
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);

  // Form state
  const [id, setId] = useState(1);
  const [entryPrice, setEntryPrice] = useState(0.1); // SOL
  const [goalAmount, setGoalAmount] = useState(10); // SOL
  const [startTime, setStartTime] = useState(""); // ISO timestamp
  const [endTime, setEndTime] = useState(""); // ISO timestamp

  const handleInitialize = async () => {
    if (!wallet) {
      setStatus("⚠️ Wallet not connected");
      return;
    }

    try {
      setLoading(true);
      setStatus("🔄 Initializing lottery...");

      const program = getProgram(wallet);

      // Convert SOL to lamports
      const entryLamports = entryPrice * 1_000_000_000;
      const goalLamports = goalAmount * 1_000_000_000;

      // Parse timestamps or use defaults
      const startTimestamp = startTime 
        ? Math.floor(new Date(startTime).getTime() / 1000)
        : Math.floor(Date.now() / 1000); // Start now

      const endTimestamp = endTime
        ? Math.floor(new Date(endTime).getTime() / 1000)
        : Math.floor(Date.now() / 1000) + 86400; // End in 24 hours

      // Derive PDAs
      const [lotteryPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("lottery"),
          wallet.publicKey.toBuffer(),
          new BN(id).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [registryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("registry"), wallet.publicKey.toBuffer()],
        program.programId
      );

      // Call initialize instruction
      const tx = await program.methods
        .initialize(
          new BN(id),
          new BN(entryLamports),
          new BN(goalLamports),
          new BN(startTimestamp),
          new BN(endTimestamp)
        )
        .accounts({
          lotteryAccount: lotteryPda,
          authority: wallet.publicKey,
          treasury: wallet.publicKey, // Using wallet as treasury
          registry: registryPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      setStatus(`✅ Lottery ${id} initialized! Tx: ${tx.slice(0, 8)}...`);
      console.log("Transaction signature:", tx);
    } catch (err) {
      console.error("Error initializing lottery:", err);
      setStatus(`❌ Error: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 border rounded-lg shadow-lg mt-6 bg-white">
      <h2 className="text-2xl font-bold mb-4 text-gray-800">
        🎯 Initialize New Lottery
      </h2>

      <div className="flex flex-col gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Lottery ID
          </label>
          <input
            type="number"
            placeholder="Enter unique lottery ID"
            value={id}
            onChange={(e) => setId(Number(e.target.value))}
            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Entry Price (SOL)
          </label>
          <input
            type="number"
            step="0.01"
            placeholder="e.g., 0.1"
            value={entryPrice}
            onChange={(e) => setEntryPrice(Number(e.target.value))}
            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Goal Amount (SOL)
          </label>
          <input
            type="number"
            step="0.1"
            placeholder="e.g., 10"
            value={goalAmount}
            onChange={(e) => setGoalAmount(Number(e.target.value))}
            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Start Time (optional, leave empty for "now")
          </label>
          <input
            type="datetime-local"
            value={startTime}
            onChange={(e) => setStartTime(e.target.value)}
            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            End Time (optional, default: 24 hours from start)
          </label>
          <input
            type="datetime-local"
            value={endTime}
            onChange={(e) => setEndTime(e.target.value)}
            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-green-500 focus:border-transparent"
          />
        </div>

        <button
          onClick={handleInitialize}
          disabled={loading || !wallet}
          className={`w-full py-3 px-4 rounded font-semibold text-white transition-colors ${
            loading || !wallet
              ? "bg-gray-400 cursor-not-allowed"
              : "bg-green-600 hover:bg-green-700"
          }`}
        >
          {loading ? "Initializing..." : "Initialize Lottery"}
        </button>
      </div>

      {status && (
        <div className={`mt-4 p-3 rounded ${
          status.includes("✅") ? "bg-green-100 text-green-800" :
          status.includes("❌") ? "bg-red-100 text-red-800" :
          "bg-blue-100 text-blue-800"
        }`}>
          {status}
        </div>
      )}
    </div>
  );
};

export default InitializeLottery;