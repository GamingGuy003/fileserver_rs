## Fileserver-rs

This project aims to be a simple to use, lightweight web/fileserver with the very few dependencies.

### Usage

You can either run the server using cargo, or by first building it and then running the binary. For simplicity, we will assume the usage of the binary fileserver-rs.

#### Setting port

```bash
# Sets the port to 8080. If no value is specified, 8080 will be assumed The long version is --port
fileserver-rs -p 8080
```

#### Setting interface

```bash
# Sets the bound interface to 0.0.0.0. If no value is specified, 127.0.0.1 will be used. The long version is --addr
fileserver-rs -a 0.0.0.0
```

#### Setting root folder

```bash
# Sets the root folder to /home/user/test. If no value is specified, '.' will be used. The long version is --root
fileserver-rs -r /home/user/test
```

#### Help

```bash
# This will output a short description of all commands. The long version is --help
fileserver-rs -h
```
