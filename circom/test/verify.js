const { readFileSync } = require('fs');
const { expect } = require("chai");
const { ethers } = require('hardhat');

describe("verify", function () {
  const testVerify = (contract) => (
    async() => {
      const input = JSON.parse(readFileSync(`proofs/${contract}/solidity-args.json`).toString());
      const verifier = await ethers.getContractFactory(`Verifier`).then(f => f.deploy());
      const result = await verifier.verifyProof(...input);
      expect(result).to.be.true;
    }
  );

  it('verifies multiplier2', testVerify(`multiplier2`));
});
