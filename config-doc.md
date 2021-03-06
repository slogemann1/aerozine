# Documentation for Server Settings and Config Files
All files for configuration are json files. The fields and objects used are
described in the following.

## Server Settings
This file is used to determine the settings that are valid for the entire server.
It should be placed in the working directory of the server program and
must be named "server_settings.json". The format is as follows:
```js
{
    // The default domain that will be requested, defaults to "localhost"
    "domain": "www.example.com",
    // The root directory for the files being served by the server
    // relative to the directory from which the server is started,
    // defaults to "root".
    // Note that this will not be included in the url path
    "root": "root",
    // The pfx file that stores the certificate relative to the directory
    // from which the server is started, defaults to "profile.pfx"
    "tls_profile": "profile.pfx",
    // The password to access the prior profile, defaults to "password"
    "profile_password": "password",
    // A list of configuration files relative to the path of the prior "root" key,
    // defaults to ["config.json"]
    "config_files": [
        "config.json"
    ],
    // The maximum amount of time (in seconds) a program can take to dynamically 
    // generate a webpage before it is stopped, defaults to 10. This can be changed
    // for specific cases as well (see Dynamic Object section)
    "max_dynamic_gen_time": 10,
    // The time (in seconds) in between caching data from dynamically generated content if
    // enabled in the respective dynamic object configuration, defaults to 300
    "cache_time": 300,
    // Determines whether or not files will be loaded into memory before running or loaded while
    // running. If not set in a lower config file, this value will be assumed, defaults to true
    "default_preload": true,
    // This determines whether the program will panic on encountering errors while
    // loading the url tree, defaults to false. Further information can be
    // found under the Never Exit section below
    "never_exit": false,
    // Determines whether or not error messages will be sent with responses to failed requests,
    // defaults to false
    "serve_errors": false,
    // Write error messages and warnings to log file, "log.txt", in the working directory of the
    // server. With this enabled, all request / response errors will be logged, defaults to true
    "log": true
    // The language of the files served by the server, defaults to null
    "default_lang": "en",
    // The text encoding of the files served by the server, defaults to null
    "default_charset": "utf-8",
    // The url path relative to the root the server uses when recieving traffic at the root
    // (e.g. user requests "gemini://www.example.com"), defaults to null
    "homepage": "index.gmi",
    // These determine on with which ip protocols the server will serve documents.
    // "ipv4" defaults to true and "ipv6" to false. If none are set, the server will
    // unconditionally terminate
    "ipv4": true,
    "ipv6": false
}
```

### Never Exit
The never_exit flag in settings should primarily be used when debugging your settings. Its
intended purpose is to display warnings instead of quitting in order to quickly find all problems.
If enabled, this flag causes the following error cases to be shown as warnings:
- Two config files were found in the same directory (The first one is used)
- Directories could not be found / opened while finding files (These are skipped)
- Files can not be read into memory (These are skipped and with every request for their data this is retried)
- Temp / cache directory could not be deleted / created (The files either remain or are not created)
- Dynamic object has both cache enabled and accepts a query (The query is discarded)

## Config Files
These files are used to describe the specific configuration of the files in
their individual directories and sub-directories. Any config file in a lower
position within the file hierarchy will take precedence over the current one
within its directory and sub-directories. These files can have any name, so
long as they are specified within a higher config file or the server settings.
**Note that it is not possible to have multiple configuration files in one
directory as that could result in conflicting settings. Furthermore, these files
will not be part of the url tree regardless of whether they have been whitelisted
or not.** The format is as follows:
```js
{
    // This specifies the domain of the file. If set to null it will default to
    // the domain specified in the server settings, defaults to null
    "domain": null,
    // These specify which files, relative to the parent directory of the config
    // file, should be included or excluded. Each only takes effect if "default_whitelist"
    // is false or true respectively
    "whitelist": [],
    "blacklist": [],
    // Determines whether to recursively include all sub-files from the parent directory of the config
    // file. This behaviour is of course overridden by higher precedence (lower directory) config files
    "default_whitelist": false,
    // Determines whether or not files under the control of this config file will be preloaded before 
    // starting the server or not. If this value is null, the default set in the server settings is used
    "default_preload": null,
    // A list of objects specifing dynamically generated files. This is documented below under
    // the Dynamic Object section
    "dynamic": [],
    // A list of objects specifing url links to other files. This is documented below under
    // the Link Object section
    "link": [],
    // A list of configuration files, with respect to the parent directory of this config
    // file, in lower directories. These lower files override the settings of the current
    // file
    "config_files": [],
}
```

