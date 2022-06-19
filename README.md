<h1 align="center">
    <div align="center">
        <img width=140 src="https://github.com/emberry-org/rhizome/blob/main/.github/assets/logo.png"> 
    </div>
    <a href="https://github.com/emberry-org/rhizome/actions/workflows/tests.yml"><img src="https://github.com/emberry-org/rhizome/actions/workflows/tests.yml/badge.svg" height=20 align="right" /></a>
    <a href="https://github.com/emberry-org/rhizome/actions/workflows/audit.yml"><img src="https://github.com/emberry-org/rhizome/actions/workflows/audit.yml/badge.svg" height=20 align="right" /></a>
</h1>


<div align="center">
  <b>Rhizome</b> - The roots of the <a href="https://github.com/emberry-org/emberry">Emberry</a> chat!<br>
</div>

<br>

<div align="center">
    <a href="#development">Development</a>
    ·
    <a href="#license">License</a>
</div>
    
<br>

<h2 align="left">
  <samp>
    <b>Development</b>
  </samp>
</h2>

To run Rhizome for testing use the following command :

```
$ cargo run -- <cert> <key> <tls_port> <udp_port>
```

It is also possible to run Rhizome using Docker
To do so simply run :
```
$ ./docker_run.sh
```
Using the docker setup requires a X509Certificate as ``server.crt`` and the corresponding PKCS8 Private key as ``server.key``
<br>

<br>

<h2 align="left">
  <samp>
    <b>License</b>
  </samp>
</h2>

Copyright (c) Max Coppen, Christopher Freund. All rights reserved.

Licensed under the GNU general public license.

<br>
