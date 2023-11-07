The container is almost working, the only thing we have left is to add the binary
execution inside our code.

# The `execve` syscall

When Linux is told to execute a software, wether it's a binary or a text script
(if the first line is `#!<interpreter>`), behind the curtains it calls the `execve`
that takes 3 arguments:

- The paths of the script / binary
- The arguments to pass to the executable
- The environment variables to set

The linux kernel will then **replace** the current process with the executable.
This is why we needed to *clone* our main process before calling this syscall.

> Check out the `execve` syscall documentation [here][execvedoc]

Everything is already set up for our execution, so we only have to call the syscall
inside the `child` function of `src/child.rs`:

``` rust
use nix::unistd::execve;
use std::ffi::CString;

fn child(config: ContainerOpts) -> isize {

    // ...

    log::info!("Starting container with command {} and args {:?}", config.path.to_str().unwrap(), config.argv);
    let retcode = match execve::<CString, CString>(&config.path, &config.argv, &[]){
        Ok(_) => 0,
        Err(e) => {
            log::error!("Error while trying to perform execve: {:?}", e);
            -1
        }
    };
    retcode
}
```

The only tricky bit here is to tell Rust that we are using `CString` as it has to
have some compatibilities with raw C.

> In case of success, the `execve` function will never return as the process will
be **replaced** by the executable

We'll want also to check that the command is provided by the user and valid, so
in `src/cli.rs` let's modify the `parse_args` functions to add this check:

``` rust
pub fn parse_args() -> Result<Args, Errcode> {
    // ...

    if args.command.is_empty() {
        return Err(Errcode::ArgumentInvalid("command"));
    }

    Ok(args)
}
```

## Testing

The implementation was straightforward, let's test this out :

``` bash
$ mkdir -p ./mountdir/bin
$ cp /bin/bash ./mountdir/bin
$ cargo build
# ...
$ sudo target/debug/crabcan --debug -u 0 -m ./mountdir -c "/bin/bash"
[2022-08-23T08:37:01Z INFO  crabcan] Args { debug: false, command: "/bin/bash", uid: 0, mount_dir: "./mountdir/" }
[2022-08-23T08:37:01Z INFO  crabcan::namespaces] User namespaces set up
[2022-08-23T08:37:01Z INFO  crabcan::child] Container set up successfully
[2022-08-23T08:37:01Z INFO  crabcan::child] Starting container with command /bin/bash and args ["/bin/bash"]
[2022-08-23T08:37:01Z ERROR crabcan::child] Error while trying to perform execve: ENOENT
```

Aouch, that doesn't look good we got `ENOENT` !
When we check the [execve documentation][execvedoc], we can read:

> **ENOENT** The file pathname or a script or ELF interpreter does not exist.

Why that ? We copied the binary inside the mount point, every file is there to
allow execution !

... Are they ?

### Dynamically linked binaries

When a program is compiled, it requires a lot of dependencies, even for a small
Hello World, such as the standard library of the programming langage or
the bindings to the underlying operating system.

It is possible to compile a binary statically, but it produces a big file, and
5 software compiled this way would embed in each of them 5 chunks of software
that could have been shared.

This is why most binaries uses shared libraries. In Linux, you can see the
libraries that are dynamically linked to an binary by executing the command

``` bash
$ ldd /bin/bash
linux-vdso.so.1 (0x00007ffca9d9e000)
libtinfo.so.6 => /lib/x86_64-linux-gnu/libtinfo.so.6 (0x00007f7911136000)
libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f7910f0e000)
/lib64/ld-linux-x86-64.so.2 (0x00007f79112dd000)
```

Here we can see that our `/bin/bash` executable depends on 4 files located
at the root of the system.

### Compile a test binary statically

We will deal with this dynamic binaries problems later, for now we want
to try our `execve`, so let's try it with a static binary which only
requires the binary file to work !
Let's create a little crate inside our repository, and compile it into
a statically built binary blob.

The process may look a little bit esoteric, but we are simply telling the
compiler to bundle the dynamic dependencies inside the binary instead of
referencing them, and as it depend on the system, we compile against a
specific target.

```
cargo new --bin testbin
cd testbin
RUSTFLAGS="-C target-feature=+crt-static" cargo build --target="x86_64-unknown-linux-gnu"
cp target/debug/testbin ../mountdir/
cd ..
```

