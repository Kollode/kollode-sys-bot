#!/bin/sh

ps -Ao user,uid,comm,pid,pcpu,tty --sort=-pcpu | head -n 6
