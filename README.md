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

## Documentation

To get started with quartz, head to the [install](#installation) and access the manual page at `man quartz`!

If you have any trouble, you can also read it in markdown [here](doc/quartz.1.md).

## License

This project is under [Apache License 2.0](/LICENSE).
