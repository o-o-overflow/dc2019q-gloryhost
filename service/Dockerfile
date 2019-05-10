FROM rust:1.34.1

WORKDIR /
EXPOSE 9999
RUN apt-get update && apt-get -y upgrade && apt-get -y install libseccomp-dev
ADD build/gloryhost /
ADD flag /flag
CMD /gloryhost
