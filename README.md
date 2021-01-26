# About
This project is a server written in rust which serves resources using the gemini protocol. Gemini
is relatively new protcol similar to some extent to gopher and acts as a secure way to transfer mainly textual data. Documentation and more information can be found at the http version of the [site](https://gemini.circumlunar.space/). This server is managed by various configuration files in json format as specified under the respective [documentation](https://github.com/slogemann1/gemini-server/blob/master/config-doc.md).

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
After running these commands, the executable should be located at target/release/gemini-server (with a different extension depending on OS). This can then either be copied directly into the directory which
has been created (see Setting Up below) or stored somwhere else, optionally within the system path, to be
later executed in the proper directory.

# Setting Up
Once the binary has been created by following the steps above, a directory can be created in which the
program can execute. The directory can have any files inside it so long as a 'server_settings.json' file,
a root directory (any name is possible for this) and a 'temp' directory can be present. __Note that any previously existing directory named 'temp' will be deleted upon starting the program.__ Below is an example
of what such a directory could look like:
```
server_dir
    --> root
        --> ... (This is where static files and configuration files are stored)
    --> temp (This is where temporary files for dynamic content are stored)
    --> data
        --> certificate.pfx (This is where your certificate could be stored)
    --> cgi
        --> ... (This is where programs for generation could be stored)
    --> server_settings.json (This is where the server settings are stored)
    --> log.txt (This is where log output is sent if enabled)
```

For more information on the configuration files that are needed and used here, see the
[documentation](https://github.com/slogemann1/gemini-server/blob/master/config-doc.md).