# FindAmbientor.cmake
# -------------------
# Usage:
#   find_package(Ambientor REQUIRED)
#
# Provides:
#   Ambientor::Ambientor   -> imported target
#   AMBIENTOR_INCLUDE_DIRS -> include path
#   AMBIENTOR_LIBRARIES    -> library path
#
# Expects standard Rust build layout:
#   rust/ambientor-ffi/include/ambientor.h
#   rust/target/release/libambientor_ffi.a or .dylib

set(_AMBIENTOR_ROOT "${CMAKE_CURRENT_LIST_DIR}/../..")
set(_AMBIENTOR_INCLUDE "${_AMBIENTOR_ROOT}/rust/ambientor-ffi/include")
set(_AMBIENTOR_LIB_DIR "${_AMBIENTOR_ROOT}/rust/target/release")

find_path(AMBIENTOR_INCLUDE_DIR ambientor.h PATHS "${_AMBIENTOR_INCLUDE}")
find_library(AMBIENTOR_LIBRARY
    NAMES ambientor_ffi libambientor_ffi
    PATHS "${_AMBIENTOR_LIB_DIR}"
)

if (NOT AMBIENTOR_INCLUDE_DIR OR NOT AMBIENTOR_LIBRARY)
    message(FATAL_ERROR "Could not find Ambientor FFI library or headers.")
endif()

add_library(Ambientor::Ambientor STATIC IMPORTED GLOBAL)
set_target_properties(Ambientor::Ambientor PROPERTIES
    IMPORTED_LOCATION "${AMBIENTOR_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${AMBIENTOR_INCLUDE_DIR}"
)

set(AMBIENTOR_INCLUDE_DIRS "${AMBIENTOR_INCLUDE_DIR}" CACHE PATH "Ambientor include dir")
set(AMBIENTOR_LIBRARIES "${AMBIENTOR_LIBRARY}" CACHE FILEPATH "Ambientor library")

message(STATUS "Found Ambientor at ${AMBIENTOR_LIBRARY}")
