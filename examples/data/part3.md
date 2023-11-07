

In this post, we're going to create the skeletton of the actual container, set up
the flow of our software and create an empty algorithm we'll fill with the parts needed
as we move on through the tutorial.

# Basis of the container
Now that the project basis are stable, the args are gathered and validated, we're going to extract
the configuration into a `ContainerOpts` struct and initialize a `Container` struct that will
have to perform the container work.

## The configuration
In a new file `src/config.rs`, let's define the [struct][rustbook-struct] for the container
configuration.
``` rust
use crate::errors::Errcode;

use std::ffi::CString;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ContainerOpts{
    pub path:       CString,
    pub argv:       Vec<CString>,

    pub uid:        u32,
    pub mount_dir:  PathBuf,
}
```
Let's analyse what we got there:
- **path**: The path of the binary / executable / script to execute inside the container
- **argv**: The full arguments passed (including the `path` option) into the commandline.
> These are required to perform an [execve syscall][syscall-execve] which we will use
> to contain our software in a process whose execution context is restricted.
- **uid**: The ID of the user inside the container. An ID of `0` means it's root (administrator).
> The user ID is visible on GNU/Linux by looking the file `/etc/passwd`, who has a format
> `username:x:uuid:guid:comment:homedir:shell`
- **mount_dir**: The path of the directory we want to use as a `/` root inside our container.   
Configurations will be added later as we need them.

We are using CString because it'll be much easier to use to call the `execve` syscall later.
Also as the configuration will be shared with the child process to be created, we need to be able
to `clone` the struct (which contains data stored on the [heap][so-stackheap]),
that's why we add a `derive(Clone)` attribute to the struct.   
(See [the chapter about ownership & data copy in the Book][rustbook-cloneandcopy])

Let's create the constructor of our configuration struct
``` rust
impl ContainerOpts{
    pub fn new(command: String, uid: u32, mount_dir: PathBuf) -> Result<ContainerOpts, Errcode> {
        let argv: Vec<CString> = command.split_ascii_whitespace()
            .map(|s| CString::new(s).expect("Cannot read arg")).collect();
        let path = argv[0].clone();

        Ok(
            ContainerOpts {
                path,
                argv,
                uid,
                mount_dir,
            }
        )
    }
}
```
Nothing too complicated here, we just get each arg from the command `String`, and creates a
`Vec<CString>` from them, cloning the first one, and creating the struct while returning an `Ok`
result.

## The container skeletton

Let's now create the Container `struct` that will perform the main tasks ahead.
``` rust
use crate::cli::Args;
use crate::errors::Errcode;
use crate::config::ContainerOpts;

pub struct Container{
    config: ContainerOpts,
}

impl Container {
    pub fn new(args: Args) -> Result<Container, Errcode> {
        let config = ContainerOpts::new(
            args.command,
            args.uid,
            args.mount_dir)?;
        Ok(Container {
            config,
            })
        }

    pub fn create(&mut self) -> Result<(), Errcode> {
        log::debug!("Creation finished");
        Ok(())
    }

    pub fn clean_exit(&mut self) -> Result<(), Errcode>{
        log::debug!("Cleaning container");
        Ok(())
    }
}
```
The `struct Container` is defined with a unique `config` field containing our configuration,
and implements 3 functions:
- `new` function that creates the `ContainerOpts` struct from the commandline arguments.
- `create` function that will handle the container creation process.
- `clean_exit` function that will be called before each exit to be sure we stay clean.   
For now we let them very basic and will fill them later on.

Finally, we create a function `start` that will get the args from the commandline and handle
everything from the `struct Container` creation to the exit.   
It returns a `Result` that will inform if an error happened during the process.

``` rust
pub fn start(args: Args) -> Result<(), Errcode> {
    let mut container = Container::new(args)?;
    if let Err(e) = container.create(){
        container.clean_exit()?;
        log::error!("Error while creating container: {:?}", e);
        return Err(e);
    }
    log::debug!("Finished, cleaning & exit");
    container.clean_exit()
}
```

The `?` you see at the end of the lines is used to propagate the errors.   
`let mut container = Container::new(args)?;` is the equivalent of:
``` rust
let mut container = match Container::new(args) {
    Ok(el) => el,
    Err(e) => return Err(e),
}
```
However for this, the type in case of `Err` has to be the same. That's why having a unique
`Errcode` for all our errors in the project is handy, we can basically put `?` everywhere and
"cascade" any error back to the `start` function which will call `clean_exit` before logging the
error and exit the process with an error return code thanks to the `exit_with_retcode` function
we defined in the part about errors handling.

## Linking to the main function
One last thing, we need to call the `start` function from our `main` function.   
In `src/main.rs`, add the following at the beginning of the file:
``` rust
mod container;
mod config;
```
And replace the `exit_with_retcode(Ok(()))` with `exit_with_retcode(container::start(args))`.