> To know more about the linking process in Rust, check out [this doc][rustlink]

Now that we have a static binary, we can test out our container properly

```
[2022-08-23T07:58:45Z INFO  crabcan] Args { debug: true, command: "/testbin", uid: 0, mount_dir: "./mountdir/" }
[2022-08-23T07:58:45Z DEBUG crabcan::container] Linux release: 5.13.0-52-generic
[2022-08-23T07:58:45Z DEBUG crabcan::container] Container sockets: (3, 4)
[2022-08-23T07:58:45Z DEBUG crabcan::resources] Restricting resources for hostname triangular-coffee-39
[2022-08-23T07:58:45Z DEBUG crabcan::hostname] Container hostname is now triangular-coffee-39
[2022-08-23T07:58:45Z DEBUG crabcan::mounts] Setting mount points ...
[2022-08-23T07:58:45Z DEBUG crabcan::mounts] Mounting temp directory /tmp/crabcan.yOXBrf4FO8v0
[2022-08-23T07:58:45Z DEBUG crabcan::mounts] Pivoting root
[2022-08-23T07:58:45Z DEBUG crabcan::mounts] Unmounting old root
[2022-08-23T07:58:45Z DEBUG crabcan::namespaces] Setting up user namespace with UID 0
[2022-08-23T07:58:45Z DEBUG crabcan::namespaces] Child UID/GID map done, sending signal to child to continue...
[2022-08-23T07:58:45Z DEBUG crabcan::container] Creation finished
[2022-08-23T07:58:45Z DEBUG crabcan::container] Container child PID: Some(Pid(2354182))
[2022-08-23T07:58:45Z DEBUG crabcan::container] Waiting for child (pid 2354182) to finish
[2022-08-23T07:58:45Z INFO  crabcan::namespaces] User namespaces set up
[2022-08-23T07:58:45Z DEBUG crabcan::namespaces] Switching to uid 0 / gid 0...
[2022-08-23T07:58:45Z DEBUG crabcan::capabilities] Clearing unwanted capabilities ...
[2022-08-23T07:58:45Z DEBUG crabcan::syscalls] Refusing / Filtering unwanted syscalls
[2022-08-23T07:58:45Z DEBUG syscallz] seccomp: setting action=Errno(1) syscall=chmod comparators=[Comparator { arg: 1, op: MaskedEq, datum_a: 2048, datum_b: 2048 }]
 ...
[2022-08-23T07:58:45Z DEBUG syscallz] seccomp: loading policy
[2022-08-23T07:58:45Z INFO  crabcan::child] Container set up successfully
[2022-08-23T07:58:45Z INFO  crabcan::child] Starting container with command /testbin and args ["/testbin"]
Hello, world!
[2022-08-23T07:58:45Z DEBUG crabcan::container] Finished, cleaning & exit
[2022-08-23T07:58:45Z DEBUG crabcan::container] Cleaning container
[2022-08-23T07:58:45Z DEBUG crabcan::resources] Cleaning cgroups
[2022-08-23T07:58:45Z DEBUG crabcan::errors] Exit without any error, returning 0
```

Yipee ! We got our **Hello, world!**, our container is now able to launch executables

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step15”][code-step15].   
The raw patch to apply on the previous step can be found [here][patch-step15]

# Mounting additionnal paths

The last bit we need for a usefull container is to allow some dynamic binaries
to be used inside our container. We could simply copy everything each time, and
it can be better in terms of security, but we'll use a lighter method for now,
let's mount directories inside our container !

The idea is simple, when creating the container, we pass a list of directory of
the hosts and where they should appear inside the container. Then we "mount" them
and can freely access to them.

Let's add a parameter to the CLI for these additionnal paths to mount, in `src/cli.rs`:

``` rust
pub struct Args {
    // ...

    /// Mount a directory inside the container
    #[structopt(parse(from_os_str), short = "a", long = "add")]
    pub addpaths: Vec<PathBuf>,
}
```

These arguments will be used to have a pair of `from_dir -> to_dir`, and pass
them to the container configuration.
In `src/container.rs`:

