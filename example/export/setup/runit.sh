#!/bin/sh
rm -rf /etc/service/{app-exit_0-1,app-exit_1-1,app-loop-1}/supervise/
mkdir -p /var/log/app/{exit_0-1,exit_1-1,loop-1}
sudo chmod 0755 /etc/service/{app-exit_0-1,app-exit_1-1,app-loop-1}/run
sudo chmod 0755 /etc/service/{app-exit_0-1,app-exit_1-1,app-loop-1}/log/run
