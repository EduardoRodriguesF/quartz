---
title: QUARTZ
section: 1
header: User Manual
date: March 9, 2024
---

# NAME
quartz - the command-line tool to build and test HTTP requests.

# SYNOPSIS
**quartz** [**-h**] [**\-x** *HANDLE*] [**\-c**, **\-\-apply-environment**] <*COMMAND*> [*OPTIONS*]...

**quartz** is a command-line tool alternative to build, design and test Rest APIs without relying on graphical interfaces.

The tool is organized across multiple configuration files to make it easy to integrate with Git or other VCS.

# OPTIONS

\-x <*HANDLE*>
: Run quartz using a specific handle.

\-c, \--apply-environment

: Apply environment on endpoint as soon as possible.

    For instance, if the current endpoint's URL is "https://{{baseUrl}}/get" and the variable "baseUrl" is correctly set to "httpbin.org", the following command would print the URL with the variable replaced:

        $ quartz --apply-environment show url
        https://httpbin.org/get

    All commands allow for *\--apply-environment*, although some of then would already apply it anyways (e.g. *send*)

# ENDPOINT HANDLE

In **quartz**, *endpoint* references are specified by a *handle*. A handle is like a file path, separated by slash (/). To access "by-id" endpoint in the following structure:

- users
    - by-id

We would use the following handle:

    users/by-id

Handles with one or more keywords are organized across the handle tree. For example: if we created a "users/create", both "users/create" and "users/by-id" use the same parent handle.
Deleting a handle also deletes its sub-handles as well.

**quartz** allows as much level of nesting as you wish.

# **ENDPOINT PATCH**

Some commands allow for endpoint patching, which means they are capable of changing the endpoint's configuration on the fly.

The patched data is not saved to the endpoint file unless stated otherwise.

All commands that allow for patching can be used with the following options:

**\--url** <*URL*>
: Patch request URL.

**\-X**, **\-\-request** <*METHOD*>
: Patch HTTP request method.

**\-q**, **\-\-query** <*PARAM*>...
: Add or patch a parameter to the URL query. It expects one or more key=value pairs.

    This argument can be passed multiple times.

**\-H**, **\-\-header** <*HEADER*>...
: Add or patch a header. It expects one or more header entry in "key: value" format.

    This argument can be passed multiple times.

**\-\-json** [<*DATA*>]
: Interpret the request body as JSON and set the appropriate content-type header. If data is passed, it overwrites the current request body.

**\-d**, **\-\-data** <*DATA*>
: Patch request body.

# URL INHERITANCE

When a handle is created as a child of another, it can inherit the parent's URL by using the "**" notation at the start of its URL field.

    $ quartz create local --url 'http://localhost:8080/'

    $ quartz create local/users --url '**/users'

The endpoint *local/users* will use *http://localhost:8080/users* when sending a request or using *\--apply-environment* with certain commands.

# COMMANDS

**init** [*PATH*]
: Initialize **quartz**. It creates a *.quartz* directory to keep track of endpoints, environments, history and other required data.

    If no path is given, **quartz** initializes in the current directory.

    Trying to initialize quartz in a directory that already has a *.quartz* directory will result in an error.

    If the target path contains a *.git* directory, **quartz** will add some private .quartz files to the .gitignore file. If no .gitignore is available, it creates one.

**create** <*HANDLE*>
: Create a new handle.

    The options are as follows:

    **\-\-use**
    : Immediately switches to this handle after creating it.

**ls**, **list**
: List all available handles.

    The options are as follows:

    **\-\-depth** *N*
    : Set a limit for how deep the listing goes in sub-handles. For reference, a depth of 1 would show top-level handles.

**use** <*HANDLE*>
: Switch to a handle or edit its endpoint. Using a handle with endpoint allows for operations like *send*, *edit* and other endpoint commands.

    All **ENDPOINT PATCH** options are available and will be applied to the current handle permanently.

    Other options are as follows:

    **\-\-empty**
    : Make handle empty. Using it with other editing options will write a new endpoint in place of the old one.

**send**
: Send the request using the current handle's endpoint and outputs the response.

    All **ENDPOINT PATCH** options are available.

    Other options are as follows:

    **\-v, \-\-var** <*KEY=VALUE*>
    : Add or patch environment variable.

    **\--no-follow**
    : Do not follow redirects.

    **\-b**, **\--cookie** <*DATA|FILENAME*>
    : Pass cookie data to request header. If a key=value pair is given, it is used as a cookie of the request URL domain. Otherwise, it is expected to be a file containing cookies.

    **\-c**, **\--cookie-jar** <*FILE*>
    : Which file to write all cookies after a completed request. Existing cookies are not overwritten.

**cp** <*SRC*> <*DEST*>
: Copy a source handle to a destination handle. If the destination handle already exists, it will be overwritten.

    For consistency, *quartz cp* tries to behave as close as possible to the **cp(1)** command in Unix systems.

    The options are as follows:

    *\-r*, **\-\-recursive**
    : Copy child handles recursively.

**mv** <*SRC*> <*DEST*>
: Move a source handle to a destination handle. If the destination handle already exists, it will be overwritten.

    For consistency, *quartz mv* tries to behave as close as possible to the **mv(1)** command in Unix systems.

**rm** <*HANDLE*>...
: Delete handles. If `--recursive` is missing and a handle has child handles, it will not be deleted.

    The options are as follows:

    **\-r**, **\-\-recursive**
    : Delete child handles recursively.