``` rust
impl Container {
    pub fn new(args: Args) -> Result<Container, Errcode> {
        let mut addpaths = vec![];
        for ap_pair in args.addpaths.iter(){
            let mut pair = ap_pair.to_str().unwrap().split(":");
            let frompath = PathBuf::from(pair.next().unwrap())
                .canonicalize().expect("Cannot canonicalize path")
                .to_path_buf();
            let mntpath = PathBuf::from(pair.next().unwrap())
                .strip_prefix("/").expect("Cannot strip prefix from path")
                .to_path_buf();
            addpaths.push((frompath, mntpath));
        }

        let (config, sockets) = ContainerOpts::new(
                args.command,
                args.uid,
                args.mount_dir,
                addpaths)?;

        // ...
    }
}
```

Let's add this new parameter in the function `ContainerOpts::new` of the file `src/config.rs`:

``` rust
pub struct ContainerOpts{
    // ...
    pub addpaths:   Vec<(PathBuf, PathBuf)>,
}


impl ContainerOpts{
    pub fn new(command: String, uid: u32, mount_dir: PathBuf, addpaths: Vec<(PathBuf, PathBuf)>)
            -> Result<(ContainerOpts, (RawFd, RawFd)), Errcode> {
        // ...
        Ok((
            ContainerOpts {
                // ...
                addpaths,
            },
            sockets
        ))
    }
}

```

Then, they are passed as an additionnal argument of the `setmountpoint` function
and used to create the mountpoint and perform the `mount` operation.
In the file `src/mounts.rs`:

``` rust
pub fn setmountpoint(mount_dir: &PathBuf, addpaths: &Vec<(PathBuf, PathBuf)>) -> Result<(), Errcode> {
    // ...

    log::debug!("Mounting additionnal paths");
    for (inpath, mntpath) in addpaths.iter(){
        let outpath = new_root.join(mntpath);
        create_directory(&outpath)?;
        mount_directory(Some(inpath), &outpath, vec![MsFlags::MS_PRIVATE, MsFlags::MS_BIND])?;
    }

    log::debug!("Pivoting root");
    // ...
}

```

> **Security Note**: You do not want all mounts to be read/write, especially when
sharing system directories. An additionnal `MS_RDONLY` flag should be added for them.
(You could have `-a` for read/write directories, and `-r` for read only ones)

Finally, let's pass the new additionnal parameter to the `setmountpoint` inside
the `setup_container_configurations` function, in `src/child.rs`:

``` rust
fn setup_container_configurations(config: &ContainerOpts) -> Result<(), Errcode> {
    // ...
    setmountpoint(&config.mount_dir, &config.addpaths)?;
    // ...
}
```

## Testing

Now we can finally test our container with additionnal paths passed using
mutliple pairs of the form `from:to`, like so:

