#!/usr/bin/env python3

# This utility generates infinite number of random points (0 <= p < 1) and
# writes them to standard output. Each point consists of two space separated
# floating point numbers.
#
# It is the intention that the output is piped to e.g. head.

import random
from signal import signal, SIGPIPE, SIG_DFL
import sys


def main():
    signal(SIGPIPE, SIG_DFL)

    while True:
        x, y = random.random(), random.random()
        sys.stdout.write(f'{x} {y}\n')


if __name__ == '__main__':
    main()
