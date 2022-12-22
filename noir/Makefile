all: multiplier2 advent2

.PHONY: build verify/* multiplier2 advent2

.SECONDARY:

CIRCUITS := multiplier2 advent2

multiplier2: verify/multiplier2
advent2: verify/advent2

circuits/%/build/main.acir: circuits/%/src/main.nr
	@echo Building $*...
	cd circuits/$* && nargo compile --witness main
	@echo

circuits/%/proofs/main.proof: circuits/%/build/main.acir
	@echo Generating proof for $*...
	cd circuits/$* && nargo prove main
	@echo

contracts/%.sol: circuits/%/build/main.acir
	@echo Generating contract for $*...
	cd circuits/$* && nargo contract
	mv circuits/$*/contract/plonk_vk.sol contracts/$*.sol
	sed -i 's/pragma solidity .*/pragma solidity >=0.8.0;'/ contracts/$*.sol
	@echo

verify/%: circuits/%/proofs/main.proof
	@echo Verifying proof for $*...
	cd circuits/$* && nargo verify main
	@echo

test: $(CIRCUITS)
	@echo Running hardhat tests...
	yarn -s hardhat test
	@echo

