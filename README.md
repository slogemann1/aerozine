# About
This project is a server written in rust which serves resources using the gemini protocol. Gemini
is relatively new protcol similar to some extent to gopher and acts as a secure way to transfer mainly 
textual data. Documentation and more information can be found at the http version of the [site](https://gemini.circumlunar.space/). This server is managed by various configuration files in json format as 
specified under the respective [documentation](https://github.com/slogemann1/gemini-server/blob/master/config-doc.md).

# Features
- Serve any static file
- Serve static files under a different name
- Dynamically generated content
- Serve on multipule domains
- Extensive configuration options
- Error logging

# What is not currently supported
- Client certificates
- More precise status codes (no redirects)
- Caching of dynamic content

# How to build
Currently the only way to use the server is by building it from source and then copying the executable.
This can be done as follows, assuming git and cargo are installed:
```shell
git clone 'https://github.com/slogemann1/gemini-server'
cd gemini-server
cargo build --release
```
After running these commands, the executable should be located at target/release/gemini-server (with a 
different extension depending on OS). This can then either be copied directly into the directory which
has been created (see Setting Up below) or stored somwhere else, optionally within the system path, to be
later executed in the proper directory.

# Setting Up
Once the binary has been created by following the steps above, a directory can be created in which the
program can execute. This can be done automatically by running the program with the 'init' subcommand 
(assuming server is in the path):
```shell
gemini-server init -p server
cd server
```

After this has run, a directory with the following contents should have been created:
```
server_dir
    --> root (This is where static files and configuration files are stored)
        --> index.gmi (This is an example homepage for the server)
        --> config.json (This is a default configuration that includes 'index.gmi')
    --> temp (This is where temporary files for dynamic content are stored, deleted upon starting)
    --> data (This is where your certificate could be stored)
    --> cgi (This is where programs for generation could be stored)
    --> server_settings.json (This is where the server settings are stored)
    --> log.txt (This is where log output is sent if enabled)
```
Generally, the directory can have any files inside it so long as a 'server_settings.json' file and
a root directory (any name is possible for this) are present. For more information on the configuration 
files that are needed and used here, see the [documentation](https://github.com/slogemann1/gemini-server/blob/master/config-doc.md).
\
Once this directory is set up, you will still need a certificate profile to run the server. This is
stored in pfx format and can be created, if no certificate has yet been generated, with the help of openssl:
```shell
cd data
openssl req -x509 -sha256 -nodes -days 365 -newkey rsa:4096 -keyout private.key -out certificate.crt
openssl pkcs12 -export -out profile.pfx -inkey private.key -in certificate.crt
```
With the first command, you will be prompted to enter various values which should be provided, making sure
to enter the domain name under 'Common Name'. For multipule domains, you must create an SAN certificate. The
second command will prompt you to enter a password for the pfx file, which should later be given in in the
'profile_password' field of the server settings.
\
After this is complete, the server can be run either by typing 'gemini-server' or 'gemini-server start'

# TODOs
- Add caching mechanism for dynamically generated content
- Add redirect options
- Add support for client certificates