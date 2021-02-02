# About
Aerozine is a server written in rust which serves resources using the gemini protocol. Gemini
is relatively new protcol similar to some extent to gopher and acts as a secure way to transfer mainly 
textual data. Documentation and more information can be found at the http version of the [site](https://gemini.circumlunar.space/). The 
server is managed by various configuration files in json format as specified under the respective [documentation](https://github.com/slogemann1/aerozine/blob/master/config-doc.md).

# Features
- Serve any static file
- Serve static files under a different name
- Dynamically generated content
- Cache dynamically generated content
- Serve on multiple domains
- Extensive configuration options
- Error logging
- Client certificates

# Upcoming Features
- More precise status codes through process return values
- Option to pass values (query, temp file path, certificate file path) through environment variables

# How to Install
Aerozine can be installed either by downloading the latest release for your platform which is indicated by the name
of the binary which has the format aerozine_operatingsystem_architecture\[.exe\]. If your platform is not included
or you would rather build from source, instructions can be found [here](https://github.com/slogemann1/aerozine/blob/master/build-doc.md).
After you have the executable, you can place it into your system path to use from the command line.

# Setting Up
Once the binary has been created by following the steps above, a directory can be created in which the
program can execute. This can be done automatically by running the program with the 'init' subcommand 
(assuming server is in the path):
```shell
aerozine init -p server
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
files that are needed and used here, see the [documentation](https://github.com/slogemann1/aerozine/blob/master/config-doc.md).
\
Once this directory is set up, you will still need a certificate profile to run the server. This is
stored in pfx format and can be created, if no certificate has yet been generated, with the help of openssl:
```shell
cd data
openssl req -x509 -sha256 -nodes -days 365 -newkey rsa:4096 -keyout private.key -out certificate.crt
openssl pkcs12 -export -out profile.pfx -inkey private.key -in certificate.crt
```
With the first command, you will be prompted to enter various values which should be provided, making sure
to enter the domain name under 'Common Name'. For multiple domains, you must create an SAN certificate. The
second command will prompt you to enter a password for the pfx file, which should later be given in in the
'profile_password' field of the server settings.
\
After this is complete, the server can be run either by typing 'aerozine' or 'aerozine start'

# Notes
Here is some further information about the server that could be of use to take into account.

## On Speed
Since Aerozine is written in rust it should in general be relatively fast when serving resources. All
static files (including link paths) are loaded into memory before the server has started. While this could
be a problem with low ram, the intended use does not expect large files, making this, at least for smaller
servers, a relatively efficient means of serving these files. This is optional, however, with the use of 
the "preload_default" and "preload" options in the configuration. As for dynamic content, however, this is 
greatly dependant on the program generating it. It should be noted that the command objects (in rust) are 
loaded with each (non-cached) request which could have some effect on performance. As for further 
implementation details, it is quite possible that things are not implemented in an efficient manner, 
although I have no more concrete examples.

## On Security
**Note that any concerns of security are entirely the responsibility of the user of this program**. That 
being said, however, aspects of security were taken into consideration when creating this implementation.
\
Dynamically generated content is placed in temporary files whose paths are passed to the programs for 
generation. These filesnames are entirely random, and it is checked, with the use of a hashmap, that
no duplicate names will be created while in use. These file ids are only removed from the hashmap once
their corresponding file has been deleted. Still, this is by no means a guarantee that this works.
\
Another, quite simple, precaution taken is the escaping of single and double quotes in queries. This is done
to attempt to stop users from being able to insert command line arguments to a program. This, again, is
not a guarantee of security.

## Name
The name "Aerozine" is the same as the name of the fuel that powered the Titan II engines, which brought the
gemini capsule into space. The idea being that you can launch your own gemini capsule on the internet using Aerozine.

## Contact
If there are any questions or issues regarding the implementation or useage of this server, I can generally 
be reached at <sllogemann1@gmail.com>. Furthermore, I would be very interested to hear of any projects that
use Aerozine.