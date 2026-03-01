cmake_minimum_required(VERSION 3.16)
project(pe2wasm_bridge CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find Remill installation.
set(REMILL_PREFIX "$ENV{REMILL_PREFIX}" CACHE PATH "Remill install prefix")
if(NOT REMILL_PREFIX)
    set(REMILL_PREFIX "/usr/local")
endif()

# Find LLVM (Remill bundles a specific LLVM version).
find_package(LLVM REQUIRED CONFIG
    HINTS "${REMILL_PREFIX}/lib/cmake/llvm")
message(STATUS "Found LLVM ${LLVM_PACKAGE_VERSION}")
message(STATUS "Using LLVMConfig.cmake in: ${LLVM_DIR}")

include_directories(${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})

# Find Remill headers and library.
find_path(REMILL_INCLUDE_DIR
    NAMES remill/Arch/Arch.h
    HINTS "${REMILL_PREFIX}/include"
)
find_library(REMILL_LIB
    NAMES remill
    HINTS "${REMILL_PREFIX}/lib"
)

if(NOT REMILL_INCLUDE_DIR OR NOT REMILL_LIB)
    message(FATAL_ERROR
        "Remill not found at ${REMILL_PREFIX}. "
        "Set REMILL_PREFIX to the install location.")
endif()

message(STATUS "Remill include: ${REMILL_INCLUDE_DIR}")
message(STATUS "Remill lib: ${REMILL_LIB}")

# Build the bridge as a static library.
add_library(remill_bridge STATIC remill_bridge.cpp)

target_include_directories(remill_bridge PRIVATE
    ${REMILL_INCLUDE_DIR}
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# Link Remill + LLVM.
llvm_map_components_to_libnames(LLVM_LIBS
    core support irreader bitreader bitwriter
    analysis transformutils passes target
    x86codegen x86asmparser x86desc x86info
    webassemblycodegen webassemblyasmparser
    webassemblydesc webassemblyinfo
)

target_link_libraries(remill_bridge PRIVATE
    ${REMILL_LIB}
    ${LLVM_LIBS}
)
