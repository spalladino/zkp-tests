all: multiplier2

.PHONY: verify/% contracts test

# Build verifying contracts
contracts: contracts/multiplier2.sol

# Build circuit, create witness, create proof, and verify it
multiplier2: verify/multiplier2

# Do not delete intermediate files
.SECONDARY:

# Builds a circuit
build/%.r1cs: circuits/%.circom
	@echo Building $@...
	mkdir -p build/
	circom $< --r1cs --wasm --sym --c -o build/
	@echo

# Creates a witness out of the circuit and input
inputs/%.wtns: build/%.r1cs inputs/%.json
	@echo Generating witness for $@...
	node build/$*_js/generate_witness.js \
		build/$*_js/$*.wasm \
		inputs/$*.json \
		inputs/$*.wtns

# General phase 1 ceremony of powers of tau (trusted setup)
proofs/pot12_0001.ptau:
	@echo Powers of tau ceremony phase 1...
	mkdir -p proofs
	yarn snarkjs powersoftau new bn128 12 proofs/pot12_0000.ptau -v
	yarn snarkjs powersoftau contribute proofs/pot12_0000.ptau proofs/pot12_0001.ptau --name="First contribution"
	@echo

# Circuit-specific phase 2 ceremony of powers of tau (trusted setup)
proofs/%/verification_key.json: proofs/pot12_0001.ptau build/%.r1cs
	@echo Phase two of the ceremony...
	mkdir -p proofs/$*
	yarn snarkjs powersoftau prepare phase2 proofs/pot12_0001.ptau proofs/$*/pot12.ptau
	yarn snarkjs groth16 setup build/multiplier2.r1cs proofs/$*/pot12.ptau proofs/$*/key_0000.zkey
	yarn snarkjs zkey contribute proofs/$*/key_0000.zkey proofs/$*/key_0001.zkey --name="First contribution"
	yarn snarkjs zkey export verificationkey proofs/$*/key_0001.zkey proofs/$*/verification_key.json
	@echo

# Create a verifier contract out of a verification key
contracts/%.sol: proofs/%/verification_key.json
	@echo Generating verifier contract...
	mkdir -p contracts/
	yarn snarkjs zkey export solidityverifier proofs/$*/key_0001.zkey contracts/$*.sol

# Create proof for a given witness, given the verification key generated in phase 2
proofs/%/proof.json: proofs/%/verification_key.json inputs/%.wtns
	@echo Generating proof...
	yarn snarkjs groth16 prove proofs/$*/key_0001.zkey inputs/$*.wtns proofs/$*/proof.json proofs/$*/public.json
	@echo

# Create proof to be fed into verifier contract
proofs/%/solidity-args.json: proofs/%/proof.json
	echo '[' > $@
	yarn run -s snarkjs zkey export soliditycalldata proofs/$*/public.json proofs/$*/proof.json >> $@
	echo ']' >> $@

# Verify the proof
verify/%: proofs/%/proof.json
	@echo Verifying proof for $*...
	yarn snarkjs groth16 verify proofs/$*/verification_key.json proofs/$*/public.json proofs/$*/proof.json
	@echo
	
# Test verifying contracts
test: proofs/multiplier2/solidity-args.json
	@echo Running hardhat tests...
	yarn hardhat test
	@echo