cmake_minimum_required(VERSION 3.26)
project(burley)

option(BUILD_TESTING "Enable testing" OFF)
if(BUILD_TESTING)
  enable_testing()
endif()

add_subdirectory("${CMAKE_SOURCE_DIR}/../externals/glm" "${CMAKE_BINARY_DIR}/externals/glm" EXCLUDE_FROM_ALL)

add_library(toolchain INTERFACE)
target_compile_features(toolchain INTERFACE cxx_std_11)
target_compile_definitions(toolchain INTERFACE "$<$<CONFIG:Debug>:_DEBUG>")
if(MSVC)
  set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS ON)
  target_compile_definitions(toolchain INTERFACE "-D_CRT_SECURE_NO_WARNINGS" "-DNOMINMAX")
endif()

add_library(
  libgenpoints SHARED
  faure05.cpp
  genpoints.cpp
  siphash.cpp
  sobol.cpp)
target_link_libraries(
  libgenpoints
  PRIVATE toolchain
  PUBLIC glm)

add_executable(genpoints main.cpp)
target_link_libraries(genpoints PRIVATE toolchain libgenpoints)

install(TARGETS libgenpoints genpoints)
