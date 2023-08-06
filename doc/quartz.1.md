---
title: QUARTZ
section: 1
header: User Manual
date: August 5, 2023
---

# NAME
quartz - the CLI way to build and test Rest API

# SYNOPSIS
**quartz** [**-h**] [**\-x** *HANDLE*] <*COMMAND*> [*OPTIONS*]...

**quartz** is a command-line tool alternative to build, design and test Rest APIs without relying on graphical interfaces.

The tool is organized across multiple configuration files to make it easy to integrate with Git or other VCS.

# OPTIONS

## \-x <*HANDLE*>

Run quartz using a specific handle.

## \-c, \-\-apply-context

Apply context on endpoint as soon as possible.

For example, if the active endpoint's URL is "https://{{baseUrl}}/get" and the variable "baseUrl" is correctly set to "httpbin.org".

    $ quartz --apply-context url --get
    https://httpbin.org/get

It is redundant to use this command alongside *send* command, because it already applies the context before sending the request.

# ENDPOINT HANDLE

In **quartz**, *endpoint* references are specified by a *handle*. A handle is like a file path, separated by slash (/). So, to access "by-id" endpoint in the following structure:

- users
    - by-id

We would use the following handle:

    users/by-id

Handles with one or more keywords are organized across the handle tree. For example: if we created a "users/create", both "users/create" and "users/by-id" use the same parent handle.
Deleting a handle also deletes its sub-handles as well.

**quartz** allows as much level of nesting as you wish.

# COMMANDS

## init [*PATH*]
Initialize **quartz**. It creates a *.quartz* directory to keep track of endpoints, contexts, history and other essential informations.

If no path is given, **quartz** initializes in the current directory.

An error will be thrown upon trying to init into an already existing **quartz** project location.

## create <*HANDLE*>
Create a new handle.

The options are as follows:

**\-\-url** *URL*
: Set handle's endpoint URL.

**\-\-method** *METHOD*
: Set handle's endpoint method.

**\-\-header** *HEADER*
: Set a header entry in "key: value" format. It can set multiple entries by using the argument multiple times.

**\-\-use**
: Immediatly switches to this handle after creating it.

## status <*OPTION*>
Print the current status of **quartz**.

The options are as follows:

**\-\-endpoint**
: Print the handle for the endpoint in use.

**\-\-context**
: Print the context in use.

## ls, list
List all available handles.

The options are as follows:

**\-\-depth** *N*
: Set a limit for how deep the listing goes in sub-handles. For reference, a depth of 1 would show top-level handles.

## use <*HANDLE*>
Switch to a handle. Using a handle with endpoint allows for operations like *send*, *edit* and other endpoint commands.

## send
Send the request using the current handle's endpoint and outputs the response.

## rm, remove <*HANDLE*>
Delete the specified handle recursively.

## show [*HANDLE*]
Print endpoint informations at a handle.

If no handle is provided, quartz will display the currently active endpoint.

The *endpoint.toml* file only includes **URL**, **method**, and **headers**. To get the request body, see *body* command.

## edit
Open an editor to modify endpoint in use. 

The editor it uses is configured through *config* command, which is **vim(1)** by default.

The options are as follows:

**\-\-editor** *EDITOR*
: Defines the editor to be used for that run, overriding the **quartz** settings.

## url
Manage current handle's endpoint URL.

The options are as follows:

**\-\-get** [**\-\-full**]
: Display the URL. The **\-\-full** flag is used to include *query* params.

**\-\-set** *URL*
: Set a new value for URL.

## method
Manage current handle's endpoint method.

The options are as follows:

**\-\-get**
: Print method.

**\-\-set** *URL*
: Set a value for URL.

## query
Manage current handle's endpoint query params.

Without options, this command prints the entire query param string.

The options are as follows:

**\-\-get** *KEY*
: Print query param value.

**\-\-set** *QUERY*
: Set query param value.

