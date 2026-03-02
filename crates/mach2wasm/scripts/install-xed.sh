#!/usr/bin/env bash
#
# Build and install Intel XED (x86 Encoder Decoder) library.
#
# This script mirrors what Remill's dependency superbuild does
# (see lifting-bits/remill dependencies/xed.cmake).
#
# Usage:
#   ./install-xed.sh [INSTALL_PREFIX]
#
# Default INSTALL_PREFIX is ${REMILL_PREFIX:-/usr/local}.
# Requires: python3, git, C/C++ compiler (clang or gcc).

set -euo pipefail

INSTALL_PREFIX="${1:-${REMILL_PREFIX:-/usr/local}}"
WORK_DIR="$(mktemp -d)"

echo "=== Installing Intel XED to ${INSTALL_PREFIX} ==="
echo "=== Working directory: ${WORK_DIR} ==="

cleanup() {
    echo "=== Cleaning up ${WORK_DIR} ==="
    rm -rf "${WORK_DIR}"
}
trap cleanup EXIT

cd "${WORK_DIR}"

# Clone mbuild (XED's build system)
echo "=== Cloning mbuild ==="
git clone --depth 1 --branch v2024.11.04 https://github.com/intelxed/mbuild.git

# Clone XED
echo "=== Cloning XED ==="
git clone --depth 1 --branch v2025.06.08 https://github.com/intelxed/xed.git

# Detect compiler
if command -v clang &>/dev/null; then
    COMPILER="clang"
    CC="clang"
    CXX="clang++"
elif command -v gcc &>/dev/null; then
    COMPILER="gnu"
    CC="gcc"
    CXX="g++"
else
    echo "ERROR: No supported compiler found (need clang or gcc)"
    exit 1
fi

echo "=== Building XED with ${COMPILER} ==="

# Build XED
cd xed
python3 mfile.py install \
    --install-dir=install \
    --cc="${CC}" \
    --cxx="${CXX}" \
    --compiler="${COMPILER}" \
    --static

# Copy artifacts to install prefix
echo "=== Installing to ${INSTALL_PREFIX} ==="
mkdir -p "${INSTALL_PREFIX}/include" "${INSTALL_PREFIX}/lib"
cp -R install/include/* "${INSTALL_PREFIX}/include/"
cp -R install/lib/* "${INSTALL_PREFIX}/lib/"

# Generate XEDConfig.cmake (mirrors Remill's dependencies/XEDConfig.cmake.in)
mkdir -p "${INSTALL_PREFIX}/lib/cmake/XED"
cat > "${INSTALL_PREFIX}/lib/cmake/XED/XEDConfig.cmake" << 'XEDCMAKE'
if(XED_FOUND)
    return()
endif()

get_filename_component(PACKAGE_PREFIX_DIR "${CMAKE_CURRENT_LIST_DIR}/../../../" ABSOLUTE)

find_library(XED_LIBRARY xed PATHS "${PACKAGE_PREFIX_DIR}/lib" NO_CACHE REQUIRED NO_DEFAULT_PATH)
add_library(XED::XED STATIC IMPORTED)
set_target_properties(XED::XED PROPERTIES
    IMPORTED_CONFIGURATIONS "NOCONFIG"
    IMPORTED_LOCATION_NOCONFIG "${XED_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${PACKAGE_PREFIX_DIR}/include"
)

find_library(ILD_LIBRARY xed-ild PATHS "${PACKAGE_PREFIX_DIR}/lib" NO_CACHE REQUIRED NO_DEFAULT_PATH)
add_library(XED::ILD STATIC IMPORTED)
set_target_properties(XED::ILD PROPERTIES
    IMPORTED_CONFIGURATIONS "NOCONFIG"
    IMPORTED_LOCATION_NOCONFIG "${XED_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${PACKAGE_PREFIX_DIR}/include"
)

set(XED_FOUND ON)
XEDCMAKE

echo "=== Verifying installation ==="
ls -la "${INSTALL_PREFIX}/lib/libxed"* 2>/dev/null || echo "WARNING: libxed not found!"
ls -la "${INSTALL_PREFIX}/lib/cmake/XED/XEDConfig.cmake" 2>/dev/null || echo "WARNING: XEDConfig.cmake not found!"

echo ""
echo "=== XED installation complete ==="
echo "Library: ${INSTALL_PREFIX}/lib/libxed.a"
echo "Headers: ${INSTALL_PREFIX}/include/xed/"
echo "CMake:   ${INSTALL_PREFIX}/lib/cmake/XED/XEDConfig.cmake"
