# rustman export example

## Support Export Format

- upstart

## Check

|format|check|desc|
|------|-----|----|
|upstart|❌|I couldn't get upstart to work on the Ubuntu image.|


### upstart

```bash
cargo run export upstart ./tmp/upstart

{
  docker-compose build
  docker-compose up -d
  docker exec -it export_export_upstart_1 /bin/bash
}

root@35a2d8d4a896:/home/app# sudo service app start # do not work
```

