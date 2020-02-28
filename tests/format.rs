use jane_eyre::{eyre, ErrReport};

#[test]
fn no_capture() {
    let msg = "Heres an error";
    let e: ErrReport = eyre!(msg);

    let dbg = format!("{:?}", e);

    let expected = "\n   0: Heres an error";
    assert_eq!(expected, dbg);
}
