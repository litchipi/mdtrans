In this post, we will prepare the field for our implementation.

Every programmer is different and some of you may go  directly into Linux syscalls
to create an isolated box and so on ...
I prefer to always create a clean ground on which lay off my implementations, which
I find is much more practical to read/understand later on, and it's always a good thing
to practice clean implementations.   
That will also provide bonus tips and tools about topics that may interest some less experiences
Rust users, like **argument parsing**, **errors handling**, **logging**, etc ...

I will assume here that you already have `rustc` and `cargo` installed, if you don't, please
follow the instructions on [the Book](https://doc.rust-lang.org/book/ch01-01-installation.html)

# Create the project

So I guess you've heard that Rust's mascot is **Ferris**, the little cutie crab.  
Well, let's put Ferris in a container ! :D

![Wanna eat Ferris ?](/images/container_in_rust/crabcan.png)

We'll create a Rust binary project called **Crabcan**, and the objective will be to separate the
different parts of the projects as distincly as possible to allow us to search in the code,
tweak it and understand it again after months of pause.

Run the command `cargo new --bin crabcan` to create the project.   
This will generate a `Cargo.toml` file in which we can describe our project, add dependencies and
tweak configurations of the rust compiler, a handy file to avoid having to avoid having to
create `rustc` commands by hand in a Makefile.
You can change the author name, e-mail and version of your project here, but we wont add any
dependencies yet.

In the folder `src/` you will put, well, all your sources.
For now there's only a `main.rs` file with a `Hello World!` code inside.

# Parse the arguments
Ok let's dive directly into our project. First of all let's get the arguments from the command line.
The objective is to get configurations from text-written flags while calling our tool.

**Command example**
```
crabcan --mount ./mountdir/ --uid 0 --debug --command "bash"
```
This command will call `crabcan` with the folder `mountdir` to mount as root for the container,
the UID number `0`, will output all `debug` messages, and will execute the command `bash` inside
the container.

## Introducing the `structopt` crate
The [structopt crate][structopt-cratesio] is a very usefull tool to parse arguments from the
commandline (using the `clap` crates as a backend).
The method is very straightforward, by defining a [struct][rustbook-struct] containing all the arguments:
```rust
#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool

    // etc ...
}
```
A detailed use of structopt and all its power is available in its [documentation][structopt-docs].
One thing worth noticing is that the `/// text` part above an argument defined in the struct
will be used as a message inside the helper (when you type `crabcan --help` for example).

## Creating our argument parsing
We are going to create a new file `src/cli.rs` containing everything related to commandline.
For it to be used inside our project, we have to include it as a [module][rustbook-module] of the
project.

In `src/main.rs` we replace the content with the following:
```rust
mod cli;

fn main() {
    let args = cli::parse_args();
}
```
Basically we expect the `src/cli.rs` file to provide a function `parse_args` that will return the
struct containing all our configuration defined by the user through the commandline.   
*Note that because `args` is not used, you will get a compiler warning.*

Now let's implement that function `parse_args` in `src/cli.rs`:
```rust
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "crabcan", about = "A simple container in Rust.")]
pub struct Args {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Command to execute inside the container
    #[structopt(short, long)]
    pub command: String,

    /// User ID to create inside the container
    #[structopt(short, long)]
    pub uid: u32,

    /// Directory to mount as root of the container
    #[structopt(parse(from_os_str), short = "m", long = "mount")]
    pub mount_dir: PathBuf,
}

pub fn parse_args() -> Args {
    let args = Args::from_args();

    // If args.debug: Setup log at debug level
    // Else: Setup log at info level

    // Validate arguments

    args
}
```
So here we first import our necessary dependencies `structopt` but also `PathBuf` from the standard
library.   
Then we define our `Args` struct, containing all the arguments and informations to be used for
argument parsing. Let's look what arguments we are expecting:
- `debug`: Will be used to display debug messages or just normal logs
- `command`: The command that will be executed inside the container (with arguments)
- `uid`: The user ID that will be created to run the software inside the container.
- `mount_dir`: The folder to use as a root `/` directory inside the container.   
*Note that this argument will be passed as **mount** in the commandline*

These arguments are defined with the [macro attribute][rustbook-macroattribute]
`structopt(short, long)` to automatically create a short and long commandline argument from the
field name.
(The field `toto` will be defined as arguments `-t` and `--toto`).

Finally, we create the `parse_args` in which we gather the arguments from the commandline with the
`from_args` function of the struct
(which  was generated thanks to the `derive(StructOpt)` [macro attribute][rustbook-macroattribute]).

After setting some placeholders for arguments validation and logging initialisation, we return
the arguments.

One last thing, add the dependencies we just imported inside the `Cargo.toml` file:
```toml
# ...
[dependencies]
structopt = "0.3.23"
```

## Testing our code

Let's test our code with `cargo run`:
```
error: The following required arguments were not provided:
    --command <command>
    --mount <mount-dir>
    --uid <uid>

USAGE:
    crabcan [FLAGS] --command <command> --mount <mount-dir> --uid <uid>

For more information try --help
```
And this is it, our argument parsing works !
Now if we try `cargo run -- --mount ./ --uid 0 --command "bash" --debug`, we don't get any errors.
You can add a `println!("{:?}", args)` in our `src/main.rs` file to get a nice output:
```
Args { debug: true, command: "bash", uid: 0, mount_dir: "./" }
```

### Patch for this step
The code for this step is available on github [litchipi/crabcan branch "step1"][code-step1].   
The raw patch to apply on a freshly created project using `cargo new --bin` can be found [here][patch-step1]

# Setup Logging

## The logging crates
Now that we got from the user its input, let's set up a way to give him outputs.
Simple text is enough, but we want to separate debug informations from basic informations and
errors. For this, there's a lot of tools, but I chose the crates `log` and `env_logger` to perform
this task.

The [log crate][log-cratesio] is a very used tool to perform logging. It provides a `Log`
trait ([see the Book for traits explanation][rustbook-trait]) which defines all the function a logger has to have, and lets any other
crate implement these functions. I chose the [env_logger crate][env_logger-cratesio] to implement
these.

In `Cargo.toml`, we add the following dependencies:
``` toml
# ...
log = "0.4.14"
env_logger = "0.9.0"
```

## Setting up logging
Loggers have to be initialized with a level of verbosity. This will define wether to display
debug messages, or only errors, or nothing at all.
On our case, we want it to display normal informations by default, and increase verbosity to
debug messages when the `--debug` flag is passed through the commandline.

Let's initialize our logger in `src/cli.rs`:
``` rust
pub fn setup_log(level: log::LevelFilter){
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .filter(None, level)
        .init();
}
```
Yeah, a function is not really needed, but it's more readable isn't it ?   
If you are into [Rust code optimisation][rustcodeopti], you may want to
[inline this function][rustbook-inlining].

Ok, now let's actually initialize logging right after getting the arguments from the commandline,
in the `parse_args` functions, let's replace the placeholders with this piece of code:
``` rust
if args.debug{
    setup_log(log::LevelFilter::Debug);
} else {
    setup_log(log::LevelFilter::Info);
}
```

## Logging

Now that everything is in place, let's actually log something in our terminal !
In the `main` function of `src/main.rs`, we can output the args gotten into a `info` message.
This is done using the `log::info!` [macro][rustbook-macros].
``` rust
log::info!("{:?}", args);
```
The `log` crate allows us to use `error!`, `warn!`, `info!`, `debug!` or `trace!` message levels.

After testing we get the output:
```
[2021-09-30T10:17:46Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
```

### Patch for this step
The code for this step is available on github [litchipi/crabcan branch "step2"][code-step2].   
The raw patch to apply on the previous step can be found [here][patch-step2]

# Prepare errors handling
As a general practise it's good to take care of handling errors.
When it comes to Rust, this langage is far too powerfull concerning errors handling to ignore
them and not exploit them.

I am no-one to teach how to properly handle errors, but this part will give an example of how
errors can be managed in a large Rust project, and use Rust specific tools to handle them more
easily.

## The Errcode enum
Let's create a `src/errors.rs` file in which we'll define the following [enum][rustbook-enum]:
``` rust
// Allows to display a variant with the format {:?}
#[derive(Debug)]
// Contains all possible errors in our tool
pub enum Errcode{
}
```
Each time we will add a new error type, we'll add a variant to this enum.
The `derive(Debug)` allows the enum to be displayed using a `{:?}` format.

But we may want to display a more complete message for each variant, allowing us to not get lost
in codes and different numbers around our project.
For this, let's implement the `std::fmt::Display` [trait][rustbook-trait], defining the behaviour
of an object when attempting to display it in a regular `{}` format.
``` rust
use std::fmt;

#[allow(unreachable_patterns)]
// trait Display, allows Errcode enum to be displayed by:
//      println!("{}", error);
//  in this case, it calls the function "fmt", which we define the behaviour below
impl fmt::Display for Errcode {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Define what behaviour for each variant of the enum
        match &self{
            _ => write!(f, "{:?}", self) // For any variant not previously covered
        }
    }
}
```
> The `unreachable_patterns` attribute ensure that we won't get any warning from the compiler
> if the `match` statement describe all the variants.

## Linux return codes
Linux executable returns a number when they exit, which describe how everything went.
A return code of **0** means that there was no errors, and any other number describe an
error and what that error is (based on the return code value).
You can find [here][exitcode-meanings] a table of special return codes and their meaning.

We do not seek to perform bash automation scripts here with our tool, but we'll just set up a
way to return 0 if there was no errors, and 1 if there was an error.   
In our `src/errors.rs` file, let's define a `exit_with_errcode` function:
``` rust
use std::process::exit;

// Get the result from a function, and exit the process with the correct error code
pub fn exit_with_retcode(res: Result<(), Errcode>) {
    match res {
        // If it's a success, return 0
        Ok(_) => {
            log::debug!("Exit without any error, returning 0");
            exit(0);
        },

        // If there's an error, print an error message and return the retcode
        Err(e) => {
            let retcode = e.get_retcode();
            log::error!("Error on exit:\n\t{}\n\tReturning {}", e, retcode);
            exit(retcode);
        }
    }
}
```

This function exit the process with a return code got from the `get_retcode` function implemented
by our `Errcode` enum. Let's implement it in the most easy and stupid way:
``` rust
impl Errcode{
    // Translate an Errcode::X into a number to return (the Unix way)
    pub fn get_retcode(&self) -> i32 {
        1 // Everything != 0 will be treated as an error
    }
}
```

## Result in Rust
When a piece of the code is not working properly, we can handle the error using a `Result` in Rust
([see the Book for detailed explanation][rustbook-results]).   
`Result<T, U>` expects two types, one type `T` to return if it's a success, one type `U` to return
if there's an error. In our case, we want to return an `Errcode` if there's an error, and return
whatever we want if everything goes well.

Let's see how we can set this up in the `parse_args` function:
``` rust
pub fn parse_args() -> Result<Args, Errcode> {
    // ...
    Ok(args)
 }
```
If something goes wrong during the execution, we can simply write:
``` rust
return Err(Errcode::MyErrorType);
```

> The `Result` in Rust are very usefull and powerfull and it's generally a good idea to use it
> everywhere you want error handling as it's the standard Rust way to do.

Okay, but now we need to do something different in our `main` depending on how the function ended
with an error or a success. Let's use a `match` statement to define what to do in both cases:
``` rust
match cli::parse_args(){
    Ok(args) => {
        log::info!("{:?}", args);
        exit_with_retcode(Ok(()))
    },
    Err(e) => {
        log::error!("Error while parsing arguments:\n\t{}", e);
        exit(e.get_retcode());
    }
};
```
Here, in case the arguments parsing was successful, we log the args and call the
`exit_with_retcode` with an `Ok(())` value (it will simply exit with the return code 0).
That is where we're going to place our container starting point later.

In case there was an error, we log it (notice the `{}` format on our `Errcode` that will
call the `fmt` function of the `Display` trait we implemented earlier), and simply exit with the
return code associated.

One final step, we have to set `src/errors.rs` as a module of our project, and import the
`exit_with_retcode` function in our `src/main.rs` file.
``` rust
mod errors;

use errors::exit_with_retcode;
```

After testing, we can get the following output:
```
[2021-09-30T13:47:45Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2021-09-30T13:47:45Z DEBUG crabcan::errors] Exit without any error, returning 0
```

### Patch for this step
The code for this step is available on github [litchipi/crabcan branch "step3"][code-step3].   
The raw patch to apply on the previous step can be found [here][patch-step3]

> Thanks to `filtoid` for his PR fixing an error in the code of this step

# Validate arguments

Before diving into the real work, let's validatet the arguments passed from the commandline.
We will just check that the `mount_dir` actually exists, but this part can be extended with
additionnal checks, as we add more options, etc ...   
Let's replace the placeholders in `src/cli.rs` with the actual arguments validation:
``` rust
pub fn parse_args() -> Result<Args, Errcode> {
    // ...
    if !args.mount_dir.exists() || !args.mount_dir.is_dir(){
        return Err(Errcode::ArgumentInvalid("mount"));
    }
    // ...
}
```
The condition checks if the path (a `PathBuf` type as we defined in our `Args` struct) exists
and if it's a directory.

If it isn't, we return a `Result::Err` with our `Errcode` enum with a custom variant
`ArgumentInvalid`, specifying that the fault was on argument `mount`.   
In `src/errors.rs`, we will define this variant:
``` rust
pub enum Errcode{
    ArgumentInvalid(&'static str),
}
```
And we can add in the `match` statement of the `fmt` function the following:
``` rust
match &self{
    // Message to display when an argument is invalid
    Errcode::ArgumentInvalid(element) => write!(f, "ArgumentInvalid: {}", element),

    // ...
}
```

### Patch for this step
The code for this step is available on github [litchipi/crabcan branch "step4"][code-step4].   
The raw patch to apply on the previous step can be found [here][patch-step4]

> **Special thanks** to *@kevinji* who pointed out an error in the code of this step :D

[rust-the-book]: https://doc.rust-lang.org/book/
[rustbook-struct]: https://doc.rust-lang.org/stable/book/ch05-01-defining-structs.html
[rustbook-module]: https://doc.rust-lang.org/stable/book/ch07-02-defining-modules-to-control-scope-and-privacy.html
[rustbook-macroattribute]: https://doc.rust-lang.org/stable/book/ch19-06-macros.html?#attribute-like-macros
[rustbook-trait]: https://doc.rust-lang.org/book/ch10-02-traits.html
[rustbook-inlining]: https://nnethercote.github.io/perf-book/inlining.html
[rustbook-macros]: https://doc.rust-lang.org/book/ch19-06-macros.html
[rustbook-enum]: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
[rustbook-results]: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
[log-cratesio]: https://crates.io/crates/log
[env_logger-cratesio]: https://crates.io/crates/env_logger
[structopt-cratesio]: https://crates.io/crates/structopt
[structopt-docs]: https://docs.rs/structopt/latest/structopt/
[code-step1]: https://github.com/litchipi/crabcan/tree/step1
[patch-step1]: https://github.com/litchipi/crabcan/compare/main..step1.diff
[code-step2]: https://github.com/litchipi/crabcan/tree/step2
[patch-step2]: https://github.com/litchipi/crabcan/compare/step1..step2.diff
[code-step3]: https://github.com/litchipi/crabcan/tree/step3
[patch-step3]: https://github.com/litchipi/crabcan/compare/step2..step3.diff
[code-step4]: https://github.com/litchipi/crabcan/tree/step4
[patch-step4]: https://github.com/litchipi/crabcan/compare/step3..step4.diff
[rustcodeopti]: https://gist.github.com/jFransham/369a86eff00e5f280ed25121454acec1
[exitcode-meanings]: https://tldp.org/LDP/abs/html/exitcodes.html
