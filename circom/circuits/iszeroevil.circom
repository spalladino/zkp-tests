// Adapted from iden3/circomlib
template IsZero() {
    signal input in;
    signal output out;

    signal inv;

    inv <-- 0;
    out <-- 0;
    
    in*out === 0;
}

template Main() {
  signal input x;
  signal output out;
  
  out <== IsZero()(x);
  out === 0;
}

component main = Main();