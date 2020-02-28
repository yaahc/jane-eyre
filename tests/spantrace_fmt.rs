use tracing_error::ErrorLayer;
use jane_eyre::{ErrReport, eyre};
use tracing::subscriber::with_default;
use tracing::{span, Level};
use tracing_subscriber::{prelude::*, registry::Registry};

const EXPECTED: &str = "Error: 
   0: Heres an error

Span Trace:
   0: spantrace_fmt::test span
             at tests/spantrace_fmt.rs:19";

#[test]
fn capture_supported() {
    let subscriber = Registry::default().with(ErrorLayer::default());

    with_default(subscriber, || {
        let span = span!(Level::ERROR, "test span");
        let _guard = span.enter();

        let msg = "Heres an error";
        let e: ErrReport = eyre!(msg);

        let dbg = format!("Error: {:?}", e);
        println!("{}", dbg);

        assert_eq!(EXPECTED, dbg);
    });
}
