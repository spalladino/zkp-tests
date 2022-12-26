// Adapted from iden3/circomlib
template IsZero() {
    signal input in;
    signal output out;

    signal inv;

    inv <-- in!=0 ? 1/in : 0;
    out <-- -in*inv +1;
    
    out === -in*inv +1;
    in*out === 0;
}

template Main() {
  signal input x;
  signal output out;
  
  out <== IsZero()(x);
  out === 0;
}

component main = Main();