FROM ubuntu
# open tcp/tls port
EXPOSE 9999
# open udp port
EXPOSE 10000/udp

# Copy rhizome binary
COPY target/release/rhizome /usr/local/bin/rhizome

# create rhizome runner user
RUN useradd -c 'rhizome runner' -m -d /home/rhizome -s /bin/bash rhizome

# insert server cert and key
COPY server.crt /home/rhizome/server.crt
COPY server.key /home/rhizome/server.key
RUN chown -R rhizome:rhizome /home/rhizome

# change workdir and lower privilages to rhizome runner
USER rhizome
ENV HOME /home/rhizome
WORKDIR /home/rhizome

# execute rhizome
CMD rhizome
