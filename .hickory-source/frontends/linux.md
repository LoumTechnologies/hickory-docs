<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Run the shared simple workspace in KDE

For maintainers adding Linux desktop behavior while keeping the editor's ordinary
entry points and source guards. KDE owns the top-level window and Wayland/X11
integration. Qt WebEngine runs the same compiled Simple Browser workspace,
including its editor, terminal emulator, agent pane and guarded actions.

## Build a self-contained application layout

Keep the browser build and the workspace host alongside the executable. Paths are
relative to the installed executable, so a staged prefix and a system install use
the same lookup. The installed application needs the Qt and KDE runtime libraries.

<ui:copy id="linux-simple-build">
cmake_minimum_required(VERSION 3.22)
project(HickorySimpleLinux VERSION 0.1.0 LANGUAGES CXX)

include(GNUInstallDirs)
find_package(Qt6 6.8 REQUIRED COMPONENTS Widgets WebEngineWidgets)
find_package(KF6CoreAddons REQUIRED)
find_package(KF6XmlGui REQUIRED)

set(HICKORY_SIMPLE_HOST "" CACHE FILEPATH "Built Hickory simple workspace host")
set(HICKORY_SIMPLE_WEB "" CACHE PATH "Compiled Simple Browser assets")
if(NOT EXISTS "${HICKORY_SIMPLE_HOST}" OR NOT EXISTS "${HICKORY_SIMPLE_WEB}/simple.html")
    message(FATAL_ERROR "Workspace host or editor assets missing. Run just simple-build linux.")
endif()

file(RELATIVE_PATH host_path "${CMAKE_INSTALL_FULL_BINDIR}"
    "${CMAKE_INSTALL_FULL_LIBEXECDIR}/hickory-simple-host")
file(RELATIVE_PATH web_path "${CMAKE_INSTALL_FULL_BINDIR}"
    "${CMAKE_INSTALL_FULL_DATADIR}/hickory-simple")

add_executable(hickory-simple main.cpp)
target_compile_features(hickory-simple PRIVATE cxx_std_17)
target_compile_definitions(hickory-simple PRIVATE
    HICKORY_HOST_PATH="${host_path}"
    HICKORY_WEB_PATH="${web_path}"
    HICKORY_VERSION="${PROJECT_VERSION}")
target_link_libraries(hickory-simple PRIVATE
    Qt6::Widgets Qt6::WebEngineWidgets KF6::CoreAddons KF6::XmlGui)
