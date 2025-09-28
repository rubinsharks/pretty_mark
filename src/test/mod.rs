mod test;
mod markdown;
mod page;

#[cfg(test)]
mod test_toml {
    #[test]
    fn test_tomlview() {
        println!("test2");
        assert_eq!(1, 1);
    }
}

#[cfg(test)]
mod test_html {
    #[test]
    fn test_htmlview() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_html() {
        assert_eq!(1, 1);
    }
}