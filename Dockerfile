FROM debian:jessie

RUN apt-get update
RUN apt-get install curl -y
RUN apt-get install file -y
RUN apt-get install sudo -y
RUN apt-get install gcc -y
RUN apt-get install libssl-dev -y

# Clean up APT when done.
RUN apt-get clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# install rust 1.3
RUN curl https://static.rust-lang.org/dist/rust-1.3.0-x86_64-unknown-linux-gnu.tar.gz -o /tmp/rust-1.3.0-x86_64-unknown-linux-gnu.tar.gz
RUN cd /tmp && tar -xvf rust-1.3.0-x86_64-unknown-linux-gnu.tar.gz > /dev/null
RUN sh /tmp/rust-1.3.0-x86_64-unknown-linux-gnu/install.sh

# build the deps first
ADD Cargo* /home/app/open-analytics/
RUN mkdir /home/app/open-analytics/src
# need a dummy file to compile
RUN touch /home/app/open-analytics/src/lib.rs
WORKDIR /home/app/open-analytics
RUN cargo build --release

# now build the source
ADD . /home/app/open-analytics/
RUN cargo build --release

EXPOSE 3000

CMD ["target/release/open_analytics"]
