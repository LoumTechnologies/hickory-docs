<web:doc xmlns:web="http://www.hickorydocs.com/1.0">
# Select reproducible tools

The framework targets .NET 10 and Python 3. Libraries are ordinary project references; the app can be published as a standard .NET application. No source generator, middleware container or framework-specific compiler is required.

## Sdk

Select the installed stable .NET 10 feature band even when a preview SDK is also installed.

<web:copy id="ew-sdk">
{"sdk":{"version":"10.0.400","rollForward":"latestPatch"}}
</web:copy>

## Build budget

Keep restores and project-reference builds serial. This small project gains little from spawning a build worker per CPU; bounded build concurrency also keeps compiler-fixture runs from exhausting host workers.

<web:copy id="ew-build-budget">
<Project><PropertyGroup><BuildInParallel>false</BuildInParallel><RestoreDisableParallel>true</RestoreDisableParallel></PropertyGroup></Project>
</web:copy>

## Workflow

Run the focused contract on all three desktop/server host families and browser forms on Linux. Windows uses Python’s bundled SQLite DLL rather than downloading an unpinned native binary. Pushes to master produce explicitly unstable package artifacts, without publishing a stable release or changing the parent release process.

<web:copy id="ew-workflow">
name: Explicit Web focused checks
on:
  push:
    branches: [master]
    paths: ['experiments/explicit-web/**', '.hickory-source/mechanisms/explicit-web/**', '.github/workflows/explicit-web.yml', 'Core/**', 'Data/**', 'Security/**', 'Hosting/**', 'Server/**', 'Baseline/**', 'Checks/**', 'tests/**', 'tools/**']
  pull_request:
    paths: ['experiments/explicit-web/**', '.hickory-source/mechanisms/explicit-web/**', '.github/workflows/explicit-web.yml', 'Core/**', 'Data/**', 'Security/**', 'Hosting/**', 'Server/**', 'Baseline/**', 'Checks/**', 'tests/**', 'tools/**']
  workflow_dispatch:
permissions:
  contents: read
jobs:
  contract:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '10.0.4xx'
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Expose Python's bundled SQLite on Windows
        if: runner.os == 'Windows'
        shell: pwsh
        run: 'Join-Path $env:pythonLocation "DLLs" | Out-File -FilePath $env:GITHUB_PATH -Append'
      - name: Run the same focused contract as the local hook
        run: python -c "import pathlib,subprocess,sys; p=pathlib.Path('experiments/explicit-web'); p=p if p.is_dir() else pathlib.Path('.'); subprocess.run([sys.executable,str(p/'tools/ci.py')],check=True)"
      - name: Build unstable library packages
        if: github.event_name == 'push'
        run: python -c "import pathlib,subprocess; p=pathlib.Path('experiments/explicit-web'); p=p if p.is_dir() else pathlib.Path('.'); [subprocess.run(['dotnet','pack',str(p/n/(n+'.csproj')),'-c','Release','-p:PackageVersion=0.1.0-unstable.${{ github.run_number }}','-o',str(p/'target/packages'),'-m:1'],check=True) for n in ['Core','Data','Security','Hosting']]"
      - uses: actions/upload-artifact@v4
        if: github.event_name == 'push'
        with:
          name: explicit-web-unstable-${{ matrix.os }}
          path: '**/target/packages/*.nupkg'
  browser:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '10.0.4xx'
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - run: python -c "import pathlib,subprocess,sys; p=pathlib.Path('experiments/explicit-web'); p=p if p.is_dir() else pathlib.Path('.'); subprocess.run([sys.executable,str(p/'tools/ci.py'),'--browser'],check=True)"
</web:copy>

## Task interface

This is a new interface: the recipe migration inventory is empty. Each addition wraps an existing focused command. The primary build artifact is the four library packages; browser checks remain a named prerequisite-dependent flow. The parent justfile is not modified.

<web:copy id="ew-task-interface">
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Show commands without building or launching.
default:
    just --list

# Restore the locked .NET project graph; browser dependencies are optional.
devenv:
    dotnet restore Checks/Checks.csproj --locked-mode --disable-parallel
    dotnet restore Baseline/Baseline.csproj --locked-mode --disable-parallel

# Run the reference app on an allocated local port; Ctrl+C stops its server.
dev:
    python3 tools/dev.py

# Check file-size limits, Python syntax and the C# build.
check:
    python3 tools/check.py

# Run focused runtime, HTTP and compiler witnesses; excludes browser provisioning.
test:
    python3 tools/ci.py

# Run actual browser forms; requires Node, Playwright and a browser.
e2e:
    python3 tests/browser.py

# All local gates, including the separately provisioned browser witness.
ci: test e2e

