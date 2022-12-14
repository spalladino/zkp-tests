all: multiplier2 advent2

.PHONY: build verify/* multiplier2 advent2

.SECONDARY:

multiplier2: circuits/multiplier2/build/main.acir
advent2: circuits/advent2/build/main.acir

circuits/%/build/main.acir: circuits/%/src/main.nr
	@echo Building $*...
	cd circuits/$* && nargo compile --witness $*
	@echo

circuits/%/proofs/main.proof: circuits/%/build/main.acir
	@echo Generating proof for $*...
	cd circuits/$* && nargo prove main
	@echo

verify/%: circuits/%/proofs/main.proof
	@echo Verifying proof for $*...
	cd circuits/$* && nargo verify main
	@echo

