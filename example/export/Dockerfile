FROM ubuntu:16.04 as for_upstart
RUN apt-get update && apt-get install -y sudo
ENV APP /home/app
RUN mkdir -p ${APP} && mkdir -p /etc/init /etc/systemd/system
WORKDIR ${APP}
EXPOSE 80
