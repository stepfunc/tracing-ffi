use std::ffi::CString;

use tracing::span::{Attributes, Record};
use tracing::{Event, Id, Metadata};
use tracing_subscriber::fmt::time::{ChronoUtc, SystemTime};
use tracing_subscriber::fmt::MakeWriter;

use crate::ffi;

thread_local! {
   pub static LOG_BUFFER: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[derive(Copy, Clone, Debug)]
pub struct TracingInitError;

pub fn configure_logging(
    config: ffi::LoggingConfig,
    handler: ffi::Logger,
) -> Result<(), TracingInitError> {
    tracing::subscriber::set_global_default(adapter(config, handler)).map_err(|_| TracingInitError)
}

struct ThreadLocalBufferWriter;

struct ThreadLocalMakeWriter;

impl<'a> MakeWriter<'a> for ThreadLocalMakeWriter {
    type Writer = ThreadLocalBufferWriter;

    fn make_writer(&self) -> Self::Writer {
        ThreadLocalBufferWriter
    }
}

impl std::io::Write for ThreadLocalBufferWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        LOG_BUFFER.with(|vec| vec.borrow_mut().extend_from_slice(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn adapter(
    config: ffi::LoggingConfig,
    handler: ffi::Logger,
) -> impl tracing::Subscriber + Send + Sync + 'static {
    Adapter {
        handler,
        inner: config.build(),
    }
}

impl ffi::LoggingConfig {
    fn build(&self) -> Box<dyn tracing::Subscriber + Send + Sync> {
        let level: tracing::Level = self.level().into();

        let builder = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(level)
            .with_level(self.print_level)
            .with_target(self.print_module_info)
            .with_writer(ThreadLocalMakeWriter);

        match self.time_format() {
            ffi::TimeFormat::None => {
                let builder = builder.without_time();
                match self.output_format() {
                    ffi::LogOutputFormat::Text => Box::new(builder.finish()),
                    ffi::LogOutputFormat::Json => Box::new(builder.json().finish()),
                }
            }
            ffi::TimeFormat::Rfc3339 => {
                let builder = builder.with_timer(ChronoUtc::default());
                match self.output_format() {
                    ffi::LogOutputFormat::Text => Box::new(builder.finish()),
                    ffi::LogOutputFormat::Json => Box::new(builder.json().finish()),
                }
            }
            ffi::TimeFormat::System => {
                let builder = builder.with_timer(SystemTime);
                match self.output_format() {
                    ffi::LogOutputFormat::Text => Box::new(builder.finish()),
                    ffi::LogOutputFormat::Json => Box::new(builder.json().finish()),
                }
            }
        }
    }
}

struct Adapter {
    handler: ffi::Logger,
    inner: Box<dyn tracing::Subscriber + Send + Sync + 'static>,
}

impl tracing::Subscriber for Adapter {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.inner.enabled(metadata)
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        self.inner.new_span(span)
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        self.inner.record(span, values)
    }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        self.inner.record_follows_from(span, follows)
    }

    fn event(&self, event: &Event<'_>) {
        self.inner.event(event);
        if let Ok(string) = LOG_BUFFER.with(|vec| CString::new(vec.borrow().as_slice())) {
            self.handler
                .on_message((*event.metadata().level()).into(), &string);
        }
        LOG_BUFFER.with(|vec| vec.borrow_mut().clear())
    }

    fn enter(&self, span: &Id) {
        self.inner.enter(span)
    }

    fn exit(&self, span: &Id) {
        self.inner.exit(span)
    }

    fn clone_span(&self, span: &Id) -> Id {
        self.inner.clone_span(span)
    }

    fn try_close(&self, span: Id) -> bool {
        self.inner.try_close(span)
    }

    fn current_span(&self) -> tracing_core::span::Current {
        self.inner.current_span()
    }
}

impl From<tracing::Level> for ffi::LogLevel {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::DEBUG => ffi::LogLevel::Debug,
            tracing::Level::TRACE => ffi::LogLevel::Trace,
            tracing::Level::INFO => ffi::LogLevel::Info,
            tracing::Level::WARN => ffi::LogLevel::Warn,
            tracing::Level::ERROR => ffi::LogLevel::Error,
        }
    }
}

impl From<ffi::LogLevel> for tracing::Level {
    fn from(level: ffi::LogLevel) -> Self {
        match level {
            ffi::LogLevel::Debug => tracing::Level::DEBUG,
            ffi::LogLevel::Trace => tracing::Level::TRACE,
            ffi::LogLevel::Info => tracing::Level::INFO,
            ffi::LogLevel::Warn => tracing::Level::WARN,
            ffi::LogLevel::Error => tracing::Level::ERROR,
        }
    }
}
