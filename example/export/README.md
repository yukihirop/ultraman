# Ultraman export example

`ultraman export` is used to export your application to another process management format.  
A location to export can be passed as an argument. This argument may be either required or optional depending on the export format.

The following options control how the application is run:

|short|long|default|description|
|-----|----|-------|-----------|
|<kbd>-m</kbd>|<kbd>--formation</kbd>|`all=1`|Specify the number of each process type to run. The value passed in should be in the format process=num,process=num|
|<kbd>-e</kbd>|<kbd>--env</kbd>|`.env`|Specify an environment file to load|
|<kbd>-f</kbd>|<kbd>--procfile</kbd>|`Procfile`|Specify an alternate Procfile to load|
|<kbd>-p</kbd>|<kbd>--port</kbd>||Specify which port to use as the base for this application. Should be a multiple of 1000|
|<kbd>-a</kbd>|<kbd>--app</kbd>||Use this name rather than the application's root directory name as the name of the application when exporting|
|<kbd>-l</kdb>|<kbd>--log</kdb>||Specify the directory to place process logs in|
|<kbd>-r</kbd>|<kbd>--run</kdb>||Specify the pid file directory, defaults to /var/run/<application>|
|<kbd>-T</kbd>|<kbd>--template</kdb>||Specify an template to use for creating export files|
|<kbd>-u</kbd>|<kbd>--user</kdb>||Specify the user the application should be run as. Defaults to the app name|
|<kbd>-d</kbd>|<kbd>--root</kdb>||Specify an alternate application root. This defaults to the directory containing the Procfile|
|<kbd>-t</kbd>|<kbd>--timeout</kdb>|`5`|Specify the amount of time (in seconds) processes have to shutdown gracefully before receiving a SIGTERM|

## Support Export Format

- upstart
- systemd
- supervisord
- runit
- launchd
- daemon

## Check

|format|check|desc|
|------|-----|----|
|upstart|❌|I couldn't get upstart to work on the Ubuntu image.|
|systemd|❌|I couldn't get upstart to work on the Ubuntu image.|
|supervisord|⭕️||
|runit|⭕️|
|launchd|❌|
|daemon|❌|I couldn't get upstart to work on the Ubuntu image.|


### upstart

```bash
cargo run export upstart ./tmp/upstart -d /home/app -u root

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
cargo run export systemd ./tmp/systemd -d /home/app -u root

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
cargo run export supervisord ./tmp/supervisord -d /home/app -u root

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_supervisord_1 /bin/bash
}

# in docker
root@162d49a056ac:/home/app# /bin/bash ./setup/supervisord.sh
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


### runit

```bash
cargo run export runit ./tmp/runit -d /home/app -u root

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_runit_1 /bin/bash
}

# in docker
root@b84eaadeb69d:/home/app# /bin/bash ./setup/runit.sh
root@b84eaadeb69d:/home/app# runsvdir -P /etc/service
^C

root@b84eaadeb69d:/home/app# cat /var/log/app/exit_0-1/current
success
success
success
```

### launchd

```bash
cargo run export launchd ./tmp/launchd -d /home/app -u root
```

I have not confirmed the operation with the actually generated plist.


### daemon

```bash
cargo run export daemon ./tmp/daemon -d /home/app -u root

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_daemon_1 /bin/bash
}

# in docker
root@35a2d8d4a896:/home/app# sudo service app start # do not work
```


## Example

Here is an example when the `Procfile` and `.env` files have the following contents

[Procfile]
```
exit_0: ./fixtures/exit_0.sh
exit_1: ./fixtures/exit_1.sh
loop: ./fixtures/loop.sh $MESSAGE
```

[.env]
```
MESSAGE="Hello World"
```

### Full option example (short)

```bssh
cargo run export supervisord ./tmp/supervisord \
  -m all=2 \
  -e .env \
  -f Procfile \
  -p 7000 \
  -a example-app \
  -l ./tmp/supervisord/log \
  -r ./tmp/supervisord/run \
  -T ../../src/cmd/export/templates/supervisord \
  -u root \
  -d /home/app \
  -t 10
```

```bash
[ultraman export] cleaning: ./tmp/supervisord/app.conf
[ultraman export] writing: ./tmp/supervisord/app.conf
```

### Full option example (long)

```bssh
cargo run export supervisord ./tmp/supervisord \
  --formation loop=1,exit_0=2 \
  --env .env \
  --procfile Procfile \
  --port 7000 \
  --app example-app \
  --log ./tmp/supervisord/log \
  --run ./tmp/supervisord/run \
  --template ../../src/cmd/export/templates/supervisord \
  --user root \
  --root /home/app \
  --timeout 10
```

```bash
[ultraman export] cleaning: ./tmp/supervisord/app.conf
[ultraman export] writing: ./tmp/supervisord/app.conf
```
