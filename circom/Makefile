all: multiplier2 advent2

.PHONY: contracts test

CIRCUITS := multiplier2 advent2 iszeroright iszerowrong iszeroevil

# Build verifying contracts
contracts: $(foreach circuit,$(CIRCUITS),contracts/$(circuit).sol)

# Build circuit, create witness, create proof, and verify it
multiplier2: verify/multiplier2
advent2: verify/advent2
iszeroright: verify/iszeroright
iszerowrong: verify/iszerowrong
iszeroevil: verify/iszeroevil

# Do not delete intermediate files
.SECONDARY:

# Builds a circuit
build/%.r1cs: circuits/%.circom
	@echo Building $@...
	mkdir -p build/
	circom $< --r1cs --wasm --sym --c -l ../node_modules/circomlib/circuits -o build/
	yarn -s snarkjs r1cs export json build/$*.r1cs build/$*.r1cs.json
	@echo 

# Creates a witness out of the circuit and input
inputs/%.wtns: build/%.r1cs inputs/%.json
	@echo Generating witness for $@...
	@echo -n "Input: "
	@jq -c '.' inputs/$*.json
	node build/$*_js/generate_witness.js \
		build/$*_js/$*.wasm \
		inputs/$*.json \
		inputs/$*.wtns
	yarn -s snarkjs wtns export json inputs/$*.wtns inputs/$*.wtns.json
	@echo

# General phase 1 ceremony of powers of tau (trusted setup)
proofs/pot12_0001.ptau:
	@echo Powers of tau ceremony phase 1...
	mkdir -p proofs
	yarn -s snarkjs powersoftau new bn128 12 proofs/pot12_0000.ptau -v
	yarn -s snarkjs powersoftau contribute proofs/pot12_0000.ptau proofs/pot12_0001.ptau --name="First contribution"
	@echo

# Circuit-specific phase 2 ceremony of powers of tau (trusted setup)
proofs/%/verification_key.json: proofs/pot12_0001.ptau build/%.r1cs
	@echo Phase two of the ceremony...
	mkdir -p proofs/$*
	yarn -s snarkjs powersoftau prepare phase2 proofs/pot12_0001.ptau proofs/$*/pot12.ptau
	yarn -s snarkjs groth16 setup build/$*.r1cs proofs/$*/pot12.ptau proofs/$*/key_0000.zkey
	yarn -s snarkjs zkey contribute proofs/$*/key_0000.zkey proofs/$*/key_0001.zkey --name="First contribution"
	yarn -s snarkjs zkey export verificationkey proofs/$*/key_0001.zkey proofs/$*/verification_key.json
	@echo

# Use proving key of iszerowrong when creating contract for iszeroevil
contracts/iszeroevil.sol: proofs/iszerowrong/verification_key.json
	@echo Generating verifier contract...
	mkdir -p contracts/
	yarn -s snarkjs zkey export solidityverifier proofs/iszerowrong/key_0001.zkey contracts/iszeroevil.sol

# Create a verifier contract out of a verification key
contracts/%.sol: proofs/%/verification_key.json
	@echo Generating verifier contract...
	mkdir -p contracts/
	yarn -s snarkjs zkey export solidityverifier proofs/$*/key_0001.zkey contracts/$*.sol

# Use proving key of iszerowrong when creating proof for iszeroevil
proofs/iszeroevil/proof.json: proofs/iszerowrong/verification_key.json inputs/iszeroevil.wtns
	@echo Generating proof...
	yarn -s snarkjs groth16 prove proofs/iszerowrong/key_0001.zkey inputs/iszeroevil.wtns proofs/iszeroevil/proof.json proofs/iszeroevil/public.json
	@echo Output:
	@jq -c '.' proofs/iszeroevil/public.json
	@echo

# Create proof for a given witness, given the verification key generated in phase 2
proofs/%/proof.json: proofs/%/verification_key.json inputs/%.wtns
	@echo Generating proof...
	yarn -s snarkjs groth16 prove proofs/$*/key_0001.zkey inputs/$*.wtns proofs/$*/proof.json proofs/$*/public.json
	@echo Output:
	@jq -c '.' proofs/$*/public.json
	@echo

# Create proof to be fed into verifier contract
proofs/%/solidity-args.json: proofs/%/proof.json
	echo '[' > $@
	yarn run -s snarkjs zkey export soliditycalldata proofs/$*/public.json proofs/$*/proof.json >> $@
	echo ']' >> $@

# Use iszerowrong verification key for iszeroevil
verify/iszeroevil: proofs/iszeroevil/proof.json
	@echo Verifying proof for $*...
	yarn -s snarkjs groth16 verify proofs/iszerowrong/verification_key.json proofs/iszeroevil/public.json proofs/iszeroevil/proof.json
	@echo
	
# Verify the proof
verify/%: proofs/%/proof.json
	@echo Verifying proof for $*...
	yarn -s snarkjs groth16 verify proofs/$*/verification_key.json proofs/$*/public.json proofs/$*/proof.json
	@echo
	
# Test verifying contracts
test: $(foreach circuit,$(CIRCUITS),proofs/$(circuit)/solidity-args.json contracts/$(circuit).sol)
	@echo Running hardhat tests...
	yarn -s hardhat test
	@echo