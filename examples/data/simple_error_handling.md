Error handling can be tedious and extermely annoying...
On most programming langages, but using **Rust** it's actually very simple !

Of course coding methods are a matter of preference and I am nobody to say this
is better than something else, but here's my 2 cents on this subject, and how I
prefer to do it.

# How I organise error handling in my projects

What I usually do with my projects is that I setup a simple error handling type
`pub enum Errcode` in a file `src/errors.rs`, and I define every possible kind
of errors in it.

In the whole project, I write any function that can result in an error with
a return type of `Result<_, Errcode>`, allowing me to use the `?` operator
everywhere, returning the error till it reaches the very `main` function of my binary.

## Error conversion

When I perform a serialization using `serde`, or some function returning a `std::io::Error`,
How can I transform the error to my "standardized" one ?

Here comes the `From` trait:

```rust
pub enum Errcode {
    SomeErrorType,

    SerdeSerialization(serde::ser::Error),
    SerdeDeserialization(serde::de::Error),

    IoError(std::io::Error),
}

impl From<serde::ser::Error> for Errcode {
    fn from(e: serde::ser::Error) -> Self {
        Errcode::SerdeSerialization(e)
    }
}

impl From<serde::de::Error> for Errcode {
    fn from(e: serde::de::Error) -> Self {
        Errcode::SerdeDeserialization(e)
    }
}

impl From<std::io::Error> for Errcode {
    fn from(e: std::io::Error) -> Self {
        Errcode::IoError(e)
    }
}
```

With this setup, I can use the code abusing the `?` operator:

```rust
use crate::errors::Errcode;

fn some_serialization(serializable: dyn Serialize) -> Result<(), Errcode> {
    let json_serialized = serializable.to_string()?;
    let deserialized = serde_json::from_str(json_serialized)?;
    std::fs::write(json_serialized.as_bytes())?;
}
```

# Avoiding code repetition

If you look at the code block defining all the `impl From<_> for Errcode` below,
you can see how this can be annoying after a while.

I repeated my code more than 2 times, it's worth spending an hour writing an
automation macro ! Luckily for you, I did it for you ;-)

There are 3 parts in the process we want to automate:

- Create an enum variant with a given name
- Include a type inside this enum variant
- Implement the `From<_> for Errcode` trait on the enum

## Getting the arguments of the macro

We will use the pattern `[ $( $name:ident : $class:ty ),+ ]`, meaning that
the macro `define_errcodes` will have the form
`define_errcodes![ NAME_A : CLASS_A, NAME_B : CLASS_B, ... ]`

- `[ .. ]` defines the characters before and after the arguments list
- `$( ... ),+` defines the **repetition**, saying that args are delimited by `,`
and that there is *at least one argument*.
- `( A : B )` means that the two arguments in one repetition are separated by a `:`
(it cannot be a `,` as  it would be confusing)
- `$name:ident` means `$name` is an Identifier, same as a variable / function name
- `$class:ty` means `$class` is a Type, it will be checked during the compilation

## Generating code for each argument

Inside our macro "code", we will use some `$( <code> )+` blocks,
which will loop through all our arguments and generate the `<code>` for
each of them.

So something like:

```rust
println!("Code before the loop");
$(
    println!("Name: $name, class: {:?}", $class");
)+
```

Would generate something like:

```rust
println!("Code before the loop");
println!("Name: NAME_A, class: {:?}", CLASS_A);
println!("Name: NAME_B, class: {:?}", CLASS_B);
println!("Name: NAME_C, class: {:?}", CLASS_C);
// ...
```

## Our final macro code

This is now what our `src/errors.rs` file look:

```rust
macro_rules! define_errcodes {
    [ $( $name:ident : $class:ty ),+ ] => {
        pub enum Errcode {
            $(
                $name($class),
            )+
        }

        $(
            impl From<$class> for Errcode {
                fn from(e: $class) -> Self {
                    Errcode::$name(e)
                }
            }
        )+
    };
}

define_errcodes![
    SerdeSerialization : serde_json::ser::Error,
    SerdeDeserialization : serde_json::de::Error,
    IoError : std::io::Error,
];
```

### Scoped yet global error handling

The good thing with this macro is that at any particular part of your project
you can create a new error type, and will just have to add it to the list !

Then you can have things like:

```rust
// src/errors.rs
define_errcodes![
    Server : crate::server::ServerError,
    Config : crate::config::ConfigError,
    Cli : crate::cli::CliParseError,
    Network: crate::network::NetworkError,
];
```

```rust
// src/server.rs
pub enum ServerError {
    PortBusy(u16),
}

pub struct Server {
    pub fn try_bind_port(&self, port: u16) -> Result<(), ServerError> {
        Err(ServerError::PortBusy)
    }
}
```

```rust
// src/main.rs
pub fn init_server() -> Result<(), Errcode> {
    let args = cli.parse_args()?;
    let config = Config::from_args(args)?;
    let server = Server::init(config)?;
    server.try_bind_port(8080)?;
    Ok(())
}
```

See how in the `init_server` everything is supposed to return a `Errcode` error case
but in our `try_bind_port` we return a `ServerError` ?
The `?` operator will perform the conversion itself, and that allows to write
clean and readable code, without all the error-handling-related code.
