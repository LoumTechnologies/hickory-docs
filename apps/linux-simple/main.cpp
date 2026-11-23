#include <KAboutData>
#include <KMainWindow>

#include <QApplication>
#include <QCloseEvent>
#include <QCommandLineParser>
#include <QDesktopServices>
#include <QDir>
#include <QFileDialog>
#include <QFileInfo>
#include <QIcon>
#include <QLabel>
#include <QMessageBox>
#include <QProcess>
#include <QTimer>
#include <QWebEngineDownloadRequest>
#include <QWebEngineNewWindowRequest>
#include <QWebEnginePage>
#include <QWebEnginePermission>
#include <QWebEngineProfile>
#include <QWebEngineView>

#include <memory>

namespace {
constexpr int defaultPort = 38174;

bool sameOrigin(const QUrl &url, const QUrl &workspace)
{
    return url.scheme() == workspace.scheme() && url.host() == workspace.host()
        && url.port() == workspace.port() && url.userInfo().isEmpty();
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

class WorkspacePage final : public QWebEnginePage {
public:
    WorkspacePage(QWebEngineProfile &profile, const QUrl &workspace, QObject *parent)
        : QWebEnginePage(&profile, parent), workspace(workspace)
    {
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
        setWindowTitle(tr("Hickory Simple"));
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
            if (workspacePage(url, this->workspace)) {
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
    const int requestedPort = parser.value(existing ? connectOption : portOption).toInt(&validPort);
    if (!validPort || requestedPort < (existing ? 1 : 0) || requestedPort > 65535) {
        return showError(existing ? QObject::tr("Use a loopback port between 1 and 65535.")
                                  : QObject::tr("Use a local host port between 0 and 65535."));
    }

    QString file = parser.value(fileOption);
    std::unique_ptr<QWebEngineProfile> profile;
    const auto open = [&profile, &file, requestedPort](int port) {
        profile = requestedPort == 0 ? std::make_unique<QWebEngineProfile>()
            : std::make_unique<QWebEngineProfile>(QStringLiteral("Workspace-%1").arg(port));
        configureProfile(*profile);
        auto *window = new WorkspaceWindow(*profile, editorUrl(port, file));
        window->open();
        window->show();
    };
    if (existing) {
        open(requestedPort);
        return app.exec();
    }

    auto workspace = parser.positionalArguments().value(0);
    if (workspace.isEmpty()) {
        workspace = QFileDialog::getExistingDirectory(nullptr, QObject::tr("Open workspace"), QDir::currentPath());
        if (workspace.isEmpty())
            return 0;
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
                open(port);
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
