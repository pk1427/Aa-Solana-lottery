import React from "react";
import ConnectWallet from "./components/ConnectWallet";
import InitializeLottery from "./components/InitializeLottery";

function App() {
  return (
    <div className="p-6 text-center">
      <h1 className="text-2xl font-bold mb-4">🎟 Lottery DApp</h1>
      <ConnectWallet />
      <InitializeLottery />
    </div>
  );
}

export default App;
