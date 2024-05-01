# quartz

The command-line tool to build and test HTTP requests.

## About

quartz allows developers to create an API organization project that can be use to share, document and send requests through the terminal.

What quartz is:

- A tool to test and document HTTP requests aimed for developers who love to stay in the terminal.
- A command-line alternative to API clients such as Postman and Insomnia.

What Quartz is **not**:

- A cURL replacement.

## Installation

<details>
    <summary>Bash (Linux)</summary>

    bash -c "$(curl -fsSL https://raw.githubusercontent.com/EduardoRodriguesF/quartz/master/install.sh)"
</details>

<details>
    <summary>Homebrew (MacOS)</summary>

    brew tap eduardorodriguesf/quartz
    brew install quartz
</details>

<details>
    <summary>Cargo</summary>

Warning: this method is not recommended because it lacks the **man** page. Prefer the other installation options above.

    cargo install quartz-cli
</details>

## Usage

To create a new project, run:

```sh
$ quartz init .
```

Now start creating your requests with the `create` command:

```sh
$ quartz create users/find --url https://api.example.com/users/{{id}}

$ quartz create users/update -X PATCH --url https://api.example.com/users/{{id}} --json '{"name": "John Doe"}'

$ quartz create users/create -X POST --url https://api.example.com/users/{{id}} --json '{"email": "foo@bar.com", "name": "John Doe"}'
```

These commands create four *handles*. A handle is like a path to an endpoint. Similarly to file paths, they are segmented
by slash (/). We can see all handles with the `ls` command:

```sh
$ quartz ls
  ---   users
  GET   users/:id
  PATCH users/update
  POST  users/create
```

Notice that `users` does not have any method. That's because it is an empty handle, while all others are definitive endpoints that we can
send requests to.

To send a request, make sure you are using it by running the `use` command:

```sh
$ quartz use users/find
```

Now you can send the request with the `send` command. Since we also defined a variable `id` in the URL, we need to give this variable a value.

```sh
$ quartz send --var id=123
```

This outputs the response body, but we can also see more details with the `last` command, which saves our latest request and response
for us to fetch the data locally.

```sh
$ quartz last res head
```

You can even output a cURL command to replicate the request:

```sh
$ quartz show snippet --var id=123 curl
curl -L 'https://api.example.com/users/123' -X GET
```

Now that you know the basics of quartz, you can start creating more requests and organizing them in your project. For more information and advanced usage
of quartz, check the [Documentation](#documentation).

## Documentation

To get started with quartz, head to the [install](#installation) and access the manual page at `man quartz`!

If you have any trouble, you can also read it in markdown [here](doc/quartz.1.md).

## License

This project is under [Apache License 2.0](/LICENSE).
