cmake_minimum_required(VERSION 3.16)
project(bench_vs_unikey CXX)

set(CMAKE_CXX_STANDARD 17)

# Unikey engine sources
set(UKENGINE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../references/ibus-unikey/ukengine")
file(GLOB UKENGINE_SOURCES "${UKENGINE_DIR}/*.cpp")
add_library(ukengine STATIC ${UKENGINE_SOURCES})
target_include_directories(ukengine PUBLIC ${UKENGINE_DIR})
target_compile_options(ukengine PRIVATE -funsigned-char -Wno-invalid-source-encoding -O2)

# Vigo library (pre-built)
set(VIGO_LIB "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/libvigo.so")
set(VIGO_INCLUDE "${CMAKE_CURRENT_SOURCE_DIR}/../../include")

# Benchmark binary
add_executable(bench_vs_unikey bench.cpp)
target_include_directories(bench_vs_unikey PRIVATE ${UKENGINE_DIR} ${VIGO_INCLUDE})
target_link_libraries(bench_vs_unikey ukengine ${VIGO_LIB})
target_compile_options(bench_vs_unikey PRIVATE -O2)
