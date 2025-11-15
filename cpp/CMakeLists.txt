cmake_minimum_required(VERSION 3.16)
project(ambientor_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ------------------------------------------------------------------------------
# Directories
# ------------------------------------------------------------------------------
set(AMBIENTOR_ROOT "${CMAKE_SOURCE_DIR}/..")
set(RUST_TARGET "${AMBIENTOR_ROOT}/rust/target/release")

include_directories("${AMBIENTOR_ROOT}/rust/ambientor-ffi/include")

# ------------------------------------------------------------------------------
# RtAudio discovery (Homebrew)
# ------------------------------------------------------------------------------
find_path(RTAUDIO_INCLUDE_DIR
  NAMES RtAudio.h rtaudio/RtAudio.h
  PATHS /usr/local/include /usr/local/opt/rtaudio/include
  PATH_SUFFIXES rtaudio
)
find_library(RTAUDIO_LIBRARY
  NAMES rtaudio
  PATHS /usr/local/lib /usr/local/opt/rtaudio/lib
)
if(NOT RTAUDIO_INCLUDE_DIR OR NOT RTAUDIO_LIBRARY)
  message(FATAL_ERROR "RtAudio not found. Try: brew install rtaudio")
endif()

message(STATUS "RtAudio include: ${RTAUDIO_INCLUDE_DIR}")
message(STATUS "RtAudio lib    : ${RTAUDIO_LIBRARY}")

# ------------------------------------------------------------------------------
# Real-Time Ambientor Host
# ------------------------------------------------------------------------------
add_executable(ambientor_host apps/ambientor_host.cpp)
target_include_directories(ambientor_host PRIVATE "${RTAUDIO_INCLUDE_DIR}")
target_link_libraries(ambientor_host PRIVATE "${RTAUDIO_LIBRARY}" "${RUST_TARGET}/libambientor_ffi.a")

if(APPLE)
  target_link_libraries(ambientor_host PRIVATE
    "-framework CoreAudio"
    "-framework AudioToolbox"
    "-framework CoreFoundation"
  )
endif()

# ------------------------------------------------------------------------------
# Plugin Example (optional)
# ------------------------------------------------------------------------------
if(EXISTS "${CMAKE_SOURCE_DIR}/plugins/ambientor_example.cpp")
  add_executable(ambientor_example plugins/ambientor_example.cpp)
  target_link_libraries(ambientor_example PRIVATE "${RUST_TARGET}/libambientor_ffi.a")
endif()

message(STATUS "")
message(STATUS "Ambientor C++ targets configured:")
message(STATUS "  - ambientor_host      (real-time RtAudio host)")
if(TARGET ambientor_example)
  message(STATUS "  - ambientor_example   (plugin-style embedding)")
endif()
message(STATUS "")
