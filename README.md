# lcs-png-diff-server
Pass in before and after bitmap URL, and then the server will generate LCS diff png and return the diff result URL in response.

## Getting Started

### Install and start server

``` bash
$ cargo install lcs-png-diff-server

$ lcs-png-diff-server
```

### Request

``` bash
curl \
  -d '{
        "before_png": "https://jianliao.github.io/lcs-test-pngs/before.png",
        "after_png": "https://jianliao.github.io/lcs-test-pngs/after.png"
      }' \
  -H 'Content-Type: application/json' \
  -X POST http://localhost:8080/api/diff
```

### Example response payload

``` json
{
  "result_url": "http://localhost:8080/assets/b02d9094-bc6c-4c40-923e-50c66bcf1951.png"
}
```

## Server startup options

``` bash
$ lcs-png-diff-server --help
lcs-png-diff-server 
A server for generating diff bitmaps from png files

USAGE:
    lcs-png-diff-server [OPTIONS]

OPTIONS:
    -a, --addr <ADDR>                set the listen addr [default: 0.0.0.0]
    -h, --help                       Print help information
    -l, --log <LOG_LEVEL>            set the log level [default: info]
    -p, --port <PORT>                set the listen port [default: 8080]
        --static-dir <STATIC_DIR>    set the directory where static files are to be found [default:
                                     ./assets]
```

### Customize the hostname of the response URL

You can customize the hostname of the diff result URL by setting the `HOST_INFO` environment variable.

``` bash
HOST_INFO=https://localhost:443/ lcs-png-diff-server 
```

## Docker

[jianliao/lcs-png-diff-server](https://hub.docker.com/repository/docker/jianliao/lcs-png-diff-server) is a demo-only docker image. CORS had enabled for GET and POST.

### Start server

``` bash
$ docker run --rm -it -p 8080:8080 jianliao/lcs-png-diff-server:0.1.4
```

### Print CLI help

``` bash
$ docker run --rm -it jianliao/lcs-png-diff-server:0.1.4 --help
```

### Change port number

``` bash
$ docker run --rm -it -p 3000:3000 jianliao/lcs-png-diff-server:0.1.4 -p 3000
```

### Change log level

``` bash
$ docker run --rm -it -p 8080:8080 jianliao/lcs-png-diff-server:0.1.4 -l debug
```

### Customize response URL

``` bash
$ docker run --rm -it -e HOST_INFO=https://domainname/ -p 8080:8080 jianliao/lcs-png-diff-server:0.1.4
```

## LICENSE

Apache License Version 2.0
