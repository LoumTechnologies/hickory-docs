cmake_minimum_required(VERSION 3.16)
project(mach2wasm_bridge C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Use find_package(remill) which handles LLVM, XED, glog, gflags, sleigh.
# REMILL_PREFIX and REMILL_DEPS_PREFIX are added to CMAKE_PREFIX_PATH so
# cmake can locate remillConfig.cmake and all transitive dependencies.
set(REMILL_PREFIX "" CACHE PATH "Remill install prefix")
set(REMILL_DEPS_PREFIX "" CACHE PATH "Remill dependencies install prefix")

if(REMILL_DEPS_PREFIX)
    list(APPEND CMAKE_PREFIX_PATH "${REMILL_DEPS_PREFIX}")
endif()
if(REMILL_PREFIX)
    list(APPEND CMAKE_PREFIX_PATH "${REMILL_PREFIX}")
endif()

find_package(remill REQUIRED CONFIG)
message(STATUS "Found Remill at: ${remill_DIR}")

# LLVM include dirs are needed for the bridge C++ code.
include_directories(${LLVM_INCLUDE_DIRS})

# Build the bridge as a static library.
add_library(remill_bridge STATIC remill_bridge.cpp)

target_include_directories(remill_bridge PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
    ${REMILL_PREFIX}/include
)

# Link against the remill imported target (brings in LLVM, XED, etc.).
target_link_libraries(remill_bridge PRIVATE remill)

install(TARGETS remill_bridge ARCHIVE DESTINATION lib)
