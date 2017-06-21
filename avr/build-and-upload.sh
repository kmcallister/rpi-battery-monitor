#!/bin/sh

set -xe

avr-gcc -Wall -Werror -std=c99 -mmcu=attiny13a -O3 -o monitor.elf monitor.c
avr-objcopy -O ihex monitor.elf monitor.hex
avr-size monitor.elf
avrdude -c avrisp2 -p attiny13 -U flash:w:monitor.hex
