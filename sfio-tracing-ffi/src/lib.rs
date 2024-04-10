//! [oo_bindgen](https://github.com/stepfunc/oo_bindgen/) model for FFI bindings to [tracing](https://docs.rs/tracing/latest/tracing/).w
use oo_bindgen::model::*;

/// Retrieve the implementation file for this schema
pub const fn get_impl_file() -> &'static str {
    include_str!("../implementation.rs")
}

/// Adds required types for logging to an `oo_bindgen` model
///
/// * `lib` - Builder for library
/// * `error_type` - Error type that will be used if logging configuration fails
pub fn define(lib: &mut LibraryBuilder, error_type: ErrorTypeHandle) -> BackTraced<()> {
    let log_level_enum = define_log_level_enum(lib)?;

    let logging_config_struct = define_logging_config_struct(lib, log_level_enum.clone())?;

    let log_callback_interface = lib
        .define_interface(
            "logger",
            "Logging interface that receives the log messages and writes them somewhere.",
        )?
        .begin_callback(
            "on_message",
            "Called when a log message was received and should be logged",
        )?
        .param("level", log_level_enum, "Level of the message")?
        .param("message", StringType, "Actual formatted message")?
        .end_callback()?
        .build_async()?;

    let configure_logging_fn = lib
        .define_function("configure_logging")?
        .param(
            "config",
            logging_config_struct,
            "Configuration options for logging"
        )?
        .param(
            "logger",
            log_callback_interface,
            "Logger that will receive each logged message",
        )?
        .fails_with(error_type)?
        .doc(
            doc("Set the callback that will receive all the log messages")
                .details("There is only a single globally allocated logger. Calling this method a second time will return an error.")
                .details("If this method is never called, no logging will be performed.")
        )?
        .build_static("configure")?;

    let _ = lib
        .define_static_class("logging")?
        .static_method(configure_logging_fn)?
        .doc("Provides a static method for configuring logging")?
        .build()?;

    Ok(())
}
fn define_log_level_enum(lib: &mut LibraryBuilder) -> BackTraced<EnumHandle> {
    let definition = lib
        .define_enum("log_level")?
        .push("error", "Error log level")?
        .push("warn", "Warning log level")?
        .push("info", "Information log level")?
        .push("debug", "Debugging log level")?
        .push("trace", "Trace log level")?
        .doc(
            doc("Log level")
                .details("Used in {interface:logger.on_message()} callback to identify the log level of a message.")
        )?
        .build()?;

    Ok(definition)
}

fn define_time_format_enum(lib: &mut LibraryBuilder) -> BackTraced<EnumHandle> {
    let definition = lib
        .define_enum("time_format")?
        .push("none", "Don't format the timestamp as part of the message")?
        .push("rfc_3339", "Format the time using RFC 3339")?
        .push(
            "system",
            "Format the time in a human readable format e.g. 'Jun 25 14:27:12.955'",
        )?
        .doc("Describes if and how the time will be formatted in log messages")?
        .build()?;

    Ok(definition)
}

fn define_log_output_format_enum(lib: &mut LibraryBuilder) -> BackTraced<EnumHandle> {
    let definition = lib
        .define_enum("log_output_format")?
        .push("text", "A simple text-based format")?
        .push("json", "Output formatted as JSON")?
        .doc("Describes how each log event is formatted")?
        .build()?;

    Ok(definition)
}

fn define_logging_config_struct(
    lib: &mut LibraryBuilder,
    log_level_enum: EnumHandle,
) -> BackTraced<FunctionArgStructHandle> {
    let logging_config_struct = lib.declare_function_argument_struct("logging_config")?;

    let log_output_format_enum = define_log_output_format_enum(lib)?;
    let time_format_enum = define_time_format_enum(lib)?;

    let level = Name::create("level")?;
    let output_format = Name::create("output_format")?;
    let time_format = Name::create("time_format")?;
    let print_level = Name::create("print_level")?;
    let print_module_info = Name::create("print_module_info")?;

    let logging_config_struct = lib
        .define_function_argument_struct(logging_config_struct)?
        .add(&level, log_level_enum, "logging level")?
        .add(
            &output_format,
            log_output_format_enum,
            "output formatting options",
        )?
        .add(&time_format, time_format_enum, "optional time format")?
        .add(
            &print_level,
            Primitive::Bool,
            "optionally print the log level as part to the message string",
        )?
        .add(
            &print_module_info,
            Primitive::Bool,
            "optionally print the underlying Rust module information to the message string",
        )?
        .doc("Logging configuration options")?
        .end_fields()?
        .begin_initializer(
            "init",
            InitializerType::Normal,
            "Initialize the configuration to default values",
        )?
        .default(&level, "info".default_variant())?
        .default(&output_format, "text".default_variant())?
        .default(&time_format, "system".default_variant())?
        .default(&print_level, true)?
        .default(&print_module_info, false)?
        .end_initializer()?
        .build()?;

    Ok(logging_config_struct)
}
