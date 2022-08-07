fn main() {
    println!("try to acquire a token");
    let token = ctk_example::acquire_token();

    println!("got token: {:?}", token);
}
