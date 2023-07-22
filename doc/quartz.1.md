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

In **quartz**, *endpoints*'s locations are specified by its *handle*. A handle is a unique combination of keywords to find an endpoint. If we were to use an endpoint named "users by-id", this is translated as:

- users
    - by-id

**users** can have multiple other handles inside it, and so can **by-id**. Take a look at a more complete example:

- products
    - by-category
    - by-id
- auth
    - users
        - create
        - by-id
        - token
            - verify

Each of these bullet points are a unique endpoint at an specific handle, which is defined by the sequence of keywords like

```
auth users create
```

Keep in mind that each path could also be an endpoint, or simply an empty handle for organization purposes.

# COMMANDS

## init [*PATH*]
Initialize a new **quartz** project. It creates a *.quartz* directory with essential subdirectories

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

**\-\-endpoint**
: Display the handle for the endpoint in use.

**\-\-context**
: Display the context in use.

## ls, list
Lists all available endpoint handles.

**\-\-depth** *N*
: Set a limit for how deep the listing goes. For reference, a depth of 1 would show top-level handles.

## use <*HANDLE*>...
Switch to an endpoint by its handle. Using an endpoint allows for operations like *send*, *edit* and other endpoint commands.

## send
Sends the request using the current endpoint and outputs the response.

## rm, remove <*HANDLE*>...
Deletes the specified handle recursively.

## show [*HANDLE*]...
Display the *endpoint.toml* file of the specified endpoint.

If no handle is provided, quartz will display the currently active endpoint.

The *endpoint.toml* file only includes **URL**, **method**, and **headers**. To get the request body, see *body* command.

## edit
Opens an editor to modify *endpoint.toml* in use. 

The original file is used, so malformed TOML files might break the endpoint alltogether.

The editor it uses is configured through *config* command, which is **vim(1)** by default.

**\-\-editor** *EDITOR*
: Defines the editor to be used for that run, overriding the **quartz** settings.

## url
Manage current endpoint's URL.

**\-\-get**
: Display the URL.

**\-\-set** *URL*
: Set a new value for URL.

## method
Manage current endpoint's method.

**\-\-get**
: Display the method.

**\-\-set** *URL*
: Set a new value for URL.

## headers [*OPTIONS*]
Manage current endpoint's headers. All flags can be used simultaneously to speed up its usage.

**\-\-add** *HEADER*
: Adds a new header entry in "key: value" format. 

**\-\-remove** *KEY*
: Removes a header by its key. 

**\-\-list**
: Display all headers for endpoint in use.

## body [*OPTIONS*]
Manage current endpoint's request body.

**\-\-stdin**
: Expect a new request body via standard input.

**\-e**, **\-\-edit**
: Opens an editor to modify the endpoint's request body.

**\-p**, **\-\-print**
: Print request body.
