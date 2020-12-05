FROM ubuntu:16.04 as for_upstart
RUN apt-get update && apt-get install -y sudo vim supervisor && rm -rf /var/lib/apt/lists/*
ENV APP /home/app


RUN mkdir -p ${APP} && mkdir -p /etc/init /etc/systemd/system /etc/supervisor/conf.d

# for supervisor
# If this is not done, the following error will occur
#
# root@25e20da59668:/# supervisorctl version
# unix:///var/run/supervisor.sock no such file
RUN touch /etc/supervisord.conf

WORKDIR ${APP}
EXPOSE 80
