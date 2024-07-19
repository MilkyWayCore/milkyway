#!/bin/bash

mkdir /tmp/mway_test
mkdir /tmp/mway_modules

cp modules/certman/target/debug/libcertman.so /tmp/mway_modules/certman.so
cp configs/cli/mwayrc.yml /tmp/mwayrc.yml