install(TARGETS hickory-simple RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")
install(PROGRAMS "${HICKORY_SIMPLE_HOST}" DESTINATION "${CMAKE_INSTALL_LIBEXECDIR}"
    RENAME hickory-simple-host)
install(DIRECTORY "${HICKORY_SIMPLE_WEB}/" DESTINATION "${CMAKE_INSTALL_DATADIR}/hickory-simple")
install(FILES com.hickorydocs.Simple.desktop DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
</ui:copy>

Plasma can launch with a selected local file. The shell uses its containing
folder as the initial workspace and passes the file as an encoded path query.

<ui:copy id="linux-simple-desktop-entry">
[Desktop Entry]
Type=Application
Name=Hickory Simple
Comment=Edit a Hickory workspace
Exec=hickory-simple %f
Icon=accessories-text-editor
Terminal=false
Categories=Development;TextEditor;IDE;
MimeType=text/plain;text/markdown;
StartupNotify=true
</ui:copy>

## Constrain the browser surface to its workspace

A top-level view stays on the chosen workspace page and origin. Follow explicit
external HTTP links through the system browser. The new-window address launches
the native workspace chooser in a separate process, so the chooser can outlive
the original workspace. A close request passes through the editor's before-unload guard. Keep the native window free of toolbars so the
shared editor can use its usual file tree, menus and anchored controls.
Set the native caption when constructing the window so it is visible during loading.

The shared editor advertises the directory command through a user-initiated popup
to a single same-origin action. Keep the picker with Qt and pass the chosen
folder directly to a new local process, preserving the originating workspace.

<ui:copy id="linux-simple-workspace-window">
#include <KAboutData>
#include <KMainWindow>

#include <QApplication>
#include <QCloseEvent>
#include <QCommandLineParser>
#include <QDesktopServices>
#include <QDialog>
#include <QDialogButtonBox>
#include <QDir>
#include <QFileDialog>
#include <QFileInfo>
#include <QFont>
#include <QIcon>
#include <QInputDialog>
#include <QLineEdit>
#include <QLabel>
#include <QMessageBox>
#include <QProcess>
#include <QPushButton>
#include <QTimer>
#include <QVBoxLayout>
#include <QWebEngineDownloadRequest>
#include <QWebEngineNewWindowRequest>
#include <QUrlQuery>
#include <QWebEnginePage>
#include <QWebEnginePermission>
#include <QWebEngineProfile>
#include <QWebEngineScript>
#include <QWebEngineScriptCollection>
#include <QWebEngineView>

#include <memory>

namespace {
constexpr int defaultPort = 38174;

bool sameOrigin(const QUrl &url, const QUrl &workspace)
{
    const int defaultSchemePort = workspace.scheme() == QStringLiteral("https") ? 443 : 80;
    return url.scheme() == workspace.scheme() && url.host() == workspace.host()
        && url.port(defaultSchemePort) == workspace.port(defaultSchemePort) && url.userInfo().isEmpty();
}

bool workspacePage(const QUrl &url, const QUrl &workspace)
{
    return sameOrigin(url, workspace) && url.path() == QStringLiteral("/simple.html");
}

void openExternalLink(const QUrl &url)
{
    if ((url.scheme() == QStringLiteral("https") || url.scheme() == QStringLiteral("http"))
        && !url.host().isEmpty() && url.userInfo().isEmpty()) {
        QDesktopServices::openUrl(url);
    }
}

QString pickDirectory(QWidget *owner)
{
    return QFileDialog::getExistingDirectory(owner, QObject::tr("Open Directory"));
}

class WorkspacePage final : public QWebEnginePage {
public:
    WorkspacePage(QWebEngineProfile &profile, const QUrl &workspace, QObject *parent)
        : QWebEnginePage(&profile, parent), workspace(workspace)
    {
        QWebEngineScript pickerCapability;
        pickerCapability.setInjectionPoint(QWebEngineScript::DocumentCreation);
        pickerCapability.setWorldId(QWebEngineScript::MainWorld);
        pickerCapability.setSourceCode(QStringLiteral(
            "Object.defineProperty(window, 'hickoryDesktopFolderPicker', { value: true });"));
        scripts().insert(pickerCapability);
    }

protected:
    bool acceptNavigationRequest(const QUrl &url, NavigationType type, bool mainFrame) override
    {
        if (!mainFrame || workspacePage(url, workspace))
            return true;
        if (type == NavigationTypeLinkClicked && !sameOrigin(url, workspace))
            openExternalLink(url);
        return false;
    }

private:
    const QUrl workspace;
};

class WorkspaceWindow final : public KMainWindow {
public:
    WorkspaceWindow(QWebEngineProfile &profile, const QUrl &workspace)
        : profile(profile), workspace(workspace)
    {
        setAttribute(Qt::WA_DeleteOnClose);
        setWindowTitle(tr("Hickory Docs"));
        setWindowIcon(QIcon::fromTheme(QStringLiteral("accessories-text-editor")));
        resize(1120, 760);

        view = new QWebEngineView(this);
        auto *page = new WorkspacePage(profile, workspace, view);
        view->setPage(page);
        setCentralWidget(view);
        connect(page, &QWebEnginePage::windowCloseRequested, this, [this] {
            closeApproved = true;
            QTimer::singleShot(0, this, [this] { close(); });
        });
        connect(page, &QWebEnginePage::newWindowRequested, this, [this](QWebEngineNewWindowRequest &request) {
            if (!request.isUserInitiated())
                return;
            const auto url = request.requestedUrl();
            if (workspacePage(url, this->workspace) && url.query() == QStringLiteral("open-directory") && !url.hasFragment()) {
                QTimer::singleShot(0, this, [this] {
                    const auto folder = pickDirectory(this);
                    if (!folder.isEmpty() && !QProcess::startDetached(QCoreApplication::applicationFilePath(),
                            {QStringLiteral("--port"), QStringLiteral("0"), QStringLiteral("--"), folder})) {
                        QMessageBox::warning(this, tr("Could not open directory"), tr("Try File → Open Directory again."));
                    }
                });
            } else if (workspacePage(url, this->workspace) && QUrlQuery(url).hasQueryItem(QStringLiteral("blank-window"))) {
                if (!QProcess::startDetached(QCoreApplication::applicationFilePath(), {QStringLiteral("--blank-window")})) {
                    QMessageBox::warning(this, tr("Could not open another window"), tr("Try File → New Window again."));
                }
            } else if (workspacePage(url, this->workspace)) {
                auto *window = new WorkspaceWindow(this->profile, this->workspace);
                request.openIn(window->view->page());
                window->show();
            } else if (!sameOrigin(url, this->workspace)) {
                openExternalLink(url);
            }
        });
        connect(page, &QWebEnginePage::permissionRequested, this, [this](QWebEnginePermission permission) {
            if (permission.permissionType() == QWebEnginePermission::PermissionType::ClipboardReadWrite
                && sameOrigin(permission.origin(), this->workspace)
                && QMessageBox::question(this, tr("Clipboard access"),
                    tr("Allow this workspace to access the clipboard for this editor request?"),
                    QMessageBox::Yes | QMessageBox::No, QMessageBox::No) == QMessageBox::Yes) {
                permission.grant();
            } else {
                permission.deny();
            }
        });
        connect(page, &QWebEnginePage::loadFinished, this, [this](bool ok) {
            if (!ok && !loaded) {
                closeApproved = true;
                setCentralWidget(new QLabel(
                    tr("The workspace could not load. Reopen Hickory and check that the workspace host is running."),
                    this));
            }
            loaded = loaded || ok;
        });
        connect(page, &QWebEnginePage::renderProcessTerminated, this, [this] {
            closeApproved = true;
            setCentralWidget(new QLabel(
                tr("The editor renderer stopped. Reopen Hickory and use Editor sessions to recover a draft."),
                this));
        });
    }

    void open() { view->load(workspace); }

protected:
    void closeEvent(QCloseEvent *event) override
    {
        if (closeApproved || !loaded) {
            KMainWindow::closeEvent(event);
        } else {
            event->ignore();
            view->page()->triggerAction(QWebEnginePage::RequestClose);
        }
    }

private:
    QWebEngineProfile &profile;
    const QUrl workspace;
    QWebEngineView *view = nullptr;
    bool loaded = false;
    bool closeApproved = false;
};
</ui:copy>

## Start the workspace host and retain the desktop profile

The host reports its port after it has attached the engine. A fixed loopback
origin lets the editor find its settings and recovery records at the next launch.
Use separate profiles for simultaneous hosts on separate ports.
The child owns the connection and leaves it on stdin closure. KDE closes the
last window before the child exits, so a cancelled editor close keeps the session.
Clipboard reads receive a native prompt at the time of the shared editor's
request. Downloads use a save dialog and the renderer gets no filesystem bridge.

A blank launch offers a folder picker and an existing server. A folder still
starts the bundled workspace host. A server must serve Simple Browser on its
own origin. Load it in a separate, temporary browser profile and preserve
the editor's navigation and permission boundaries.

<ui:copy id="linux-simple-startup">
int showError(const QString &message)
{
    QMessageBox dialog(QMessageBox::Critical, QObject::tr("Hickory Simple"), message);
    dialog.setTextFormat(Qt::PlainText);
    dialog.exec();
    return 1;
}

void configureProfile(QWebEngineProfile &profile)
{
    profile.setPersistentPermissionsPolicy(QWebEngineProfile::PersistentPermissionsPolicy::AskEveryTime);
    QObject::connect(&profile, &QWebEngineProfile::downloadRequested,
        &profile, [](QWebEngineDownloadRequest *download) {
            const auto chosen = QFileDialog::getSaveFileName(QApplication::activeWindow(),
                QObject::tr("Save download"), QDir::home().filePath(download->suggestedFileName()));
            if (chosen.isEmpty()) {
                download->cancel();
                return;
            }
            const QFileInfo path(chosen);
            download->setDownloadDirectory(path.absolutePath());
            download->setDownloadFileName(path.fileName());
            download->accept();
        });
}

QUrl editorUrl(int port, const QString &file)
{
    QUrl url(QStringLiteral("http://127.0.0.1:%1/simple.html").arg(port));
    if (!file.isEmpty()) {
        url.setQuery(QStringLiteral("path=") + QString::fromLatin1(QUrl::toPercentEncoding(file)));
    }
    return url;
}

bool chooseWorkspace(QString &folder, QUrl &server, bool allowServer)
{
    QDialog welcome;
    welcome.setWindowTitle(QObject::tr("Hickory Docs"));
    auto *layout = new QVBoxLayout(&welcome);
    auto *title = new QLabel(QObject::tr("Welcome to Hickory Docs"), &welcome);
    auto font = title->font();
    font.setBold(true);
    font.setPointSizeF(font.pointSizeF() * 1.5);
    title->setFont(font);
    layout->addWidget(title);
    auto *openFolder = new QPushButton(QObject::tr("Open Directory…"), &welcome);
    openFolder->setDefault(true);
    layout->addWidget(openFolder);
    QObject::connect(openFolder, &QPushButton::clicked, &welcome, [&] {
        folder = pickDirectory(&welcome);
        if (!folder.isEmpty()) welcome.accept();
    });
    if (allowServer) {
        auto *connectServer = new QPushButton(QObject::tr("Connect to a Workspace Server…"), &welcome);
        layout->addWidget(connectServer);
        QObject::connect(connectServer, &QPushButton::clicked, &welcome, [&] {
            bool accepted = false;
            const auto address = QInputDialog::getText(&welcome, QObject::tr("Connect to a Workspace Server"),
                QObject::tr("Workspace server address"), QLineEdit::Normal, {}, &accepted);
            if (!accepted) return;
            QUrl url(address.trimmed(), QUrl::StrictMode);
            if (!url.isValid() || url.host().isEmpty() || url.authority().contains(QLatin1Char('@'))
                || (url.scheme() != QStringLiteral("http") && url.scheme() != QStringLiteral("https"))
                || (!url.path().isEmpty() && url.path() != QStringLiteral("/"))
                || url.hasQuery() || url.hasFragment() || url.port() == 0) {
                QMessageBox::warning(&welcome, QObject::tr("Invalid workspace address"),
                    QObject::tr("Enter the server’s HTTP or HTTPS address, without a path or credentials."));
                return;
            }
            url.setPath(QStringLiteral("/simple.html"));
            server = url;
            welcome.accept();
        });
    }
    auto *buttons = new QDialogButtonBox(QDialogButtonBox::Cancel, &welcome);
    QObject::connect(buttons, &QDialogButtonBox::rejected, &welcome, &QDialog::reject);
    layout->addWidget(buttons);
    return welcome.exec() == QDialog::Accepted;
}

bool resolveLocalTarget(QString &workspace, QString &file)
{
    const QFileInfo input(workspace);
    if (input.isFile() && file.isEmpty()) {
        file = input.fileName();
        workspace = input.absolutePath();
    }
    if (!QFileInfo(workspace).isDir())
        return false;
    workspace = QFileInfo(workspace).canonicalFilePath();
    if (workspace.isEmpty())
        return false;
    if (file.isEmpty())
        return true;

    const QDir root(workspace);
    const QFileInfo selected(root.filePath(file));
    const auto canonical = selected.canonicalFilePath();
    const auto relative = root.relativeFilePath(canonical);
    if (canonical.isEmpty() || !selected.isFile() || QDir::isAbsolutePath(relative)
        || relative == QStringLiteral("..") || relative.startsWith(QStringLiteral("../"))) {
        return false;
    }
    file = relative;
    return true;
}
} // namespace

int main(int argc, char **argv)
{
    QApplication app(argc, argv);
    KAboutData about(QStringLiteral("hickory-simple"), QObject::tr("Hickory Simple"),
        QStringLiteral(HICKORY_VERSION), QObject::tr("Hickory workspace editor"),
        KAboutLicense::GPL_V3);
    about.setDesktopFileName(QStringLiteral("com.hickorydocs.Simple"));
    KAboutData::setApplicationData(about);

    QCommandLineParser parser;
    parser.setApplicationDescription(QObject::tr("Open a Hickory workspace."));
    parser.addHelpOption();
    parser.addVersionOption();
    parser.addPositionalArgument(QStringLiteral("workspace"), QObject::tr("Local folder or file to open."));
    const QCommandLineOption fileOption(QStringLiteral("file"),
        QObject::tr("Open this workspace-relative file."), QStringLiteral("path"));
    const QCommandLineOption connectOption(QStringLiteral("connect"),
        QObject::tr("Use an existing Hickory host forwarded to this loopback port."), QStringLiteral("port"));
    const QCommandLineOption portOption(QStringLiteral("port"),
        QObject::tr("Choose the local host port (use 0 for a temporary port)."), QStringLiteral("port"),
        QString::number(defaultPort));
    const QCommandLineOption blankOption(QStringLiteral("blank-window"),
        QObject::tr("Choose a workspace in a new window."));
    parser.addOption(blankOption);
    parser.addOption(fileOption);
    parser.addOption(connectOption);
    parser.addOption(portOption);
    parser.process(app);
    if (parser.positionalArguments().size() > 1
        || (parser.isSet(connectOption) && (parser.isSet(portOption) || !parser.positionalArguments().isEmpty()))) {
        return showError(QObject::tr("Pass one local folder or file, or use --connect with a loopback port."));
    }

    const bool existing = parser.isSet(connectOption);
    bool validPort = false;
    const auto defaultToFreePort = parser.isSet(blankOption) && !existing && !parser.isSet(portOption);
    const int requestedPort = (defaultToFreePort ? QStringLiteral("0")
        : parser.value(existing ? connectOption : portOption)).toInt(&validPort);
    if (!validPort || requestedPort < (existing ? 1 : 0) || requestedPort > 65535) {
        return showError(existing ? QObject::tr("Use a loopback port between 1 and 65535.")
                                  : QObject::tr("Use a local host port between 0 and 65535."));
    }

    QString file = parser.value(fileOption);
    std::unique_ptr<QWebEngineProfile> profile;
    const auto open = [&profile](const QUrl &url, bool temporary) {
        profile = temporary ? std::make_unique<QWebEngineProfile>()
            : std::make_unique<QWebEngineProfile>(QStringLiteral("Workspace-%1").arg(url.port()));
        configureProfile(*profile);
        auto *window = new WorkspaceWindow(*profile, url);
        window->open();
        window->show();
    };
    if (existing) {
        open(editorUrl(requestedPort, file), false);
        return app.exec();
    }

    auto workspace = parser.positionalArguments().value(0);
    if (workspace.isEmpty()) {
        QUrl server;
        if (!chooseWorkspace(workspace, server, file.isEmpty()))
            return 0;
        if (!server.isEmpty()) {
            open(server, true);
            return app.exec();
        }
    }
    if (!resolveLocalTarget(workspace, file)) {
        return showError(QObject::tr("Cannot open the requested workspace or file. Choose an existing folder and a file inside it."));
    }

    const QDir bin(QCoreApplication::applicationDirPath());
    QProcess host;
    QTimer deadline;
    deadline.setSingleShot(true);
    bool ready = false;
    QByteArray stdoutBuffer;
    QByteArray diagnostics;
    QObject::connect(&host, &QProcess::readyReadStandardOutput, &app, [&] {
        stdoutBuffer += host.readAllStandardOutput();
        if (ready) {
            stdoutBuffer.clear();
            return;
        }
        while (stdoutBuffer.contains('\n')) {
            const auto end = stdoutBuffer.indexOf('\n');
            const auto line = stdoutBuffer.left(end).trimmed();
            stdoutBuffer.remove(0, end + 1);
            const QByteArray prefix("HICKORY_SIMPLE_PORT ");
            if (!line.startsWith(prefix))
                continue;
            bool valid = false;
            const int port = line.mid(prefix.size()).toInt(&valid);
            if (valid && port >= 1 && port <= 65535 && (requestedPort == 0 || port == requestedPort)) {
                ready = true;
                deadline.stop();
                open(editorUrl(port, file), requestedPort == 0);
                stdoutBuffer.clear();
                break;
            }
        }
        if (stdoutBuffer.size() > 4096)
            stdoutBuffer.clear();
    });
    QObject::connect(&host, &QProcess::readyReadStandardError, &app, [&] {
        diagnostics = (diagnostics + host.readAllStandardError()).right(4096);
    });
    QObject::connect(&host, &QProcess::errorOccurred, &app, [&](QProcess::ProcessError error) {
        if (error != QProcess::FailedToStart)
            return;
        deadline.stop();
        showError(QObject::tr("Cannot start the Hickory workspace host. Reinstall Hickory Simple and try again.\n\n%1")
            .arg(host.errorString()));
        app.exit(1);
    });
    QObject::connect(&host, qOverload<int, QProcess::ExitStatus>(&QProcess::finished), &app,
        [&](int, QProcess::ExitStatus) {
            deadline.stop();
            showError(QObject::tr("The Hickory workspace host stopped. Reopen Hickory after checking the reported error.\n\n%1")
                .arg(QString::fromUtf8(diagnostics)));
            if (!ready)
                app.exit(1);
        });
    QObject::connect(&deadline, &QTimer::timeout, &app, [&] {
        showError(QObject::tr("The workspace host did not become ready. Check the workspace and reopen Hickory."));
        app.exit(1);
    });
    host.setProgram(bin.absoluteFilePath(QStringLiteral(HICKORY_HOST_PATH)));
    host.setArguments({QStringLiteral("--linux-host"), workspace,
        bin.absoluteFilePath(QStringLiteral(HICKORY_WEB_PATH)), QString::number(requestedPort)});
    host.start();
    deadline.start(25000);
    const int status = app.exec();
    host.disconnect(&app);
    host.closeWriteChannel();
    if (host.state() != QProcess::NotRunning && !host.waitForFinished(7000)) {
        host.kill();
        host.waitForFinished(1000);
    }
    return status;
}
</ui:copy>

<ui:file path="apps/linux-simple/CMakeLists.txt"><ui:paste select="#linux-simple-build"/></ui:file>
<ui:file path="apps/linux-simple/com.hickorydocs.Simple.desktop"><ui:paste select="#linux-simple-desktop-entry"/></ui:file>
<ui:file path="apps/linux-simple/main.cpp"><ui:paste select="#linux-simple-workspace-window"/><ui:paste select="#linux-simple-startup"/></ui:file>

## Attach the same engine and serve the compiled workspace

The desktop host serves static files and the existing API from one address,
including event and terminal WebSockets. Exact Host and Origin checks protect
the privileged workspace surface from cross-origin requests and host-name
rebinding. The native app sends no path from the editor directly to the OS.

A packaged browser-based desktop supplies its compiled editor assets. The
native macOS development preview needs only the engine API, so it can omit
assets and launch an independent host for a picked folder. Both modes use the
same loopback request checks and close the host when its owning pipe closes.

<ui:copy id="linux-simple-host">
use super::interrupted;
use anyhow::{Context, Result, ensure};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use hickory_cli::{ExecutorChoice, engine};
use std::{io::Write, path::Path};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

fn admits(headers: &HeaderMap, authority: &str, origin: &str) -> bool {
    headers
        .get(header::HOST)
        .is_some_and(|host| host == authority)
        && headers
            .get(header::ORIGIN)
            .is_none_or(|found| found == origin)
        && headers
            .get("sec-fetch-site")
            .is_none_or(|site| site == "same-origin" || site == "none")
}

async fn local_request(request: Request, next: Next, authority: &str, origin: &str) -> Response {
    if !admits(request.headers(), authority, origin) {
        return StatusCode::FORBIDDEN.into_response();
    }
    next.run(request).await
}

pub(super) async fn run(args: &[String]) -> Result<()> {
    let (workspace, assets, port) = match args {
        [workspace, assets, port] => (workspace, Some(assets), port),
        [workspace, port] => (workspace, None, port),
        _ => anyhow::bail!(
            "Usage: hickory-simple-host --workspace-host WORKSPACE [COMPILED_ASSETS] LOOPBACK_PORT"
        ),
    };
    let root = Path::new(workspace)
        .canonicalize()
        .with_context(|| format!("Open workspace {workspace}. Choose an existing folder."))?;
    ensure!(
        root.is_dir(),
        "Choose a workspace folder: {}",
        root.display()
    );
    let assets = assets
        .map(|path| -> Result<_> {
            let assets = Path::new(path)
                .canonicalize()
                .context("Editor assets are missing. Rebuild or reinstall Hickory Simple.")?;
            ensure!(
                assets.join("simple.html").is_file(),
                "Editor assets are incomplete in {}. Rebuild or reinstall Hickory Simple.",
                assets.display()
            );
            Ok(assets)
        })
        .transpose()?;
    let port: u16 = port
        .parse()
        .context("Use a loopback port between 0 and 65535.")?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .with_context(|| {
            format!(
                "Cannot bind 127.0.0.1:{port}. Close the other Hickory window or choose --port."
            )
        })?;
    let address = listener.local_addr()?;
    let authority = address.to_string();
    let origin = format!("http://{authority}");

    let mut attach = engine::Attach::new(root, ExecutorChoice::Local);
    attach.callback = Some(origin.clone());
    attach.callback_token = engine::window_token();
    attach.key_store_path = hickory_agent::KeyStore::desktop_path();
    // The development launcher can hand off its worker. A packaged host can
    // supply its own even when another executable started the engine.
    attach.python_runtime_path = std::env::var_os("HICKORY_AGENT_PYTHON").map(Into::into)
        .or_else(hickory_cli::serve::bundled_runtime);
    let token = attach.callback_token.clone();
    let (closed, owner) = tokio::sync::oneshot::channel();
    // A dedicated thread can wait for pipe EOF without blocking Tokio shutdown
    // on an open terminal after a signal or server failure.
    std::thread::spawn(move || {
        let _ = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
        let _ = closed.send(());
    });
    tokio::pin!(owner);
    let connection = tokio::select! {
        result = engine::connect(attach) => result?,
        _ = &mut owner => return Ok(()),
        result = interrupted() => return result,
    };
    let guard = connection.guard();
    let router = engine::client_router(connection, None, token, None);
    let router = match assets {
        Some(assets) => router.fallback_service(ServeDir::new(assets)),
        None => router,
    };
    let router = router.layer(middleware::from_fn(move |request, next| {
        let authority = authority.clone();
        let origin = origin.clone();
        async move { local_request(request, next, &authority, &origin).await }
    }));
    let result = async {
        println!("HICKORY_SIMPLE_PORT {}", address.port());
        std::io::stdout().flush()?;
        tokio::select! {
            result = axum::serve(listener, router) => Ok(result?),
            _ = &mut owner => Ok(()),
            result = interrupted() => result,
        }
    }
    .await;
    let detached = guard.detach().await;
    result.and(detached)
}
</ui:copy>

The origin witness includes a same-site request from a different port. A foreign
site also cannot reach the workspace by resolving its own host name to loopback.

<ui:copy id="linux-simple-host-test">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_host_admits_only_its_own_origin() {
        let authority = "127.0.0.1:38174";
        let origin = "http://127.0.0.1:38174";
        let mut headers = HeaderMap::new();
        assert!(!admits(&headers, authority, origin));
        headers.insert(header::HOST, authority.parse().unwrap());
        assert!(admits(&headers, authority, origin));
        headers.insert(header::ORIGIN, origin.parse().unwrap());
        headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
        assert!(admits(&headers, authority, origin));
        headers.insert(header::ORIGIN, "https://another.example".parse().unwrap());
        assert!(!admits(&headers, authority, origin));
        headers.remove(header::ORIGIN);
        headers.insert("sec-fetch-site", "same-site".parse().unwrap());
        assert!(!admits(&headers, authority, origin));
        headers.remove("sec-fetch-site");
        headers.insert(header::HOST, "other.example:38174".parse().unwrap());
        assert!(!admits(&headers, authority, origin));
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/examples/simple_dev/linux_host.rs"><ui:paste select="#linux-simple-host"/><ui:paste select="#linux-simple-host-test"/></ui:file>

## Open, save and run a terminal command in the mounted editor

The focused Linux test launches the built application with a private Qt profile,
workspace and engine directory. It opens a file whose name exercises URL
encoding, saves through the on-screen File menu, checks generated output stays
read-only, and exercises a real PTY round trip. A
loopback-only DevTools port is enabled for
this test process so Playwright can drive the mounted editor. Host-only mode
uses the same compiled assets and engine with Chromium on another host OS.

The same rendered witness checks the shadow-root boundary in both system color
schemes. Table furniture must inherit the surrounding scheme and retain readable
contrast; changing the scheme must keep the mounted widget and source intact.
The graph and equation exercise the other shared renderers beside that table.

<ui:copy id="linux-simple-smoke">
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { access, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { setTimeout as delay } from "node:timers/promises";
import { chromium, expect } from "../apps/web/node_modules/@playwright/test/index.mjs";

const hostOnly = process.argv.includes("--host-only");
const binary = resolve(hostOnly
  ? "target/debug/examples/simple_dev"
  : "target/simple-linux/app/bin/hickory-simple");
await access(binary);
const temporary = await mkdtemp(join(tmpdir(), "hickory-linux-"));
const project = join(temporary, "workspace");
await mkdir(project);
const file = "note + #é.txt";
const source = join(project, file);
await writeFile(source, "first line\n");
await writeFile(join(project, "sibling.txt"), "same workspace\n");
await writeFile(join(project, "author.md"), [
  '<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">',
  '<hick:file path="generated.txt">Generated source',
  "</hick:file>",
  "</hick:doc>",
  "",
].join("\n"));
await writeFile(join(project, "generated.txt"), "Generated source\n");
const visualSource = [
  '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">',
  '# Platform colors', '', '| Product | Quantity |', '| --- | --- |', '| Notebook | 2 |', '',
  '<h:math>x^2 + y^2</h:math>', '',
  '<h:diagram renderer="graph">',
  '{"topology":{"nodes":[{"id":"note"}],"edges":[]},"layout":{"note":{"x":20,"y":20,"w":130,"h":50}}}',
  '</h:diagram>', '</h:doc>', '',
].join("\n");
await writeFile(join(project, "colors.md"), visualSource);

async function sparePort() {
  const server = createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const port = server.address().port;
  await new Promise((resolve, reject) => server.close(error => error ? reject(error) : resolve()));
  return port;
}

async function until(read, description) {
  const deadline = Date.now() + 25000;
  do {
    const value = await read();
    if (value) return value;
    await delay(100);
  } while (Date.now() < deadline);
  throw new Error(`Timed out waiting for ${description}`);
}

const debug = hostOnly ? null : `http://127.0.0.1:${await sparePort()}`;
const settings = {
  ...process.env,
  HICKORY_STATE_DIR: join(temporary, ".engine"),
  XDG_DATA_HOME: join(temporary, ".data"),
  XDG_CACHE_HOME: join(temporary, ".cache"),
  XDG_CONFIG_HOME: join(temporary, ".config"),
  SHELL: "/bin/sh",
};
if (debug) settings.QTWEBENGINE_REMOTE_DEBUGGING = new URL(debug).host;
const args = hostOnly
  ? ["--linux-host", project, resolve("target/simple-linux/web"), "0"]
  : ["--port", "0", source];
const app = spawn(binary, args, { env: settings, stdio: ["pipe", "pipe", "inherit"] });
const stopped = once(app, "exit");
let port;
const output = createInterface({ input: app.stdout });
output.on("line", line => {
  const match = /^HICKORY_SIMPLE_PORT (\d+)$/.exec(line);
  if (match) port = Number(match[1]);
});
let browser;

async function menu(page, group, action) {
  await page.locator("nav").getByText(group, { exact: true }).click();
  await page.getByRole("button", { name: action, exact: true }).click();
}

try {
  let page;
  if (hostOnly) {
    await until(() => port, "the workspace host");
    browser = await chromium.launch({ headless: true });
    page = await browser.newPage({ viewport: { width: 1100, height: 750 } });
    const url = new URL(`http://127.0.0.1:${port}/simple.html`);
    url.searchParams.set("path", file);
    await page.goto(url.href);
  } else {
    await until(() => fetch(`${debug}/json/version`, { signal: AbortSignal.timeout(1000) })
      .then(response => response.ok).catch(() => false), "Qt WebEngine");
    browser = await chromium.connectOverCDP(debug, { noDefaults: true });
    page = await until(() => browser.contexts().flatMap(context => context.pages())
      .find(candidate => new URL(candidate.url()).pathname === "/simple.html"), "the workspace window");
  }
  const editor = page.locator(".cm-content").and(page.locator(`[aria-label=${JSON.stringify(file)}]`));
  const tree = page.locator('[aria-label="Files"]');
  await expect(editor).toHaveText("first line");
  await expect(editor).toHaveAttribute("contenteditable", "true");
  await expect(tree.locator('[title="sibling.txt"]')).toBeVisible();

  const endpoint = new URL("/api/files", page.url());
  const refused = await fetch(endpoint, { headers: { Origin: "https://other.example" } });
  assert.equal(refused.status, 403);

  await editor.press("End");
  await page.keyboard.insertText(" from the Linux workspace");
  assert.equal(await readFile(source, "utf8"), "first line\n");
  await menu(page, "File", "Save");
  await until(async () => await readFile(source, "utf8") === "first line from the Linux workspace\n",
    "the guarded save");

  await tree.locator('[title="generated.txt"]').click();
  const generated = page.locator('.cm-content[aria-label="generated.txt"]');
  await expect(generated).toHaveText("Generated source");
  await expect(generated).toHaveAttribute("contenteditable", "false");
  await generated.press("End");
  await page.keyboard.insertText(" attempted change");
  await expect(generated).toHaveText("Generated source");
  assert.equal(await readFile(join(project, "generated.txt"), "utf8"), "Generated source\n");

  await menu(page, "View", "Terminal");
  const terminal = page.locator(".terminal-screen .xterm-helper-textarea");
  await terminal.focus();
  await page.keyboard.insertText("printf 'hickory-linux-terminal\\n'");
  await terminal.press("Enter");
  await until(async () => (await page.locator(".terminal-screen .xterm-rows").innerText())
    .split("\n").some(line => line.trim() === "hickory-linux-terminal"), "terminal output");
  await tree.locator('[title="colors.md"]').click();
  const table = page.locator(".table-panel__grid");
  await expect(table).toBeVisible();
  await expect(page.locator('[data-testid="graph-node-note"]')).toBeVisible();
  await expect(page.locator(".katex")).toBeVisible();
  await table.evaluate(element => { element.dataset.retainedThemeWitness = "yes"; });
  await page.locator('.cm-content[aria-label="colors.md"]').focus();
  for (const colorScheme of ["light", "dark", "light"]) {
    await page.emulateMedia({ colorScheme });
    const colors = await table.evaluate(element => {
      const host = element.getRootNode().host;
      const probe = document.createElement("span");
      probe.style.cssText = "color:CanvasText;background:Canvas";
      document.body.append(probe);
      const outer = getComputedStyle(probe), inner = getComputedStyle(host);
      const cell = getComputedStyle(element.querySelector("th"));
      const result = { outer: outer.color, inner: inner.color, scheme: inner.colorScheme,
        outerScheme: getComputedStyle(document.documentElement).colorScheme,
        foreground: cell.color, background: cell.backgroundColor };
      probe.remove(); return result;
    });
    assert.equal(colors.inner, colors.outer);
    assert.equal(colors.scheme, colors.outerScheme);
    const gutter = await page.locator('.editor-cm-host .cm-gutters').first().evaluate(element => getComputedStyle(element).backgroundColor);
    const caret = await page.locator('.editor-cm-host .cm-cursor').first().evaluate(element => getComputedStyle(element).borderLeftColor);
    assert.equal(caret, colors.outer);
    const gutterChannel = Number(gutter.match(/[\d.]+/)[0]) * (gutter.startsWith("color(srgb") ? 255 : 1);
    assert.ok(colorScheme === "dark" ? gutterChannel < 80 : gutterChannel > 180, gutter);
    const luminance = color => {
      const channels = color.match(/[\d.]+/g).slice(0, 3).map(Number).map(value => {
        const channel = color.startsWith("color(srgb") ? value : value / 255;
        return channel <= .04045 ? channel / 12.92 : ((channel + .055) / 1.055) ** 2.4;
      });
      return channels[0] * .2126 + channels[1] * .7152 + channels[2] * .0722;
    };
    const a = luminance(colors.foreground), b = luminance(colors.background);
    assert.ok((Math.max(a, b) + .05) / (Math.min(a, b) + .05) >= 4.5, JSON.stringify(colors));
    await expect(table).toHaveAttribute("data-retained-theme-witness", "yes");
  }
  assert.equal(await readFile(join(project, "colors.md"), "utf8"), visualSource);
  console.log(hostOnly ? "Shared Linux host smoke passed." : "KDE workspace smoke passed.");
} finally {
  try {
    if (browser) await browser.close();
  } finally {
    app.stdin.end();
    if (!hostOnly) app.kill("SIGTERM");
    const exited = await Promise.race([stopped.then(() => true), delay(9000, false, { ref: false })]);
    if (!exited) {
      app.kill("SIGKILL");
      await stopped;
    }
    await rm(temporary, { recursive: true, force: true });
  }
}
</ui:copy>

<ui:file path="scripts/test-simple-linux.mjs"><ui:paste select="#linux-simple-smoke"/></ui:file>

## Executed checks

The origin admission test and shared host smoke passed against the compiled
editor. The smoke opened the requested file, saved through the File menu,
preserved generated output, and ran a command in the real terminal.

Debian 13 arm64 linked the C++ window with Qt WebEngine 6.8.2 and KDE
Frameworks 6.13. Under Xvfb, the window opened a filename containing spaces,
a plus sign, a hash and Unicode, saved an edit, returned real PTY output, and
opened a second KDE window. These checks used a forwarded macOS engine.
A launcher fixture also exercised the native child-process readiness path.
The end-to-end Linux engine build, CMake install, Wayland, and native input
method acceptance remain open.

</ui:doc>