**\-\-remove** *KEY*
: Remove query param.

**\-\-list**
: List all query param.

## header
Manage current handle's endpoint headers. All options can be used simultaneously to speed up its usage.

The options are as follows:

**\-\-get** *KEY*
: Print a header value.

**\-\-set** *HEADER*
: Set new header entry or overwrites an existing one in "key: value" format. 

**\-\-remove** *KEY*
: Removes a header. 

**\-\-list**
: Print headers.

## body
Manage current endpoint's request body.

The options are as follows:

**\-\-stdin**
: Expect a new request body via standard input.

**\-e**, **\-\-edit**
: Open an editor to modify the endpoint's request body.

**\-p**, **\-\-print**
: Print request body.

## history
Print request history. It uses informations about past requests saved in *.quartz/user/history/*.

The options are as follows:

**\-n**, **\-\-max-count** *N*
: Maximum number of requests to be listed.

**\-\-date *FORMAT*
: Format date time output.

## var, variable
Manage current context's variables.

The options are as follows:

**\-\-get** *KEY*
: Print a variable value.

**\-\-set** *VARIABLE*
: Set a variable: key=value.

**\-\-list**
: Print all variables.

**\-e**, **\-\-edit**
: Open an editor to modify the context variables file.

# LAST COMMAND
Print informations about the last sent request or its response.

## last request
Print most recent request information.

**\-\-url**
: Print last used URL.

**\-\-query**
: Print last used query params.

**\-\-method**
: Print last used method.

**\-\-headers**
: Print last used headers.

**\-\-body**
: Print last used body.

**\-\-context**
: Print last used context.

## last response
Print most recent response information.

**\-\-status**
: Print last response status.

**\-\-headers**
: Print last response headers.

**\-\-body**
: Print last response body.

**\-\-size**
: Print last response content size.

# CONTEXT COMMAND
Endpoints can benefit from variables. The collection of variables to be used are defined by the active *context*.

By default, **quartz** uses the **default** context containing nothing.

To manage context variables, see *variable* command.

## context create <*NAME*>
Create a new context.

The options are as follows:

**\-c**, **\-\-copy** *CONTEXT*
: Copy variables from another context.

## context use <*CONTEXT*>
: Switch to another context.

## context list
: Print all available contexts.

## context remove <*CONTEXT*>
: Delete a context.

# CONFIG COMMAND
**quartz** default configuration file is *~/.quartz.toml*.

Available configuration keys are:

* preferences.editor -- Command to be run when an editor is needed (default: vim(1)).
* ui.colors -- Whether outputs should be colored (default: true).

## config \-\-get <*KEY*>
Print configuration value.

## config \-\-set <*KEY*> <*VALUE*>
Set a configuration.

## config \-\-list
Print configuration file.

## config \-\-edit
Open an editor to modify configuration file.

# FILES

*~/.quartz.toml* -- Default **quartz** configuration file.

# EXAMPLES

To create a new **quartz** project in the current directory:

    $ quartz init

Create a new endpoint *products* with some configuration:

    $ quartz create products --method GET --url http://localhost:8080/products/

Use this new endpoint by specifying its *handle*:

    $ quartz use products

Create a new endpoint in a sub-handle within *products*, immediatly switching to it.

    $ quartz create products/create --method POST --url http://localhost:8080/products/ --use

Set a new header to that endpoint:

    $ quartz header --set 'Content-type: application/json'

Considering that you have a *data.json* file, it is possible to pipe that file so that it uses the contents as a request body:

    $ cat data.json | quartz body --stdin --edit

Send this request:

    $ quartz send

Use **\-x** option to run a **quartz** command from another handle, like sending another request:

    $ quartz -x products send

Every sent request is stored in *.quartz/user/history/* and can be printed chronologically. It can be piped through **less(1)** or other pagers to help navigating through:

    $ quartz history | less -r

# AUTHORS

Eduardo Rodrigues <contato@edurodrigues.dev>
