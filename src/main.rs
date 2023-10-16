
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn should() {
        assert_eq!(true, false)
    }
}
