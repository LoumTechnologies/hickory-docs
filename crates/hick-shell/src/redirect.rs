// Phase 1: File redirects (>, >>, <, 2>, &>) via WASI pre-opens.
//
// Parse IoRedirect AST nodes and configure WASI context accordingly:
// - Output redirects: create/truncate or append to file
// - Input redirects: read from file
// - Here-documents and here-strings
