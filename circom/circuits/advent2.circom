pragma circom 2.1.1;

include "comparators.circom";

/*
 Rock=0
 Paper=1
 Scissors=2

 x | y | out
 -----------
 0 | 2 | 0
 0 | 0 | 1
 0 | 1 | 2
 1 | 0 | 0
 1 | 1 | 1
 1 | 2 | 2
 2 | 1 | 0
 2 | 2 | 1
 2 | 0 | 2
*/ 

// Checks if input x is a valid rock-paper-scissors value (0,1,2)
template AssertIsRPS() {
  signal input x;
  signal isRP <== (x-0) * (x-1);
  isRP * (x-2) === 0;
}

// Returns the score for a single round, given the plays by x and y
template Round() {
  signal input x, y;
  signal output out;
  
  // ensure that each input is within 0,1,2
  AssertIsRPS()(x);
  AssertIsRPS()(y);

  // check if match was a draw
  signal isDraw <== IsEqual()([x, y]);
  
  signal diffYX <== (y+3)-x;

  // y wins if y-x = 1 mod 3
  signal yWins1 <== (diffYX-1) * (diffYX-4);
  signal yWins <== IsZero()(yWins1);

  // x wins if y-x = 2 mod 3
  signal xWins1 <== (diffYX-2) * (diffYX-5);
  signal xWins <== IsZero()(xWins1);

  // check that exactly one of xWins, yWins, isDraw is true
  // we probably can do without these constraints
  signal xOrYWins <== (xWins - 1) * (yWins - 1);
  xOrYWins * (isDraw - 1) === 0;
  xWins + yWins + isDraw === 1;

  // score is 6 if y wins, 3 if draw, 0 if x wins
  // plus 1, 2, 3 depending on the choice of RPS
  out <== yWins * 6 + isDraw * 3 + y + 1;
}

// Returns the score over all matches in a game with n rounds
// Inputs are the plays by x and y
template Game(n) {
  signal input xs[n];
  signal input ys[n];
  signal scores[n];
  signal output out;
  
  var score = 0;
  for (var i = 0; i < n; i++) {
    scores[i] <== Round()(xs[i], ys[i]);
    score += scores[i];
  }

  out <== score;
}

component main = Game(3);
