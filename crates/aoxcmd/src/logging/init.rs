use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const DATA_ROOT: &str = "AOXC_DATA";
const LOG_DIR: &str = "AOXC_DATA/logs";

const NODE_LOG_NAME: &str = "node.log";
const ERROR_LOG_NAME: &str = "critical.log";
const WARN_LOG_NAME: &str = "warnings.log";

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    DEBUG,
    TRACE,
}

/// Optional chain execution context attached to a log event.
///
/// The structure is intentionally lightweight and serializable into a
/// single-line logging format without additional allocation-heavy metadata.
#[derive(Clone, Debug)]
pub struct ChainContext {
    pub era: u64,
    pub block: u64,
    pub block_hash: String,
}

/// Global logger state.
///
/// This structure centralizes all log sinks so that:
/// - initialization is performed once,
/// - sink routing remains explicit and deterministic,
/// - sink mutation is protected by interior synchronization.
///
/// The design intentionally avoids background logging threads in order to keep
/// failure semantics synchronous and easy to reason about during node bootstrap
/// and runtime incident analysis.
struct Logger {
    node: Mutex<BufWriter<File>>,
    warn: Mutex<BufWriter<File>>,
    error: Mutex<BufWriter<File>>,
}

impl Logger {
    /// Constructs a fully initialized logger instance.
    ///
    /// Security and operational properties:
    /// - ensures all required directories exist before opening files,
    /// - opens log files in append mode to preserve prior records,
    /// - fails atomically if any sink cannot be opened.
    fn initialize() -> Result<Self, io::Error> {
        ensure_directories()?;

        Ok(Self {
            node: Mutex::new(open_log_file(NODE_LOG_NAME)?),
            warn: Mutex::new(open_log_file(WARN_LOG_NAME)?),
            error: Mutex::new(open_log_file(ERROR_LOG_NAME)?),
        })
    }

    /// Writes a formatted line to the appropriate sinks.
    ///
    /// Routing policy:
    /// - every event is written to the main node log,
    /// - warnings are additionally written to the warning log,
    /// - errors are additionally written to the critical log.
    fn write(&self, level: LogLevel, file_line: &str) {
        if let Err(error) = write_to_sink(&self.node, file_line) {
            emit_internal_logger_error("node", &error);
        }

        match level {
            LogLevel::WARN => {
                if let Err(error) = write_to_sink(&self.warn, file_line) {
                    emit_internal_logger_error("warning", &error);
                }
            }
            LogLevel::ERROR => {
                if let Err(error) = write_to_sink(&self.error, file_line) {
                    emit_internal_logger_error("critical", &error);
                }
            }
            LogLevel::INFO | LogLevel::DEBUG | LogLevel::TRACE => {}
        }
    }
}

/// Initializes the AOXC logging subsystem.
///
/// This function is idempotent. Repeated calls succeed as long as the logger
/// has already been initialized successfully.
pub fn init() -> Result<(), io::Error> {
    if LOGGER.get().is_some() {
        return Ok(());
    }

    let logger = Logger::initialize()?;

    match LOGGER.set(logger) {
        Ok(()) => Ok(()),
        Err(_) => Ok(()),
    }
}

/// Writes a log event to terminal output and file sinks.
///
/// Behavior:
/// - terminal output is always attempted,
/// - file sink output is attempted only when initialization has completed,
/// - message fields are sanitized to preserve single-line log integrity.
///
/// Failure policy:
/// This function never panics and never returns an error. Logging failures are
/// reported to stderr in a best-effort manner in order to avoid recursive
/// application instability during incident conditions.
pub fn log(level: LogLevel, module: &str, ctx: Option<&ChainContext>, message: &str) {
    let timestamp = current_timestamp_millis();
    let thread_id = format!("{:?}", thread::current().id());

    let sanitized_module = sanitize_field(module);
    let sanitized_message = sanitize_field(message);
    let chain_info = format_chain_context(ctx);

    let label = level_label(level);
    let (icon, color) = terminal_style(level);

    let terminal_line = format!(
        "\x1b[90m[{}]\x1b[0m {}{}{} \x1b[1m[{}]\x1b[0m \x1b[94m{}\x1b[0m {}\n",
        timestamp, color, icon, label, sanitized_module, chain_info, sanitized_message
    );

    let file_line = format!(
        "[{}] [{}] [{}] [{}] [{}] {}\n",
        timestamp, thread_id, label, sanitized_module, chain_info, sanitized_message
    );

    write_terminal(level, &terminal_line);

    if let Some(logger) = LOGGER.get() {
        logger.write(level, &file_line);
    }
}

