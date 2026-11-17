/// Keep stdio and shared workspace sockets on the same protocol registration.
pub fn service<F>(
    init: F,
) -> (
    tower_lsp::LspService<crate::HickBackend>,
    tower_lsp::ClientSocket,
)
where
    F: FnOnce(tower_lsp::Client) -> crate::HickBackend,
{
    tower_lsp::LspService::build(init)
        .custom_method("hick/captureDocument", crate::HickBackend::capture_document)
        .custom_method("hick/rename", crate::HickBackend::source_bound_rename)
        .custom_method("hick/formatting", crate::HickBackend::source_bound_formatting)
        .finish()
}
