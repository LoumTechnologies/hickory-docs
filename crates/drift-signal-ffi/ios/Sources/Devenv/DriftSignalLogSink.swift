import Foundation
import os

// MARK: - VSLICE: DriftSignal @Logging

/// Forwards Rust tracing events to Apple's unified logging system.
class DriftSignalLogSink: LogSink {
    private let logger = Logger(subsystem: "dev.drift-signal", category: "rust")

    func log(level: String, message: String, target: String, file: String, line: UInt32) {
        switch level {
        case "ERROR":
            logger.error("[\(target)] \(message)")
        case "WARN":
            logger.warning("[\(target)] \(message)")
        case "INFO":
            logger.info("[\(target)] \(message)")
        case "DEBUG":
            logger.debug("[\(target)] \(message)")
        default:
            logger.trace("[\(target)] \(message)")
        }
    }
}
