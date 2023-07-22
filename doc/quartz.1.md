---
title: QUARTZ
section: 1
header: User Manual
date: July 22, 2023
---

# NAME
quartz - the CLI way to build and test Rest API

# SYNOPSIS
**quartz** [**-h**] [*COMMAND*] [*OPTIONS*]...

**quartz** is a command-line tool alternative to build, design and test Rest APIs without relying on graphical interfaces.

The tool is organized across multiple configuration files to make it easy to integrate with Git or other VCS.

# ENDPOINT HANDLE

In **quartz**, *endpoint* references are specified by a *handle*. A handle is a unique combination of keywords to find an endpoint. If we were to use an endpoint named "users by-id", this is translated as:

- users
    - by-id

Handles with one or more keywords are organized across the handle tree. For example: if we created a "users create", both "users create" and "users by-id" use the same parent handle.
Deleting a handle also deletes its sub-handles as well.

**quartz** allows as much level of nesting as you wish.

# COMMANDS

## init [*PATH*]
Initialize **quartz**. It creates a *.quartz* directory to keep track of endpoints, contexts, history and other essential informations.

If no path is given, **quartz** initializes in the current directory.

An error will be thrown upon trying to init into an already existing **quartz** project location.

## create <*HANDLE*>...
Create a new handle with an endpoint.

The options are as follows:

**\-\-url** *URL*
: Sets the new endpoint's URL.

**\-\-method** *METHOD*
: Sets the new endpoint's URL.

**\-\-header** *HEADER*
: Set a header entry for the new endpoint in "key: value" format. It can set multiple entries by using the argument multiple times.

**\-\-use**
: Immediatly switches to this handle after creating it.

## status <*OPTION*>
Display the current status of quartz.

The options are as follows:

**\-\-endpoint**
: Print the handle for the endpoint in use.

**\-\-context**
: Print the context in use.

## ls, list
List all available endpoint handles.

The options are as follows:

**\-\-depth** *N*
: Set a limit for how deep the listing goes. For reference, a depth of 1 would show top-level handles.

## use <*HANDLE*>...
Switch to an endpoint by its handle. Using an endpoint allows for operations like *send*, *edit* and other endpoint commands.

## send
Send the request using the current endpoint and outputs the response.

## rm, remove <*HANDLE*>...
Delete the specified handle recursively.

## show [*HANDLE*]...
Display the *endpoint.toml* file of the specified endpoint.

If no handle is provided, quartz will display the currently active endpoint.

The *endpoint.toml* file only includes **URL**, **method**, and **headers**. To get the request body, see *body* command.

## edit
Opens an editor to modify *endpoint.toml* in use. 

The original file is used, so malformed TOML files might break the endpoint alltogether.

The editor it uses is configured through *config* command, which is **vim(1)** by default.

The options are as follows:

**\-\-editor** *EDITOR*
: Defines the editor to be used for that run, overriding the **quartz** settings.

## url
Manage current endpoint's URL.

The options are as follows:

**\-\-get**
: Display the URL.

**\-\-set** *URL*
: Set a new value for URL.

## method
Manage current endpoint's method.

The options are as follows:

**\-\-get**
: Display the method.

**\-\-set** *URL*
: Set a new value for URL.

## headers
Manage current endpoint's headers. All options can be used simultaneously to speed up its usage.

The options are as follows:

**\-\-add** *HEADER*
: Adds a new header entry in "key: value" format. 

**\-\-remove** *KEY*
: Removes a header by its key. 

**\-\-list**
: Display all headers for endpoint in use.

## body
Manage current endpoint's request body.

The options are as follows:

**\-\-stdin**
: Expect a new request body via standard input.

**\-e**, **\-\-edit**
: Opens an editor to modify the endpoint's request body.

**\-p**, **\-\-print**
: Print request body.

## history
Print request history. It uses informations about past requests saved in *.quartz/user/history/*.

The options are as follows:

**\-n**, **\-\-max-count** *N*
: Maximum number of requests to be listed.

**\-\-date *FORMAT*
: Format date time output.

## variable
Manage current context's variables.

The options are as follows:

**\-\-get** *KEY*
: Print a variable value.

**\-\-set** *VARIABLE*
: Sets a variable: key=value.

**\-\-list**
: Print all variables.

**\-e**, **\-\-edit**
: Open an editor to modify the context variables file.

# CONTEXT COMMANDS
Endpoints can benefit from variables. The collection of variables to be used are defined by the active *context*.

By default, **quartz** uses the **default** context containing nothing.

To manage context variables, see *variable* command.

## context create <*NAME*>
Create a new context.

The options are as follows:

**\-c**, **\-\-copy** *CONTEXT*
: Copies variables from another existing context.

## context use <*CONTEXT*>
: Switch to another context.

## context list
: Print all available contexts.

## context remove <*CONTEXT*>
: Delete the specified context.

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

    $ quartz create products create --method POST --url http://localhost:8080/products/ --use

Set a new header to that endpoint:

    $ quartz headers --add 'Content-type: application/json'

Considering that you have a *data.json* file, it is possible to pipe that file so that it uses the contents as a request body:

    $ cat data.json | quartz body --stdin --edit

Send this request:

    $ quartz send

Every sent request is stored in *.quartz/user/history/* and can be printed chronologically. It can be piped through **less(1)** or other pagers to help navigating through:

    $ quartz history | less -r

# AUTHORS

Eduardo Rodrigues <contato@edurodrigues.dev>
