cmake_minimum_required(VERSION 3.16)
project(fcitx5-vigo VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(ECM REQUIRED 1.0.0)
set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/cmake)

find_package(Fcitx5Core REQUIRED)
find_package(Gettext REQUIRED)

# Find vigo library
find_library(VIGO_LIBRARY vigo PATHS ${CMAKE_SOURCE_DIR}/../target/release)
if(NOT VIGO_LIBRARY)
    message(STATUS "vigo library not found in release, trying debug")
    find_library(VIGO_LIBRARY vigo PATHS ${CMAKE_SOURCE_DIR}/../target/debug)
endif()

if(NOT VIGO_LIBRARY)
    message(FATAL_ERROR "vigo library not found. Build it first with: cargo build --release --features ffi")
endif()

include(GNUInstallDirs)

# Addon source
add_library(vigo MODULE
    src/vigo.cpp
)

target_include_directories(vigo PRIVATE
    ${CMAKE_SOURCE_DIR}/../include
    ${Fcitx5Core_INCLUDE_DIRS}
)

target_link_libraries(vigo
    Fcitx5::Core
    ${VIGO_LIBRARY}
)

# Install addon
install(TARGETS vigo DESTINATION "${FCITX_INSTALL_ADDONDIR}")
install(FILES "data/vigo.conf" DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon")
install(FILES "data/vigo-addon.conf" DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod")
