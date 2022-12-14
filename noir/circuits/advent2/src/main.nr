global N: Field = 3;

#[builtin(arraylen)]
fn len<T>(_input : [T]) -> comptime Field {}

fn round(x: Field, y: Field) -> Field {
    constrain (x as u8) <= 2;
    constrain (y as u8) <= 2;

    let mut score = 0;
    let diffYX = (y + 3 - x);

    if x == y {
        score = 3;
    } 
    
    // TODO: writing this as else-if causes witness compilation to fail
    if (diffYX == 4) | (diffYX == 1) {
        score = 6;
    }

    score + y + 1
}

fn main(xs: [Field; N], ys: [Field; N]) -> pub Field {
    let mut result = 0;
    
    for i in 0..len(xs) {
        result = result + round(xs[i], ys[i]);
    }
    result
}