``` bash
$ mkdir -p ./mountdir
$ cargo build
$ cp /bin/bash ./mountdir
$ cp /bin/ls ./mountdir
$ sudo ./target/debug/crabcan --debug -u 0 -m ./mountdir/ -c "/bash" -a /lib64:/lib64 -a /lib:/lib
[2022-08-23T09:27:34Z INFO  crabcan] Args { debug: true, command: "/bash", uid: 0, mount_dir: "./mountdir/", addpaths: ["/lib64:/lib64", "/lib:/lib"] }
[2022-08-23T09:27:34Z DEBUG crabcan::container] Linux release: 5.13.0-52-generic
[2022-08-23T09:27:34Z DEBUG crabcan::container] Container sockets: (3, 4)
[2022-08-23T09:27:34Z DEBUG crabcan::resources] Restricting resources for hostname blue-pinguin-161
[2022-08-23T09:27:34Z DEBUG crabcan::hostname] Container hostname is now blue-pinguin-161
[2022-08-23T09:27:34Z DEBUG crabcan::mounts] Setting mount points ...
[2022-08-23T09:27:34Z DEBUG crabcan::mounts] Mounting temp directory /tmp/crabcan.0j3XHGRYJMf7
[2022-08-23T09:27:34Z DEBUG crabcan::mounts] Mounting additionnal paths
[2022-08-23T09:27:34Z DEBUG crabcan::mounts] Pivoting root
[2022-08-23T09:27:34Z DEBUG crabcan::mounts] Unmounting old root
[2022-08-23T09:27:34Z DEBUG crabcan::namespaces] Setting up user namespace with UID 0
[2022-08-23T09:27:34Z DEBUG crabcan::namespaces] Child UID/GID map done, sending signal to child to continue...
[2022-08-23T09:27:34Z DEBUG crabcan::container] Creation finished
[2022-08-23T09:27:34Z DEBUG crabcan::container] Container child PID: Some(Pid(2412263))
[2022-08-23T09:27:34Z DEBUG crabcan::container] Waiting for child (pid 2412263) to finish
[2022-08-23T09:27:34Z INFO  crabcan::namespaces] User namespaces set up
[2022-08-23T09:27:34Z DEBUG crabcan::namespaces] Switching to uid 0 / gid 0...
[2022-08-23T09:27:34Z DEBUG crabcan::capabilities] Clearing unwanted capabilities ...
[2022-08-23T09:27:34Z DEBUG crabcan::syscalls] Refusing / Filtering unwanted syscalls
[2022-08-23T09:27:34Z DEBUG syscallz] seccomp: setting action=Errno(1) syscall=chmod comparators=[Comparator { arg: 1, op: MaskedEq, datum_a: 2048, datum_b: 2048 }]
 ...
[2022-08-23T09:27:34Z DEBUG syscallz] seccomp: loading policy
[2022-08-23T09:27:34Z INFO  crabcan::child] Container set up successfully
[2022-08-23T09:27:34Z INFO  crabcan::child] Starting container with command /bash and args ["/bash"]
bash-5.1# ./ls
bash  lib  lib64  ls
bash-5.1# exit
[2022-08-23T09:27:36Z DEBUG crabcan::container] Finished, cleaning & exit
[2022-08-23T09:27:36Z DEBUG crabcan::container] Cleaning container
[2022-08-23T09:27:36Z DEBUG crabcan::resources] Cleaning cgroups
[2022-08-23T09:27:36Z DEBUG crabcan::errors] Exit without any error, returning 0
```

And voilà ! We just called a dynamically linked binary from inside our
container !

### Patch for this step

The code for this step is available on github [litchipi/crabcan branch “step16”][code-step16].   
The raw patch to apply on the previous step can be found [here][patch-step16]

# Closing thoughts

If you have to remember only one thing of this part, remember that
**YOU SHOULD NOT USE THIS TOY PROJECT FOR ANYTHING SERIOUS**,
use one of the many containerisation solutions instead

## From there to Docker

It may seams like we arrived to a very minimalist state of a container, but once
you can add files and execute binaries, you can do pretty much anything from there.
This environment is already enough to execute a web service, or as a compilation
sandbox, but it lacks severall things to get to the level comparable to Docker:

- **Network isolation**: Creating fake interfaces, bind them to the real network
hardware, but isolated enough to not have any bad surprises
- **Port forwarding**: Executing a web service on port 80 of the container and
redirect the data to whatever local port you wish
- **Good security**: Security is hard because it has so many pitfalls.
- **Container generation from file**: A `Crabfile` with a weird syntax to build
a container from text

And many more things that I am forgetting.

## End of this tutorial

This was the last post of this tutorial that took way too long to write.
Feel free to reach to me if you have any question or remark, or leave a
comment using your Github Account.

I tried to keep it beginer-friendly in every aspects, as what I love to
do when learning a langage is build something real with it.

The code of this tutorial is a Rust rewrite from [this tutorial][linux-containers-tutorial],
you **should** go and take a look at least to it as it's filled with
comments, experiments, remarks and interesting thoughts that are
not present here.

I will credit you for any contribution, error rectification, and any
content you think could be added to the text of any of these posts.

Take care, happy coding <3

[code-step15]: https://github.com/litchipi/crabcan/tree/step15/
[patch-step15]: https://github.com/litchipi/crabcan/compare/step14..step15.diff

[code-step16]: https://github.com/litchipi/crabcan/tree/step16/
[patch-step16]: https://github.com/litchipi/crabcan/compare/step15..step16.diff

[execvedoc]: https://man7.org/linux/man-pages/man2/execve.2.html
[rustlink]: https://doc.rust-lang.org/reference/linkage.html
[linux-containers-tutorial]: https://blog.lizzie.io/linux-containers-in-500-loc.html
