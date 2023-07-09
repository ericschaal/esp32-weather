#!/usr/bin/env bash
espflash /dev/ttyUSB0 target/xtensa-esp32-espidf/debug/weather-esp32
espflash serial-monitor /dev/ttyUSB0