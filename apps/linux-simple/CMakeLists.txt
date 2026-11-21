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