After testing, we get the following output:
```
[2021-10-02T14:16:41Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2021-10-02T14:16:41Z DEBUG crabcan::container] Creation finished
[2021-10-02T14:16:41Z DEBUG crabcan::container] Finished, cleaning & exit
[2021-10-02T14:16:41Z DEBUG crabcan::container] Cleaning container
[2021-10-02T14:16:41Z DEBUG crabcan::errors] Exit without any error, returning 0
```

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step5”][code-step5].   
The raw patch to apply on the previous step can be found [here][patch-step5]





# Checking the Linux kernel version
This step is entirely based on the [`<<check-linux-version>>` part of the original tutorial][tuto].   
However as I work on a much newer version of the kernel, I will just check that the kernel version
is at least the `v4.8` one, and that the architecture is `x86`.

## Getting system information
As we want to start interacting with the system to gather informations, we will start to use a
crate that will be massively usefull later, the [nix crate][nix-cratesio].

Let's check our kernel version:
``` rust
pub const MINIMAL_KERNEL_VERSION: f32 = 4.8;

pub fn check_linux_version() -> Result<(), Errcode> {
    let host = uname();
    log::debug!("Linux release: {}", host.release());

    if let Ok(version) = scan_fmt!(host.release(), "{f}.{}", f32) {
        if version < MINIMAL_KERNEL_VERSION {
            return Err(Errcode::NotSupported(0));
        }
    } else {
        return Err(Errcode::ContainerError(0));
    }

    if host.machine() != "x86_64" {
        return Err(Errcode::NotSupported(1));
    }

    Ok(())
}
```
In this code, we first get the information on the system using [uname][man-uname].   
From these informations, we get the kernel version as a `f32` float using the [scan_fmt crate][scanfmt-cratesio]
and check if it's at least the `v4.8`, then check if the machine architecture is `x86_64`.

## Handle errors
If the kernel version is too low or we have a wrong architecture, the function returns a
`Errcode::NotSupported`, with a number indicating what was not supported.   
If the scan_fmt fails, we return a `Errcode::ContainerError`, a new error type for the
"not supposed to happend at all" kind of errors, in our container.

Let's add these new errors to the `src/errors.rs` file:
``` rust
pub enum Errcode{
    ContainerError(u8),
    NotSupported(u8),
    ArgumentInvalid(&'static str),
}
```

## Add to flow & test
As we will use a macro from the `scan_fmt` crate, let's import it in our `src/main.rs`:
``` rust
#[macro_use] extern crate scan_fmt;
```

And add the needed dependancies in the `Cargo.toml` file:
``` toml
[dependancies]
# ...
nix = "0.22.1"
scan_fmt = "0.2.6"
```

Finally, let's insert the `check_linux_version` function into the flow of our `start` function in
`src/container.rs`:
``` rust
pub fn start(args: Args) -> Result<(), Errcode> {
    check_linux_version()?;
    let mut container = Container::new(args)?;
    // ...
}
```
> I wont write again how errors handling are so elegant in Rust, but check out how we wired a new
> function into the flow without needing any additionnal line of code to handle its errors.

After testing that's the kind of output we get:
```
[2021-10-02T14:50:14Z INFO  crabcan] Args { debug: true, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2021-10-02T14:50:14Z DEBUG crabcan::container] Linux release: 5.11.0-36-generic
[2021-10-02T14:50:14Z DEBUG crabcan::container] Creation finished
[2021-10-02T14:50:14Z DEBUG crabcan::container] Finished, cleaning & exit
[2021-10-02T14:50:14Z DEBUG crabcan::container] Cleaning container
[2021-10-02T14:50:14Z DEBUG crabcan::errors] Exit without any error, returning 0
```

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step6”][code-step6].   
The raw patch to apply on the previous step can be found [here][patch-step6]

[rustbook-struct]: https://doc.rust-lang.org/book/ch05-01-defining-structs.html
[syscall-execve]: https://man7.org/linux/man-pages/man2/execve.2.html
[rustbook-cloneandcopy]: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#ways-variables-and-data-interact-clone
[so-stackheap]: https://stackoverflow.com/questions/79923/what-and-where-are-the-stack-and-heap
[tuto]: https://blog.lizzie.io/linux-containers-in-500-loc.html
[nix-cratesio]: https://crates.io/crates/nix
[man-uname]: https://man7.org/linux/man-pages/man2/uname.2.html
[scanfmt-cratesio]: https://crates.io/crates/scan_fmt

[code-step5]: https://github.com/litchipi/crabcan/tree/step5/
[patch-step5]: https://github.com/litchipi/crabcan/compare/step4..step5.diff
[code-step6]: https://github.com/litchipi/crabcan/tree/step6/
[patch-step6]: https://github.com/litchipi/crabcan/compare/step5..step6.diff
