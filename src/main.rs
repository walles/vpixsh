#![allow(clippy::needless_return)]

fn main() {
    println!("{}+{}={}", 1, 2, add(1, 2));
}

fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
