#!/bin/bash

#
# This script creates a test enviroment for MilkyWay CLI
#

mkdir -p /tmp/mway_test
mkdir -p /tmp/mway_modules

cp modules/certman/target/debug/libcertman.so /tmp/mway_modules/certman.so
cp configs/cli/mwayrc.yml /tmp/mwayrc.yml