**edit**
: Open an editor to modify endpoint in use. 

    The editor is chosen through the criteria described in **CONFIGURATION** and **ENVIRONMENT** sections.

**history**
: Display request and response history. It uses informations about past requests saved in *.quartz/user/history/*.

    Each request is displayed as HTTP messages exchanges, indicated by lines starting with ">" for request and "<" for response data.

    The options are as follows:

    **\-n**, **\-\-max-count** *N*
    : Maximum number of requests to be listed.

## HEADER
Manage endpoint's headers.

**header get** <*KEY*>
: Display a header value.

**header set** <*HEADER*>...
: Add or patch a header. It expects one or more header entry in "key: value" format.

**header rm** <*KEY*>...
: Remove headers.

**header ls**
: List all headers.

## QUERY
Manage endpoint's query params.

**query get** <*KEY*>
: Display a query value.

**query set** <*PARAM*>...
: Add or patch a header. It expects one or more header entry in "key=value" format.

**query rm** <*KEY*>...
: Remove query parameters.

**query ls**
: List all query parameters.

## BODY
Manage endpoint's request body.

**body show**
: Print request body.

**body stdin**
: Expect a new request body via standard input.

**body edit**
: Open an editor to modify the endpoint's request body.

## SHOW

**show url**
: Display endpoint's request URL.

**show method**
: Display endpoint's request method.

**show headers** [*KEY*]
: Display endpoint's headers.

**show query** [*KEY*]
: Display endpoint's query params.

**show body**
: Display endpoint's request body.

**show handle**
: Display current handle.

**show env**
: Display current environment.

**show cookie**
: Display environment cookies.

**show endpoint**
: Display endpoint file.

**show snippet** [*OPTIONS*] <*COMMAND*>
: Generate code snippet for endpoint.

    All **ENDPOINT PATCH** options are available.

    Other options are as follows:

    **\-v, \-\-var** <*KEY=VALUE*>
    : Add or patch environment variable.

    Code snippet commands are as follows:

    **curl**
    : Generate a curl command. Use **\-\-help** for more options.

    **http**
    : Generate HTTP message.

## LAST
Print informations about the last sent request or its response.

**last handle**
: Print most recent used handle.

**last req[uest]**
: Print most recent request information.

**last res[ponse]**
: Print most recent response information.

**last res head**
: Print most recent response headers.

**last res body**
: Print most recent response body.

## ENV
**quartz** uses environment to manage variables that can be used in endpoints.

By default, the **default** environment is used.

To manage environment variables, see *variable* command.

**env create** <*NAME*>
: Create a new environment.

**env use** <*ENV*>
: Switch to another environment.

**env ls**
: Display all available environments.

**env cp** <*SRC*> <*DEST*> 
: Copy variables from source environment to a new or existing destination environment.

**env rm** <*ENV*>
: Delete an environment.

## VAR
Manage current environment's variables.

**var get** <*KEY*>
: Display a variable value.

**var set** <*KEY=VALUE*>...
: Add or patch a variable.

**var rm** <*KEY*>...
: Remove variables.

**var ls**
: List all variables.

**var edit**
: Open an editor to modify the environment variables file.

# CONFIGURATION
**quartz** default configuration file is *~/.quartz.toml*. Unset options might fallback to environment variables described in the **ENVIRONMENT** section.

Available configuration keys are:

**preferences.editor**
: Command to be run when an editor is needed. If pager is not configured, it defaults to **EDITOR** environment variable.

**preferences.pager**
: Command to be run when a pager is needed. If pager is not configured, it defaults to **PAGER** environment variable.

**ui.colors**
: Whether outputs should be colored (default: true).

Commands are as follows:

**config get** <*KEY*>
: Display a configuration value.

**config set** <*KEY*> <*VALUE*>
: Set a configuration.

**config ls**
: Print *~/.quartz.toml* contents.

**config edit**
: Open an editor to modify the configuration file.

# FILES

*~/.quartz.toml*
: Default **quartz** configuration file.

# ENVIRONMENT

**EDITOR**
: Which text-editor to be used when editing files. If not set, it fallbacks to **vim(1)**.

**PAGER**
: Which pager to be used when needed. If not set, it fallbacks to **less(1)**.

# BUGS

See GitHub Issues: https://github.com/eduardorodriguesf/quartz/issues

# EXAMPLES

To create a new **quartz** project in the current directory:

    $ quartz init

Create a new endpoint *products* with some configuration:

    $ quartz create products --method GET --url http://localhost:8080/products/

Use this new endpoint by specifying its *handle*:

    $ quartz use products

Create a new endpoint in a sub-handle within *products*, switching to it afterwards.

    $ quartz create products/create --method POST --url http://localhost:8080/products/ --use

Set a new header to that endpoint:

    $ quartz header set 'Content-type: application/json'

Considering that you have a *data.json* file, it is possible to pipe that file so that it uses the contents as a request body:

Send this request:

    $ quartz send

Set extra query params and change request method on the fly:

    $ quartz send -X PUT --query somevalue=true

Use **\-x** option to run a **quartz** command from another handle, like sending another request:

    $ quartz -x products send

Every sent request is stored in *.quartz/user/history/* and can be printed chronologically.

    $ quartz history

Or print the most recent request data:

    $ quartz last req

# AUTHORS

Eduardo Rodrigues <contato@edurodrigues.dev>

# SEE ALSO
**cp(1)**, **mv(1)**, **vim(1)**, **less(1)**
