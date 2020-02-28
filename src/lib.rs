#![feature(backtrace)]

pub use eyre::*;

pub type ErrReport = eyre::ErrReport<JaneContext>;
pub type Result<T, E = ErrReport> = core::result::Result<T, E>;

use std::backtrace::BacktraceStatus;
use std::error::Error;
use std::backtrace::Backtrace;
use std::any::{TypeId, Any};
use std::fmt::Write as _;
use indenter::Indented;
use tracing_error::{SpanTrace, SpanTraceStatus, ExtractSpanTrace};


pub struct JaneContext {
    backtrace: Option<Backtrace>,
    span_trace: Option<SpanTrace>,
}

fn get_deepest_backtrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a Backtrace> {
    Chain::new(error).rev().flat_map(|error| error.backtrace()).next()
}

fn get_deepest_spantrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SpanTrace> {
    Chain::new(error).rev().flat_map(|error| error.span_trace()).next()
}

impl EyreContext for JaneContext {
    fn default(error: &(dyn std::error::Error + 'static)) -> Self {
        let backtrace = if get_deepest_backtrace(error).is_none() {
            Some(Backtrace::capture())
        } else {
            None
        };

        let span_trace = if get_deepest_spantrace(error).is_none() {
            Some(SpanTrace::capture())
        } else {
            None
        };

        Self { backtrace, span_trace }
    }

    fn context_raw(&self, typeid: TypeId) -> Option<&dyn Any> {
        if typeid == TypeId::of::<Backtrace>() {
            self.backtrace.as_ref().map(|b| b as &dyn Any)
        } else {
            None
        }
    }

    fn display(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}", error)?;

        if f.alternate() {
            for cause in Chain::new(error).skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }

    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        let errors = Chain::new(error).rev().enumerate();

        for (n, error) in errors {
            writeln!(f)?;
            write!(Indented::numbered(f, n), "{}", error)?;
        }

        let span_trace = self
            .span_trace
            .as_ref()
            .or_else(|| get_deepest_spantrace(error))
            .expect("SpanTrace capture failed");

        if span_trace.status() == SpanTraceStatus::CAPTURED {
            write!(f, "\n\nSpan Trace:\n{}", span_trace)?;
        }

        let backtrace = self
            .backtrace
            .as_ref()
            .or_else(|| get_deepest_backtrace(error))
            .expect("backtrace capture failed");

        if let BacktraceStatus::Captured = backtrace.status() {
            write!(f, "\n\nStack Backtrace:\n{}", backtrace)?;
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
