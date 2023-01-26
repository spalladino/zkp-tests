const { readFileSync } = require('fs');
const { expect } = require("chai");
const { ethers } = require('hardhat');

describe("verify", function () {
  const testVerify = (contract) => (
    async() => {
      console.log(`      Inputs: ${JSON.stringify(JSON.parse(readFileSync(`inputs/${contract}.json`)))}\n      Output: ${JSON.parse(readFileSync(`proofs/${contract}/public.json`))}`);
      const input = JSON.parse(readFileSync(`proofs/${contract}/solidity-args.json`).toString());
      const verifier = await ethers.getContractFactory(`contracts/${contract}.sol:Verifier`).then(f => f.deploy());
      const result = await verifier.verifyProof(...input);
      expect(result).to.be.true;
    }
  );

  it('verifies multiplier2', testVerify(`multiplier2`));
  it.only('verifies multipliersq', testVerify(`multipliersq`));
  it('verifies advent2', testVerify(`advent2`));

  it('verifies iszero-right', testVerify(`iszeroright`));
  it('verifies iszero-wrong', testVerify(`iszerowrong`));
  it('verifies iszero-evil', testVerify(`iszeroevil`));

  it('verifies iszero-evil using iszero-wrong verifier', async () => {
    const input = JSON.parse(readFileSync(`proofs/iszeroevil/solidity-args.json`).toString());
    const verifier = await ethers.getContractFactory(`contracts/iszerowrong.sol:Verifier`).then(f => f.deploy());
    const result = await verifier.verifyProof(...input);
    expect(result).to.be.true;
  });
});
