pragma circom 2.0.0;

template MultiplierSq() {
  signal input a;
  signal input b;
  signal ab;
  signal output c;
  ab <== a * b;
  c <== ab * ab;
}

component main = MultiplierSq();
