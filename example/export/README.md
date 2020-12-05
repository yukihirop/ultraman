# rustman export example

## Support Export Format

- upstart
- systemd
- supervisord

## Check

|format|check|desc|
|------|-----|----|
|upstart|❌|I couldn't get upstart to work on the Ubuntu image.|
|systemd|❌|I couldn't get upstart to work on the Ubuntu image.|
|supervisord|⭕️||


### upstart

```bash
cargo run export upstart ./tmp/upstart -d /home/app

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_upstart_1 /bin/bash
}

# in docker
root@35a2d8d4a896:/home/app# sudo service app start # do not work
```

### systemd

```bash
cargo run export systemd ./tmp/systemd -d /home/app

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_systemd_1 /bin/bash
}

# in docker
root@d09938652e40:/home/app# sudo systemctl daemon-reload
root@d09938652e40:/home/app# sudo systemctl restart exit_0-exit_0.0 exit_1-exit_1.0 loop-loop.0 # do not work (Failed to connect to bus: No such file or directory)
```

### supervisord

```bash
cargo run export supervisord ./tmp/supervisord -d /home/app

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_supervisord_1 /bin/bash
}

# in docker
root@162d49a056ac:/home/app# export MESSAGE="Hello\ World"

root@162d49a056ac:/home/app# supervisord -c /etc/supervisor/conf.d/app.conf
2020-12-06 05:57:32,120 CRIT Supervisor running as root (no user in config file)
2020-12-06 05:57:32,125 INFO supervisord started with pid 25
2020-12-06 05:57:33,129 INFO spawned: 'app-exit_1-1' with pid 28
2020-12-06 05:57:33,136 INFO spawned: 'app-loop-1' with pid 29
2020-12-06 05:57:33,145 INFO spawned: 'app-exit_0-1' with pid 31
2020-12-06 05:57:34,150 INFO success: app-exit_1-1 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
2020-12-06 05:57:34,151 INFO success: app-loop-1 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
2020-12-06 05:57:34,151 INFO success: app-exit_0-1 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
2020-12-06 05:57:35,145 INFO exited: app-exit_1-1 (exit status 1; not expected)
2020-12-06 05:57:35,156 INFO spawned: 'app-exit_1-1' with pid 36
2020-12-06 05:57:36,156 INFO success: app-exit_1-1 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
2020-12-06 05:57:36,158 INFO exited: app-exit_0-1 (exit status 0; expected)
2020-12-06 05:57:37,160 INFO spawned: 'app-exit_0-1' with pid 40
2020-12-06 05:57:37,166 INFO exited: app-exit_1-1 (exit status 1; not expected)
2020-12-06 05:57:38,161 INFO spawned: 'app-exit_1-1' with pid 43
2020-12-06 05:57:38,163 INFO success: app-exit_0-1 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
2020-12-06 05:57:39,161 INFO success: app-exit_1-1 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
^C2020-12-06 05:57:39,637 WARN received SIGINT indicating exit request
2020-12-06 05:57:39,639 INFO waiting for app-exit_1-1, app-loop-1, app-exit_0-1 to die
2020-12-06 05:57:40,163 INFO stopped: app-loop-1 (terminated by SIGTERM)
2020-12-06 05:57:40,164 INFO stopped: app-exit_0-1 (terminated by SIGTERM)
2020-12-06 05:57:40,165 INFO stopped: app-exit_1-1 (terminated by SIGTERM)

root@162d49a056ac:/home/app# cat /var/log/app/loop-1.log
Hello World
Hello World
Hello World
Hello World
Hello World
Hello World

root@162d49a056ac:/home/app#
```

See [this](http://supervisord.org/subprocess.html#nondaemonizing-of-subprocesses) to know about supervisord
