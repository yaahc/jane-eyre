#![feature(backtrace)]

mod help;

pub use eyre::*;

pub use help::Help;
pub type ErrReport = eyre::ErrReport<JaneContext>;
pub type Result<T, E = ErrReport> = core::result::Result<T, E>;

use help::HelpInfo;
use indenter::Indented;
use std::any::{Any, TypeId};
use std::backtrace::Backtrace;
use std::backtrace::BacktraceStatus;
use std::error::Error;
use std::fmt::Write as _;
use tracing_error::{ExtractSpanTrace, SpanTrace, SpanTraceStatus};

pub struct JaneContext {
    backtrace: Option<Backtrace>,
    span_trace: Option<SpanTrace>,
    help: Vec<HelpInfo>,
}

fn get_deepest_backtrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a Backtrace> {
    Chain::new(error)
        .rev()
        .flat_map(|error| error.backtrace())
        .next()
}

fn get_deepest_spantrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SpanTrace> {
    Chain::new(error)
        .rev()
        .flat_map(|error| error.span_trace())
        .next()
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

        Self {
            backtrace: backtrace,
            span_trace: span_trace,
            help: Vec::new(),
        }
    }

    fn member_ref(&self, typeid: TypeId) -> Option<&dyn Any> {
        if typeid == TypeId::of::<Backtrace>() {
            self.backtrace.as_ref().map(|b| b as &dyn Any)
        } else if typeid == TypeId::of::<SpanTrace>() {
            self.span_trace.as_ref().map(|s| s as &dyn Any)
        } else {
            None
        }
    }

    fn member_mut(&mut self, typeid: TypeId) -> Option<&mut dyn Any> {
        if typeid == TypeId::of::<Vec<HelpInfo>>() {
            Some(&mut self.help)
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

        for help in &self.help {
            write!(f, "\n{}", help)?;
        }

        let span_trace = self
            .span_trace
            .as_ref()
            .or_else(|| get_deepest_spantrace(error))
            .expect("SpanTrace capture failed");

        match span_trace.status() {
            SpanTraceStatus::CAPTURED => write!(f, "\n\nSpan Trace:\n{}", span_trace)?,
            SpanTraceStatus::UNSUPPORTED => write!(f, "\n\nWarning: SpanTrace capture is Unsupported.\nEnsure that you've setup an error layer and the versions match")?,
            _ => (),
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