/// Ensures all runtime storage directories required by the node exist.
///
/// This helper keeps bootstrap filesystem concerns localized and consistent.
fn ensure_directories() -> Result<(), io::Error> {
    let directories = [
        PathBuf::from(DATA_ROOT).join("logs"),
        PathBuf::from(DATA_ROOT).join("db/blocks"),
        PathBuf::from(DATA_ROOT).join("db/state"),
        PathBuf::from(DATA_ROOT).join("db/mempool"),
        PathBuf::from(DATA_ROOT).join("identity"),
    ];

    for directory in directories {
        fs::create_dir_all(directory)?;
    }

    Ok(())
}

/// Opens a log file in append mode under the configured log directory.
fn open_log_file(name: &str) -> Result<BufWriter<File>, io::Error> {
    let path = build_log_path(name);
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    Ok(BufWriter::new(file))
}

/// Writes a single record to a sink under mutex protection.
///
/// Poisoned locks are recovered intentionally in order to maximize logging
/// continuity even after a prior panic in an unrelated code path.
fn write_to_sink(target: &Mutex<BufWriter<File>>, content: &str) -> Result<(), io::Error> {
    let mut guard = match target.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    guard.write_all(content.as_bytes())?;
    guard.flush()?;
    Ok(())
}

/// Writes a terminal line to stdout or stderr depending on severity.
///
/// Severity routing policy:
/// - WARN and ERROR are emitted to stderr,
/// - INFO, DEBUG, and TRACE are emitted to stdout.
fn write_terminal(level: LogLevel, content: &str) {
    match level {
        LogLevel::WARN | LogLevel::ERROR => {
            let mut stderr = io::stderr().lock();
            let _ = stderr.write_all(content.as_bytes());
            let _ = stderr.flush();
        }
        LogLevel::INFO | LogLevel::DEBUG | LogLevel::TRACE => {
            let mut stdout = io::stdout().lock();
            let _ = stdout.write_all(content.as_bytes());
            let _ = stdout.flush();
        }
    }
}

/// Emits internal logger failures in a minimal and recursion-safe format.
fn emit_internal_logger_error(target: &str, error: &io::Error) {
    let _ = writeln!(
        io::stderr(),
        "[LOGGER_ERROR] sink={} message={}",
        target,
        sanitize_field(&error.to_string())
    );
}

/// Returns the stable textual label for a log level.
fn level_label(level: LogLevel) -> &'static str {
    match level {
        LogLevel::INFO => "INFO ",
        LogLevel::WARN => "WARN ",
        LogLevel::ERROR => "ERROR",
        LogLevel::DEBUG => "DEBUG",
        LogLevel::TRACE => "TRACE",
    }
}

/// Returns terminal icon and ANSI color prefix for a log level.
///
/// The returned color sequence intentionally includes the reset code so that
/// the caller does not need to manage style state externally.
fn terminal_style(level: LogLevel) -> (&'static str, &'static str) {
    match level {
        LogLevel::INFO => ("🟢", "\x1b[32m"),
        LogLevel::WARN => ("⚠️ ", "\x1b[33m"),
        LogLevel::ERROR => ("🔥", "\x1b[31m"),
        LogLevel::DEBUG => ("🔧", "\x1b[36m"),
        LogLevel::TRACE => ("🔍", "\x1b[35m"),
    }
}

/// Formats optional chain context into a stable single-line representation.
fn format_chain_context(ctx: Option<&ChainContext>) -> String {
    match ctx {
        Some(context) => {
            let sanitized_hash = sanitize_field(&context.block_hash);
            let short_hash = truncate_for_display(&sanitized_hash, 16);

            format!(
                "Era:{} Block:{} Hash:{}",
                context.era, context.block, short_hash
            )
        }
        None => "Era:- Block:- Hash:-".to_string(),
    }
}

/// Builds a fully qualified log file path.
fn build_log_path(name: &str) -> PathBuf {
    Path::new(LOG_DIR).join(name)
}

/// Sanitizes an arbitrary input field for safe single-line logging.
///
/// Sanitization rules:
/// - carriage return and newline are replaced with spaces,
/// - ANSI escape introducers are replaced,
/// - other control characters are replaced with '?' except horizontal tab.
fn sanitize_field(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\r' | '\n' => output.push(' '),
            '\x1b' => output.push('?'),
            c if c.is_control() && c != '\t' => output.push('?'),
            c => output.push(c),
        }
    }

    output
}

/// Truncates a string for operator-friendly display without panicking on UTF-8 boundaries.
fn truncate_for_display(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    input.chars().take(max_chars).collect()
}

/// Returns the current wall-clock UNIX timestamp in milliseconds.
///
/// This representation is compact, sortable, and stable across process restarts.
fn current_timestamp_millis() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis().to_string(),
        Err(_) => "0".to_string(),
    }
}
