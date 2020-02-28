use jane_eyre::{eyre, ErrReport};

const START: &str = "Error: 
   0: Heres an error

Stack Backtrace:
   0: <jane_eyre::JaneContext as eyre::EyreContext>::default";

#[test]
fn with_backtrace() {
    let msg = "Heres an error";
    std::env::set_var("RUST_LIB_BACKTRACE", "1");
    let e: ErrReport = eyre!(msg);

    let dbg = format!("Error: {:?}", e);
    println!("{}", dbg);

    assert!(dbg.starts_with(START));
}
