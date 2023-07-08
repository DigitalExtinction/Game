#!/usr/bin/env bash

git grep -n '//\s*TODO' -- '*.rs'
exit_status=$?

if [ $exit_status -eq 0 ]; then
    exit 1
elif [ $exit_status -eq 1 ]; then
    exit 0
else
    exit $exit_status
fi