### Dynamic Object
This object specifies various parameters for the execution of a program to provide
dynamically generated content at a specific url path. A unique temporary filename
will be provided as the first argument to the program generating the content in the
format: unique_file_path='/some/path/here'. This path will be absolute. The format
for the dynamic object is as follows:
```js
{
    // The url path for the content to be requested at, relative to the parent directory
    // of the current config file
    "link_path": "echo",
    // The path to the program to be executed. It is absolute, but also reads from
    // the environment path variable (e.g. both "/bin/sh" and "sh" are valid)
    "program_path": "python3",
    // The arguments to be passed to the program. These will be passed before the temporary
    // file path and the query, defaults to []
    "args": ["echo.py"],
    // The working directory for the program to be run in. This path should be absolute or relative
    // to the working directory of the server
    "cmd_working_dir": "/home/pi/Desktop/server/cgi",
    // A list of environment values and their keys which should determine the environment
    // of the program to run
    "cmd_env": [
        {
            "key": "LOG_QUERY", // Example key that the cgi program would handle
            "value": "1"
        },
    ],
    // This determines the query that should be requested at this url. The resulting
    // value will be passed on the command line in the following format: query='value'.
    // Note that all characters will be escaped as needed. *In addition to regular url
    // escape codes, ' is escaped as %27 and " as %22*. This defaults to null
    "query": {
        "display_text": "Enter a message", // The text prompt for retrieving the query
        "private": false // Whether or not the query contains sensitive information
    },
    // This determines whether or not a client certificate is required in order to generate
    // the content. If enabled, the absolute path for a file which contains formatted
    // certificate information will be passed on the command line in the following format:
    // cert_file_path='/path/to/formatted/data/file'. The format of the contained data is
    // described below under Client Certificate Data. This option defaults to false  
    "takes_certificate": false
    // This determines whether or not the program will cache the output of the program
    // instead of re-running it on each request. The time between each cache is determined
    // in the server settings
    "cache": false,
    // A parameter to set the mime-type of the content. *Note that queries are not passed
    // when caching files, so queries will be ignored even if they are enabled for this
    // dynamic object*
    "mime_type": "text/gemini",
    // This defines the amount of time allowed for a program to run before being shut down.
    // If this is null, the default time set in the server settings is used
    "gen_time": 5,
    // The domain for this specific path. If this is null the domain of the config
    // file will be used
    "domain": null
}
```
The idea behind this example is that the cgi python program will read in the command line arguments
for query and file path and then output the query into the file before exiting. Deleting the temporary
file is handled by the server.

### Client Certificate Data
The data in the file passed when generating dynamic content which requires a client certificate follows
a simple key=value format with each key-value pair being seperated by a line break. If the key is not 
present, the corresponding value will be __null. The used keys are shown in the following:
```
fingerprint=D04B98F48E8F8BCC15C6AE5AC050801CD6DCFD428FB5F9E65C4E16E7807340FA
subject=name
email=name@example.com
domain=www.name.com
country=US
province=New York
locality=Exampletown
organization=Example Incorporated
organization_unit=Example Unit
valid_after=Jan 19 14:56:57 2021 GMT
valid_until=Jan  1 12:46:01 2022 GMT
```
Here the fingerprint is the SHA256 digest of the certificate and the times are formatted with a
3-letter month code the day, space-padded, then the hours, minutes and seconds, zero-padded and
seperated by colons, followed by the year.


### Link Object
This object specifies url links to other files. This can be used to either provide multiple
distinct urls for a specific file or to show content under a different name than it is
saved to disk as. Note that this can only link to acutal files and not dynamically generated
content or other links. The format is as follows:
```js
{
    // This determines the domain of the file. If it is null, the domain of the
    // config file is used
    "domain": null,
    // The actual file path relative to the parent directory of the config file
    "file_path": "random_file_name.txt",
    // The url path through which the file from the previous file path should be
    // requested
    "link_path": "well_though_out_name",
    // The mime-type of the content being served through this link. If this is null,
    // the mime type will be inferred from the link path
    "mime_type": "text/plain",
    // Determines whether or not this file will be preloaded before running or if it
    // will be loaded when requested. If this is null, the value of the config file will
    // be used
    "preload": false
}
```