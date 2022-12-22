# Comparing languages for zk-proofs

The purpose of this repo is just to play around with general-purpose languages for generating zero-knowledge proofs, and compare them when working with the same example problem. The languages used so far are:

- [Circom](/circom/)
- [Halo2](/halo2/)
- [Noir](/noir/)

For each language, there is a circuit that generates and verifies a zero-knowledge proof for the problem presented in [Day 2 of Advent of Code 2022](https://adventofcode.com/2022/day/2), which asks you to calculate the score in a game of several rounds of rock-paper-scissors. Given a set of rounds, the score is calculated as 6 points if you win, 3 on a draw, 0 on a loss, plus 1, 2, or 3 points depending on whether you played rock, paper, or scissors. 

The circuits in this repo generate a proof that the prover knows a set of rounds that amount to a given total score, without disclosing them. In other words:

- The private inputs are the arrays of choices by each player on each round, where 0 is rock, 1 is paper, and 2 is scissors
- The public value is the total score achieved by the second player
- The proof proves that the public score corresponds to the private inputs, and that all inputs are valid

> :warning: I wrote these circuits as a means to learn these languages. I am definitely not an expert in any of them, so please take everything you see here with a pinch of salt.