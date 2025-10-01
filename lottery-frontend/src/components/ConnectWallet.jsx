import React from "react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

// WalletMultiButton comes with prebuilt styles (connect/disconnect)
const ConnectWallet = () => {
  return (
    <div className="flex justify-center my-4">
      <WalletMultiButton />
    </div>
  );
};

export default ConnectWallet;