# Produce the framework's four unstable NuGet packages without publishing.
build:
    dotnet pack Core/Core.csproj -c Release -p:PackageVersion=0.1.0-unstable.local -o target/packages -m:1
    dotnet pack Data/Data.csproj -c Release -p:PackageVersion=0.1.0-unstable.local -o target/packages -m:1
    dotnet pack Security/Security.csproj -c Release -p:PackageVersion=0.1.0-unstable.local -o target/packages -m:1
    dotnet pack Hosting/Hosting.csproj -c Release -p:PackageVersion=0.1.0-unstable.local -o target/packages -m:1
</web:copy>

## Lock core

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-core">
{
  "version": 1,
  "dependencies": {
    "net10.0": {}
  }
}</web:copy>

## Lock data

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-data">
{
  "version": 1,
  "dependencies": {
    "net10.0": {
      "ExplicitWeb.Core": {
        "type": "Project"
      }
    }
  }
}</web:copy>

## Lock security

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-security">
{
  "version": 1,
  "dependencies": {
    "net10.0": {
      "ExplicitWeb.Core": {
        "type": "Project"
      },
      "ExplicitWeb.Data": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      }
    }
  }
}</web:copy>

## Lock hosting

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-hosting">
{
  "version": 1,
  "dependencies": {
    "net10.0": {
      "ExplicitWeb.Core": {
        "type": "Project"
      }
    }
  }
}</web:copy>

## Lock server

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-server">
{
  "version": 1,
  "dependencies": {
    "net10.0": {
      "ExplicitWeb.Core": {
        "type": "Project"
      },
      "ExplicitWeb.Data": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      },
      "ExplicitWeb.Hosting": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      },
      "ExplicitWeb.Security": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )",
          "ExplicitWeb.Data": "[1.0.0, )"
        }
      }
    }
  }
}</web:copy>

## Lock baseline

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-baseline">
{
  "version": 1,
  "dependencies": {
    "net10.0": {
      "ExplicitWeb.Core": {
        "type": "Project"
      },
      "ExplicitWeb.Data": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      },
      "ExplicitWeb.Hosting": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      },
      "ExplicitWeb.Security": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )",
          "ExplicitWeb.Data": "[1.0.0, )"
        }
      },
      "server": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )",
          "ExplicitWeb.Data": "[1.0.0, )",
          "ExplicitWeb.Hosting": "[1.0.0, )",
          "ExplicitWeb.Security": "[1.0.0, )"
        }
      }
    }
  }
}</web:copy>

## Lock checks

Retain the project dependency graph produced by the successful .NET restore. These projects require no external NuGet packages; locked restore checks that the graph stays aligned.

<web:copy id="ew-lock-checks">
{
  "version": 1,
  "dependencies": {
    "net10.0": {
      "ExplicitWeb.Core": {
        "type": "Project"
      },
      "ExplicitWeb.Data": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      },
      "ExplicitWeb.Hosting": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )"
        }
      },
      "ExplicitWeb.Security": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )",
          "ExplicitWeb.Data": "[1.0.0, )"
        }
      },
      "server": {
        "type": "Project",
        "dependencies": {
          "ExplicitWeb.Core": "[1.0.0, )",
          "ExplicitWeb.Data": "[1.0.0, )",
          "ExplicitWeb.Hosting": "[1.0.0, )",
          "ExplicitWeb.Security": "[1.0.0, )"
        }
      }
    }
  }
}</web:copy>

## Assemble these outputs

<web:file path="experiments/explicit-web/global.json"><web:paste select="#ew-sdk"/></web:file>
<web:file path="experiments/explicit-web/Directory.Build.props"><web:paste select="#ew-build-budget"/></web:file>
<web:file path="experiments/explicit-web/.github/workflows/explicit-web.yml"><web:paste select="#ew-workflow"/></web:file>
<web:file path=".github/workflows/explicit-web.yml"><web:paste select="#ew-workflow"/></web:file>
<web:file path="experiments/explicit-web/justfile"><web:paste select="#ew-task-interface"/></web:file>
<web:file path="experiments/explicit-web/Core/packages.lock.json"><web:paste select="#ew-lock-core"/></web:file>
<web:file path="experiments/explicit-web/Data/packages.lock.json"><web:paste select="#ew-lock-data"/></web:file>
<web:file path="experiments/explicit-web/Security/packages.lock.json"><web:paste select="#ew-lock-security"/></web:file>
<web:file path="experiments/explicit-web/Hosting/packages.lock.json"><web:paste select="#ew-lock-hosting"/></web:file>
<web:file path="experiments/explicit-web/Server/packages.lock.json"><web:paste select="#ew-lock-server"/></web:file>
<web:file path="experiments/explicit-web/Baseline/packages.lock.json"><web:paste select="#ew-lock-baseline"/></web:file>
<web:file path="experiments/explicit-web/Checks/packages.lock.json"><web:paste select="#ew-lock-checks"/></web:file>

</web:doc>
