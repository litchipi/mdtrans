# Introduction

In this post I will present a little Nix library I made to build web applications
easily using Nix. The skeletton is there but there is still a lot of work
to do to make it really practical, and the purpose of that blog post is to
present what this library does, and is a **call for contributions** as
I really need those to make the library grow. (See the *last section* of this post)

The main idea is to **always** keep it simple for the developper and have a *nice
API*, allow the user to have a *readable* nix code,
and allow devs to rely on *tested and factorized* code when building their web applications.

Throughout the presentation, I will assume that you are using **nix flakes** and
that the library is added to the inputs like so:

``` nix
{
    description = "My web app";

    inputs = {
        nixpkgs.url = github:NixOS/nixpkgs/22.11;
        flake-utils.url = github:numtide/flake-utils;
        nix_web_lib.url = github:litchipi/nix_web_lib;
    };

    outputs = inputs: with inputs; flake-utils.lib.eachDefaultSystem (system: let
        weblib = nix_web_lib.lib.${system};
    in {
        # ...
    });
}
```

In the examples I will use `weblib` directly, which is simply the library imported
for the system you use.

# Building the backend

Building the backend of an application is essentially the same as building any
pieces of software, the process is then no different.  
How to start the backend is up to the developper using the lib, and he'll be
able to pass whatever argument and environment variables he wants, we only
seek to build the binary here.

As of today, **only Rust backend** is implemented (tested with `actix-web` framework)
using [cargo2nix](https://github.com/cargo2nix/cargo2nix)
so you need a `Cargo.nix` file inside the source directory.

It builds like so:

``` nix
backend = weblib.backend.rust.build {
    src = ./backend;
    bin_name = "my_binary_name";      # Default binary name is "backend"

    # Any argument to pass to "pkgs.rustBuilder.makePackageSet" function
    rustBuilderArgs = {
        rustChannel = "stable";
        rustVersion = "1.61.0";
    };
};
```
In itself, this doesn't bring much to the life of a dev as using `cargo2nix` is
just as simple.  
However, the objective of the library is to offer a simple API to build the
all kinds of possible backends flavours, so in case a backend gets a little
complicated to build, *it will have to end up being as simple* as the example given.

There is of course room for improvements as this is a very simple example that
may not be convenient enough for some use cases. But **you can contribute easily**
to make it better (see last section).

# Building the frontend

Where the library really makes it better, is to handle building the frontend
frameworks, which I found was really difficult to do because of the tendency
of Javascript to fetch things from Internet at compile time. (*ugh*)

Fortunately, I found [this answer][buildreactdiscourse] on discourse that
helped me build a basic React frontend, and then translate the process for
VueJS also.

This seam like a bit of a hack solution, but it works well so far and I've
been able to implement the API with a simple function:

``` nix
                    # Replace with "vue" if using VueJS
frontend_compiled_dir = weblib.frontend.react.build {
    src = ./frontend;
};
```

**Huge thanks to enobayram** for finding a solution to make this happend.

The workflow when developping the frontend would be to simply fire up
the backend locally and develop using your usual tools, but for the
final project to compile you'll need to have a `yarn.lock` file.

# Starting a database

Working with databases is not really the funniest thing, and when developping
and application it can be annoying to go against all kinds of database issues
when testing the behaviour.

To help with that, the idea is to provide to the library all sorts of utility
functions to help manage a database for testing locally.

For now only `postgresql` is implemented, and this is how easy it gets to
start the database locally:

``` nix
db_script = let
    dblib = weblib.database.postgresql;

    name = "my_web_app";
    args = {
        dir = "./testdb";
        host = "localhost";
        dbname = "${name}_db";
        user = "${name}_user";
        port = 5465;
    };
in ''
    set -e
    ${dblib.init_db args }            # Init
    ${dblib.pg_ctl args "start" }     # Start
    ${dblib.db_check_connected args } # Check started OK
    ${dblib.ensure_user_exists args } # Create user if doesn't exist
    ${dblib.ensure_db_exists args }   # Created DB if doesn't exist
    ${dblib.stop_on_interrupt args }  # Trap keyboard interrupt to stop db

    tail -f ${args.dir}/logs          # Display the logs
'';
```

# How you can contribute

For now, the usage of this library is very limited, however it is fairly easy
to contribute to implement much more frameworks !

Of course, you still have all the original ways to contribute, open an issue,
fork the repo, propose a PR, etc ...

## Feed me examples

The idea of [the Github repository][githublink] is to gather contributions to
this particular form:

### Create a PR
A contributor creates a PR, adding *an example* to the repository.  
For now the PR only adds:

- A source code stored in `examples/{type}/{langage}/{framework}/`
- A `README.md` file that explains how to build this code **using native tools**

This is the main part of the contribution, and **does not require any Nix knowledge**.

### Bind the new example to the CI

Essentially, we only have to add a line in the `examples/flake.nix` file, calling
the library to build the thing.

This flake has a `build_all` app that will be triggered by the CI to build all the
examples using Nix.

### Implement the building process in the library

It can be done by anyone knowning a bit of Nix, including me,
and the objective is to get a very simple API, and makes the
CI green.  
Once this is done, we merge the PR and voilà !

## An example is worth a thousand words

Not only that it's a practical way to improve the library incrementally,
but it also serves the purpose of showing examples on how to use all the
different flavours that the library provides.

In case one special setup breaks the build system and requires a fix,
*we can also make an example for it*, serving as a regression test, a piece of
documentation for anyone wanting to get to this very special case, etc ...

# Closing thoughts

I like how some projects ease the way to build some programming langages using Nix,
but sometimes it needs to be *scoped down* to a more precise application in order
to have something more efficient. I attempt to create a way to improve the build
of all kinds of web-related software using Nix.

So if you think this is worth *some minutes* of your time, please come to the
[github repository][githublink] and create a PR to add an example for your
favourite framework, or share ideas on how to make it better.

Of course, in case of, you can also contact me directly

[buildreactdiscourse]: https://discourse.nixos.org/t/how-to-use-nix-to-build-a-create-react-app-project/5200/10
[githublink]: https://github.com/litchipi/nix_web_lib
