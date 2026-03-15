pub fn greet() {
    println!("Hello from the aoxclibs library!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
