#!/bin/sh
# Builds the stub server and the C++ round-trip tool against the vendored
# flatbuffers 24.3.25 headers in third_party/ and the checked-in generated
# header cdda_ipc_generated.h (flatc 24.3.25, --cpp --gen-object-api).
set -e
cd "$(dirname "$0")"
CXX="${CXX:-g++}"
CXXFLAGS="-std=c++17 -O1 -Wall -Wextra -I. -Ithird_party"
$CXX $CXXFLAGS stub_server.cpp -o stub-server
$CXX $CXXFLAGS cpp_roundtrip.cpp -o cpp-roundtrip
echo "built stub-server and cpp-roundtrip"
