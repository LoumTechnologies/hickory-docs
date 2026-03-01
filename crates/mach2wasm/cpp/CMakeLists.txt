cmake_minimum_required(VERSION 3.16)
project(mach2wasm_bridge C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find Remill installation.
set(REMILL_PREFIX "$ENV{REMILL_PREFIX}" CACHE PATH "Remill install prefix")
if(NOT REMILL_PREFIX)
    set(REMILL_PREFIX "/usr/local")
endif()

# Find LLVM (Remill depends on a specific LLVM version).
find_package(LLVM REQUIRED CONFIG
    HINTS "${REMILL_PREFIX}/lib/cmake/llvm")
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

# Find Remill headers.
find_path(REMILL_INCLUDE_DIR
    NAMES remill/Arch/Arch.h
    HINTS "${REMILL_PREFIX}/include"
)

if(NOT REMILL_INCLUDE_DIR)
    message(FATAL_ERROR
        "Remill headers not found at ${REMILL_PREFIX}/include. "
        "Set REMILL_PREFIX to the install location.")
endif()

message(STATUS "Remill include: ${REMILL_INCLUDE_DIR}")

# Collect all Remill static libraries.
file(GLOB REMILL_LIBS "${REMILL_PREFIX}/lib/libremill_*.a")
# Also grab sleigh/decomp support libs that Remill depends on.
file(GLOB SLEIGH_LIBS "${REMILL_PREFIX}/lib/libsla*.a"
                      "${REMILL_PREFIX}/lib/libdecomp.a")

message(STATUS "Remill libs: ${REMILL_LIBS}")
message(STATUS "Sleigh libs: ${SLEIGH_LIBS}")

# Build the bridge as a static library.
add_library(remill_bridge STATIC remill_bridge.cpp)

target_include_directories(remill_bridge PRIVATE
    ${REMILL_INCLUDE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# glog 0.7+ requires GLOG_USE_GLOG_EXPORT to pull in export.h before logging.h.
target_compile_definitions(remill_bridge PRIVATE GLOG_USE_GLOG_EXPORT)

# Link Remill + LLVM.
llvm_map_components_to_libnames(LLVM_LIBS
    core support irreader bitreader bitwriter
    analysis transformutils passes target
    x86codegen x86asmparser x86desc x86info
    webassemblycodegen webassemblyasmparser
    webassemblydesc webassemblyinfo
)

target_link_libraries(remill_bridge PRIVATE
    ${REMILL_LIBS}
    ${SLEIGH_LIBS}
    ${LLVM_LIBS}
)

# Install the bridge library.
install(TARGETS remill_bridge
    ARCHIVE DESTINATION lib)
