## Fileserver-rs

This project aims to be a simple to use, lightweight web/fileserver with the very few dependencies.

### Usage

You can either run the server using cargo, or by first building it and then running the binary. For simplicity, we will assume the usage of the binary fileserver-rs.

#### Setting port

```
fileserver-rs -p 8080 # sets the port to 8080. If no value is specified, 8080 will be assumed The long version is --port
```

#### Setting interface

```
fileserver-rs -a 0.0.0.0 # sets the bound interface to 0.0.0.0. If no value is specified, 127.0.0.1 will be used. The long version is --addr
```

#### Setting root folder

```
fileserver-rs -r /home/user/test # sets the root folder to /home/user/test. If no value is specified, '.' will be used. The long version is --root
```

#### Help

```
fileserver-rs -h # this will output a short description of all commands. The long version is --help
```
