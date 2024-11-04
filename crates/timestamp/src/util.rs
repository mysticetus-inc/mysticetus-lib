macro_rules! min {
    ($a:expr, $b:expr $(, $rem:expr)* $(,)?) => {
        min!(if $a < $b { $a } else { $b } $(, $rem)*)
    };
    ($t:expr) => { $t };
}

macro_rules! max {
    ($a:expr, $b:expr $(, $rem:expr)* $(,)?) => {
        max!(if $a < $b { $b } else { $a } $(, $rem)*)
    };
    ($t:expr) => { $t };
}

macro_rules! clamp {
    ($t:expr; $min:tt.. $max:tt) => {
        if $t < $min {
            $min
        } else if $max - 1 < $t {
            $max - 1
        } else {
            $t
        }
    };
    ($t:expr; $min:tt..= $max:tt) => {
        if $t < $min {
            $min
        } else if $max < $t {
            $max
        } else {
            $t
        }
    };
}

pub(crate) use {clamp, max, min};

#[test]
fn test_macros() {
    let m = min!(5, 2, 3, 3, 4, 1, 1234);
    assert_eq!(m, 1);

    let m2 = max!(1, 4, 123, 521);
    assert_eq!(m2, 521);

    const MAX: usize = 3;
    const MIN: usize = 1;

    let clamped = clamp!(4; MIN..=MAX);
    assert_eq!(clamped, 3);

    let clamped = clamp!(4; MIN..MAX);
    assert_eq!(clamped, 2);
}
