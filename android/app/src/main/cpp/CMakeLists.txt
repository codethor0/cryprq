cmake_minimum_required(VERSION 3.22.1)

project(cryprq_android LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(cryprq_android SHARED CrypRqNative.cpp)

find_library(log-lib log)
find_library(dl-lib dl)

target_link_libraries(cryprq_android PRIVATE ${log-lib} ${dl-lib})

