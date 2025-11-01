pub fn auth(client_id: String) {
    let path = "http://rust-lang.org";

    println!("{}", client_id);

    match open::that(path) {
        Ok(()) => println!("Opened '{}' successfully.", path),
        Err(err) => eprintln!("An error occurred when opening '{}': {}", path, err),
    }
}
