# rustman export example

## Support Export Format

- upstart
- systemd

## Check

|format|check|desc|
|------|-----|----|
|upstart|❌|I couldn't get upstart to work on the Ubuntu image.|
|systemd|❌|I couldn't get upstart to work on the Ubuntu image.|


### upstart

```bash
cargo run export upstart ./tmp/upstart -d /home/app

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_upstart_1 /bin/bash
}

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

root@d09938652e40:/home/app# sudo systemctl daemon-reload
root@d09938652e40:/home/app# sudo systemctl restart exit_0-exit_0.0 exit_1-exit_1.0 loop-loop.0 # do not work (Failed to connect to bus: No such file or directory)
```
