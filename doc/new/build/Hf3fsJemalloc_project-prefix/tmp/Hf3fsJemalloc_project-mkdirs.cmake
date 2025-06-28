# Distributed under the OSI-approved BSD 3-Clause License.  See accompanying
# file Copyright.txt or https://cmake.org/licensing for details.

cmake_minimum_required(VERSION 3.5)

file(MAKE_DIRECTORY
  "/home/renzz/Valor-Go/doc/new/third_party/jemalloc"
  "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/src/Hf3fsJemalloc_project-build"
  "/home/renzz/Valor-Go/doc/new/build/third_party/jemalloc"
  "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/tmp"
  "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/src/Hf3fsJemalloc_project-stamp"
  "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/src"
  "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/src/Hf3fsJemalloc_project-stamp"
)

set(configSubDirs RelWithDebInfo;Debug;Release;MinSizeRel)
foreach(subDir IN LISTS configSubDirs)
    file(MAKE_DIRECTORY "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/src/Hf3fsJemalloc_project-stamp/${subDir}")
endforeach()
if(cfgdir)
  file(MAKE_DIRECTORY "/home/renzz/Valor-Go/doc/new/build/Hf3fsJemalloc_project-prefix/src/Hf3fsJemalloc_project-stamp${cfgdir}") # cfgdir has leading slash
endif()
